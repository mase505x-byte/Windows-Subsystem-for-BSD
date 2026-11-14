#!/bin/bash
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
cargo +nightly build -Z build-std=core,std,alloc,panic_abort --target x86_64-wsb-freebsd.json --release
