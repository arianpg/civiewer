#!/bin/bash

set -e
docker build -t civiewer-build-linux -f $(pwd)/build/Dockerfile.linux .
docker run --rm -v $(pwd):/app -w /app civiewer-build-linux /bin/bash -c "cargo build --release && \
rm -rf target/release/CIViewer && \
mkdir -p target/release/CIViewer && \
cp target/release/civiewer target/release/CIViewer && \
cp LICENSE target/release/CIViewer && \
cp ThirdPartyNotices.txt target/release/CIViewer && \
cd target/release && \
zip -r CIViewer-linux.zip CIViewer"
