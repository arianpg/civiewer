#!/bin/bash

set -e

docker build -t civiewer-build-windows -f $(pwd)/build/Dockerfile.windows .
docker run --rm -v $(pwd)/:/app -w /app civiewer-build-windows /bin/bash -c "cargo build --target x86_64-pc-windows-gnu --release && \
python3 build/gather_deps.py target/x86_64-pc-windows-gnu/release/civiewer.exe target/x86_64-pc-windows-gnu/release/ && \
rm -rf target/x86_64-pc-windows-gnu/release/CIViewer && \
mkdir -p target/x86_64-pc-windows-gnu/release/CIViewer && \
cp target/x86_64-pc-windows-gnu/release/civiewer.exe target/x86_64-pc-windows-gnu/release/CIViewer && \
cp target/x86_64-pc-windows-gnu/release/*.dll target/x86_64-pc-windows-gnu/release/CIViewer && \
cp LICENSE target/x86_64-pc-windows-gnu/release/CIViewer && \
cp ThirdPartyNotices.txt target/x86_64-pc-windows-gnu/release/CIViewer && \
cd target/x86_64-pc-windows-gnu/release/ && \
zip -r CIViewer-windows.zip CIViewer"
