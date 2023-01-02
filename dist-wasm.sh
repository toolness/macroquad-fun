#! /usr/bin/env sh

./build-wasm.sh && rm -rf dist && \
  mkdir -p dist/target/wasm32-unknown-unknown/release && \
  cp target/wasm32-unknown-unknown/release/macroquad-fun.wasm dist/target/wasm32-unknown-unknown/release && \
  cp index.html dist/ && \
  # Note that this copies a bunch of stuff that's not actually needed by the game.
  cp -r media dist/ && \
  rm -rf dist/media/world/backups && \
  echo "Created WASM distribution in 'dist' directory."
