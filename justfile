host_target := `rustc -vV | grep host | awk '{print $2}'`

install:
    cargo build --release
    mkdir -p ~/.config/zellij/plugins
    cp target/wasm32-wasip1/release/zellij-fingers.wasm ~/.config/zellij/plugins/

lint:
    cargo clippy --target {{host_target}} -- -D warnings

test:
    cargo test --target {{host_target}}

fmt:
    cargo fmt
