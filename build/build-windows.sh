#!/bin/bash

set -e

# Build the docker image
docker build -t civiewer-windows -f Dockerfile.windows .

# Run the build and gather dependencies
# -v ../:/app maps the parent directory (project root) to /app in the container
docker run --rm -v "$(pwd)/..":/app -w /app civiewer-windows \
    /bin/bash -c "cargo build --target x86_64-pc-windows-gnu --release && python3 build/gather_deps.py target/x86_64-pc-windows-gnu/release/civiewer.exe target/x86_64-pc-windows-gnu/release/ && cp ThirdPartyNotices.txt LICENSE target/x86_64-pc-windows-gnu/release/"
