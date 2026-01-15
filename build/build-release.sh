#!/bin/bash

set -e

docker build -t civiewer-dev -f Dockerfile .
docker run --rm -v ../:/app -w /app civiewer-dev /bin/bash -c "cargo build --release && cp ThirdPartyNotices.txt LICENSE target/release/"