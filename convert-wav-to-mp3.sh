#! /usr/bin/env sh

cd media/audio

for f in *.wav; do
  ffmpeg -i "$f" -b:a 192k "${f%.wav}.mp3"
done
