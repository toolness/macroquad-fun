#! /usr/bin/env bash

# You might want to run `eval $(ssh-agent) && ssh-add` before
# running this. Be sure to run `ssh-agent -k` when finished.

./build-tarball.sh && \
  scp dist/macroquad-fun.tar.gz labs:macroquad-fun-latest.tar.gz && \
  ssh labs 'web/macroquad_fun.py update macroquad-fun-latest.tar.gz && rm macroquad-fun-latest.tar.gz' && \
  echo "Deploy successful."
