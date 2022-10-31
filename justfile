
build:
    cd native \
    && cargo build

build-release:
    cd native \
    && cargo build --release

clippy:
    cd native \
    && cargo clippy --fix --allow-dirty
