#!/bin/bash

set -e
docker build -t civiewer-build-linux -f $(pwd)/build/Dockerfile.linux .
docker run --rm -v $(pwd):/app -w /app civiewer-build-linux /bin/bash -c "cargo build --release && \
cargo deb && \
cargo generate-rpm && \
rm -rf target/release/CIViewer && \
mkdir -p target/release/CIViewer && \
cp target/release/civiewer target/release/CIViewer && \
cp LICENSE target/release/CIViewer && \
cp ThirdPartyNotices.txt target/release/CIViewer && \
cp assets/civiewer.desktop target/release/CIViewer && \
cp assets/civiewer.svg target/release/CIViewer && \
cp target/debian/*.deb target/release/ && \
cp target/generate-rpm/*.rpm target/release/ && \
cd target/release && \
zip -r CIViewer-linux.zip CIViewer"
