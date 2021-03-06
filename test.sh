#!/bin/bash

cargo test --all --no-default-features
cargo test --all --all-features


# Install wasm-pack if it isn't installed yet
if ! type wasm-pack > /dev/null; then
    cargo install wasm-pack
fi

wasm-pack test --firefox --headless
