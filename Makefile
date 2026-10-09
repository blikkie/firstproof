.PHONY: all check build-core build-bridge build-wasm build-web clean test test-all run

ARCH := $(shell uname -m)
ifeq ($(ARCH),arm64)
	RUST_TARGET := aarch64-apple-darwin
else
	RUST_TARGET := x86_64-apple-darwin
endif

all: build-bridge

check:
	./scripts/check.sh

build-core:
	GIT_CONFIG_GLOBAL=/dev/null cargo build --release --target $(RUST_TARGET) -p firstproof-core

build-bridge: build-core
	mkdir -p FirstProofBridge/Sources/FirstProofBridge
	mkdir -p FirstProofBridge/Sources/FirstProofCoreFFI/include
	mkdir -p FirstProofBridge/Sources/FirstProofShaderTypes/include
	GIT_CONFIG_GLOBAL=/dev/null cargo run --manifest-path firstproof-core/Cargo.toml --target $(RUST_TARGET) --features uniffi/cli --bin uniffi-bindgen -- generate \
		--library target/$(RUST_TARGET)/release/libfirstproof_core.dylib \
		--language swift \
		--out-dir FirstProofBridge/Sources/FirstProofBridge/ \
		--no-format \
		--metadata-no-deps
	mv FirstProofBridge/Sources/FirstProofBridge/firstproof_coreFFI.h FirstProofBridge/Sources/FirstProofCoreFFI/include/
	mv FirstProofBridge/Sources/FirstProofBridge/firstproof_engineFFI.h FirstProofBridge/Sources/FirstProofCoreFFI/include/
	rm -f FirstProofBridge/Sources/FirstProofBridge/firstproof_*FFI.modulemap
	printf 'module firstproof_coreFFI {\n    header "firstproof_coreFFI.h"\n    export *\n}\n\nmodule firstproof_engineFFI {\n    header "firstproof_engineFFI.h"\n    export *\n}\n' > FirstProofBridge/Sources/FirstProofCoreFFI/include/module.modulemap
	cp FirstProofApp/Sources/Shaders/ShaderTypes.h FirstProofBridge/Sources/FirstProofShaderTypes/include/ShaderTypes.h
	cp target/$(RUST_TARGET)/release/libfirstproof_core.a FirstProofBridge/Sources/FirstProofCoreFFI/include/
	swift build --package-path FirstProofBridge

build-app: build-bridge
	swift build --package-path FirstProofApp

run: build-bridge
	swift run --package-path FirstProofApp

bundle: build-app
	mkdir -p dist/FirstProof.app/Contents/MacOS
	mkdir -p dist/FirstProof.app/Contents/Resources
	cp FirstProofApp/Resources/Info.plist dist/FirstProof.app/Contents/
	cp FirstProofApp/Resources/AppIcon.icns dist/FirstProof.app/Contents/Resources/
	cp FirstProofApp/Sources/Shaders/*.metal dist/FirstProof.app/Contents/Resources/
	cp FirstProofApp/Sources/Shaders/ShaderTypes.h dist/FirstProof.app/Contents/Resources/
	cp $(shell swift build --package-path FirstProofApp --show-bin-path)/FirstProofApp dist/FirstProof.app/Contents/MacOS/
	codesign --force --deep --sign - dist/FirstProof.app
	@echo "✅ Standalone bundle created at dist/FirstProof.app"

test:
	GIT_CONFIG_GLOBAL=/dev/null cargo test -p firstproof-core

test-all:
	GIT_CONFIG_GLOBAL=/dev/null cargo test --workspace
	GIT_CONFIG_GLOBAL=/dev/null cargo check -p firstproof-wasm --target wasm32-unknown-unknown
	cd firstproof-wasm && wasm-pack build --target web --out-dir pkg
	node firstproof-wasm/test_wasm.js

build-wasm:
	cd firstproof-wasm && wasm-pack build --target web --out-dir pkg

build-web: build-wasm
	cd web && npm run build

test-parity:
	GIT_CONFIG_GLOBAL=/dev/null cargo test -p firstproof-core --test dual_engine_synthetic_parity -- --nocapture


test-calibration:
	GIT_CONFIG_GLOBAL=/dev/null cargo run --manifest-path firstproof-core/Cargo.toml --target $(RUST_TARGET) --bin calibrate

clean:
	cargo clean
	rm -rf FirstProofBridge/Sources/FirstProofBridge/firstproof_*.swift FirstProofBridge/Sources/FirstProofCoreFFI/include/firstproof_*FFI.h FirstProofBridge/Sources/FirstProofCoreFFI/include/module.modulemap FirstProofBridge/Sources/FirstProofCoreFFI/include/libfirstproof_core.a
	rm -rf FirstProofApp/.build FirstProofBridge/.build dist/
