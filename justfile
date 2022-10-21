
build:
    cd native \
    && cargo build \
    && cp target/debug/libflowfield_native.so ../addons/tilemap_flowfields

build-release:
    cd native \
    && cargo build --release \
    && cp target/release/libflowfield_native.so ../addons/tilemap_flowfields