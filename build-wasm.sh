#! /usr/bin/env sh

cargo build --target wasm32-unknown-unknown --release && \
  ls -al target/wasm32-unknown-unknown/release/macroquad-fun.wasm && \
  echo "WASM build successful."
