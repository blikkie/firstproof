.PHONY: all build-core build-bridge clean test

all: build-bridge

build-core:
	cargo build --release -p lumenbox-core

build-bridge: build-core
	mkdir -p LumenBoxBridge/Sources/LumenBoxBridge
	mkdir -p LumenBoxBridge/Sources/LumenBoxCoreFFI/include
	cargo run --manifest-path lumenbox-core/Cargo.toml --features uniffi/cli --bin uniffi-bindgen -- generate \
		--library target/release/liblumenbox_core.dylib \
		--language swift \
		--out-dir LumenBoxBridge/Sources/LumenBoxBridge/ \
		--no-format \
		--metadata-no-deps
	mv LumenBoxBridge/Sources/LumenBoxBridge/lumenbox_coreFFI.h LumenBoxBridge/Sources/LumenBoxCoreFFI/include/
	mv LumenBoxBridge/Sources/LumenBoxBridge/lumenbox_coreFFI.modulemap LumenBoxBridge/Sources/LumenBoxCoreFFI/include/module.modulemap
	cp target/release/liblumenbox_core.a LumenBoxBridge/Sources/LumenBoxCoreFFI/include/
	swift build --package-path LumenBoxBridge

test:
	cargo test

clean:
	cargo clean
	rm -rf LumenBoxBridge/Sources/
