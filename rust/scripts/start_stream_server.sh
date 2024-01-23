#!/usr/bin/env bash
set -e

echo "Stream can be accessed (read as well as write) via any of the following mechanisms:"
echo "  Browser:    http://localhost:8889/pixelflut"
echo "  RTSP:       rtsp://localhost:8554/pixelflut"
echo "  RTMP:       rtmp://localhost:1935/pixelflut"
echo
echo

exec docker run \
  -it \
  --net=host \
  -e MTX_RTSP=yes \
  -e MTX_WEBRTC=yes \
  -e MTX_RTMP=yes \
  -e MTX_HLS=no \
  -e MTX_SRT=no \
  docker.io/bluenviron/mediamtx:latest-ffmpeg
