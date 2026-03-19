#!/bin/bash

set -e

docker build -t civiewer-build-linux -f $(pwd)/build/Dockerfile.linux .
docker run --rm -v $(pwd)/:/app -w /app civiewer-build-linux cargo build

mkdir -p ~/.local/share/applications
cp "$(pwd)/assets/com.arianpg.civiewer.desktop" ~/.local/share/applications/
update-desktop-database ~/.local/share/applications/ 2>/dev/null || true
