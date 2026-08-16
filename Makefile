.PHONY: all build-core build-bridge clean test run

ARCH := $(shell uname -m)
ifeq ($(ARCH),arm64)
	RUST_TARGET := aarch64-apple-darwin
else
	RUST_TARGET := x86_64-apple-darwin
endif

all: build-bridge

build-core:
	GIT_CONFIG_GLOBAL=/dev/null cargo build --release --target $(RUST_TARGET) -p lumenbox-core

build-bridge: build-core
	mkdir -p LumenBoxBridge/Sources/LumenBoxBridge
	mkdir -p LumenBoxBridge/Sources/LumenBoxCoreFFI/include
	GIT_CONFIG_GLOBAL=/dev/null cargo run --manifest-path lumenbox-core/Cargo.toml --target $(RUST_TARGET) --features uniffi/cli --bin uniffi-bindgen -- generate \
		--library target/$(RUST_TARGET)/release/liblumenbox_core.dylib \
		--language swift \
		--out-dir LumenBoxBridge/Sources/LumenBoxBridge/ \
		--no-format \
		--metadata-no-deps
	mv LumenBoxBridge/Sources/LumenBoxBridge/lumenbox_coreFFI.h LumenBoxBridge/Sources/LumenBoxCoreFFI/include/
	mv LumenBoxBridge/Sources/LumenBoxBridge/lumenbox_coreFFI.modulemap LumenBoxBridge/Sources/LumenBoxCoreFFI/include/module.modulemap
	cp target/$(RUST_TARGET)/release/liblumenbox_core.a LumenBoxBridge/Sources/LumenBoxCoreFFI/include/
	swift build --package-path LumenBoxBridge

build-app: build-bridge
	swift build --package-path LumenBoxApp

run: build-bridge
	swift run --package-path LumenBoxApp

test:
	GIT_CONFIG_GLOBAL=/dev/null cargo test --target $(RUST_TARGET) -p lumenbox-core

clean:
	cargo clean
	rm -rf LumenBoxBridge/Sources/
	rm -rf LumenBoxApp/.build LumenBoxBridge/.build
