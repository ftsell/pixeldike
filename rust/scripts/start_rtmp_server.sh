#!/usr/bin/env bash
set -e

echo "Stream can be viewed via any of the following mechanisms:"
echo "  Browser:                      http://localhost:8889/pixelflut"
echo "  VLC:                          rtsp://localhost:8554/pixelflut"
echo
echo

exec docker run \
  -it \
  --net=host \
  -e MTX_RTSP=yes \
  -e MTX_RTMP=yes \
  -e MTX_WEBRTC=yes \
  -e MTX_HLS=no \
  -e MTX_SRT=no \
  bluenviron/mediamtx:latest
