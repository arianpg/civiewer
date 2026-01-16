#!/bin/bash
set -e

# Create release directory
mkdir -p target/release

# Build the docker image
echo "Building Docker image..."
docker build -t civiewer-build-arch -f build/Dockerfile.arch .

# Run the build container
echo "Running build..."
docker run --rm \
    -v $(pwd):/src \
    -v $(pwd)/target/release:/out \
    civiewer-build-arch \
    /bin/bash -c "
        set -e
        shopt -s extglob
        # Copy source files excluding build artifacts and version control
        cp -r /src/!(target|.git|.github) .
        
        # Build package
        makepkg -f
        
        # Copy package to output
        sudo cp *.pkg.tar.zst /out/
        echo 'Package created successfully in target/release'
    "
