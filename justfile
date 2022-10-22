
build:
    cd native \
    && cargo build \
    && cp target/debug/libtilemap_flowfields_native.so ../addons/tilemap_flowfields/libflowfield_native.so

build-release:
    cd native \
    && cargo build --release \
    && cp target/release/libtilemap_flowfields_native.so ../addons/tilemap_flowfields/libflowfield_native.so

clippy:
    cd native \
    && cargo clippy --fix --allow-dirty
