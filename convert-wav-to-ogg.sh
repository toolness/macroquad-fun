#! /usr/bin/env sh

cd media/audio

for f in *.wav; do
  ffmpeg -i "$f" -acodec libvorbis "${f%.wav}.ogg"
done
