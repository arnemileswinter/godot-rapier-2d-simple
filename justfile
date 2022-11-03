addon_dir_name := "rapier-2d-simple"
library_file_ext := if os_family() == "windows" {"dll"} else {"so"}
library_file := "godot_rapier2d_simple." + library_file_ext
library_target_file := "library-" + os() +"-"+arch()+"."+library_file_ext

build:
    cd native \
    && cargo build \
    && cargo test \
    && cp target/debug/{{library_file}} ../addons/{{addon_dir_name}}/bin/{{library_target_file}}

build-release:
    cd native \
    && cargo build --release \
    && cargo test \
    && cp target/release/{{library_file}} ../addons/{{addon_dir_name}}/bin/{{library_target_file}}

fmt:
    cd native \
    && cargo fmt

clippy:
    cd native \
    && cargo clippy --fix --allow-dirty
