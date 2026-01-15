#!/bin/bash

set -e

docker build -t civiewer-build-linux -f $(pwd)/build/Dockerfile.linux .
docker run --rm -v $(pwd)/:/app -w /app civiewer-build-linux cargo check