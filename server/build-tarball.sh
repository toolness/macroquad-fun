#! /usr/bin/env bash

cargo build --release && \
  rm -rf dist && \
  mkdir -p dist/macroquad-fun && \
  cd dist && \
  cp ../target/release/macroquad-fun macroquad-fun && \
  tar -zcvf macroquad-fun.tar.gz macroquad-fun && \
  rm -rf macroquad-fun && \
  echo "Wrote dist/macroquad-fun.tar.gz."
