#!/bin/bash

set -e

docker build -t civiewer-build-windows -f $(pwd)/build/Dockerfile.windows .
docker run --rm -v $(pwd)/:/app -w /app civiewer-build-windows /bin/bash -c "cargo build --target x86_64-pc-windows-gnu --release && \
python3 build/gather_deps.py target/x86_64-pc-windows-gnu/release/civiewer.exe target/x86_64-pc-windows-gnu/release/ && \
rm -f target/x86_64-pc-windows-gnu/release/CIViewer-windows.zip && \
rm -rf target/x86_64-pc-windows-gnu/release/CIViewer && \
mkdir -p target/x86_64-pc-windows-gnu/release/CIViewer && \
cp target/x86_64-pc-windows-gnu/release/civiewer.exe target/x86_64-pc-windows-gnu/release/CIViewer && \
cp target/x86_64-pc-windows-gnu/release/*.dll target/x86_64-pc-windows-gnu/release/CIViewer && \
cp LICENSE target/x86_64-pc-windows-gnu/release/CIViewer && \
cp ThirdPartyNotices.txt target/x86_64-pc-windows-gnu/release/CIViewer && \
mkdir -p target/x86_64-pc-windows-gnu/release/CIViewer/share/glib-2.0/schemas && \
cp /usr/x86_64-w64-mingw32/sys-root/mingw/share/glib-2.0/schemas/* target/x86_64-pc-windows-gnu/release/CIViewer/share/glib-2.0/schemas/ && \
glib-compile-schemas target/x86_64-pc-windows-gnu/release/CIViewer/share/glib-2.0/schemas/ && \
cd target/x86_64-pc-windows-gnu/release/ && \
zip -r CIViewer-windows.zip CIViewer"
