#!/bin/bash

set -e

docker run --rm -v $(pwd):/app -w /app civiewer-dev cargo build