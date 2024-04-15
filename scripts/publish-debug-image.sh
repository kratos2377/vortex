#!/bin/bash

# Check if an argument was provided
if [ "$#" -eq 0 ]; then
    echo "No arguments provided"
    echo "Usage: scripts/build-current-arch.sh 20230826-1"
    exit 1
fi

echo "[profile.release]" >> Cargo.toml
echo "debug = true" >> Cargo.toml

TAG=$1-debug
echo "Building images, will tag for ghcr.io with $TAG-debug!"
docker build -t ghcr.io/vortex/base:latest -f Dockerfile.useCurrentArch .
docker build -t ghcr.io/vortex/cerotis:$TAG - < crates/cerotis/Dockerfile
docker build -t ghcr.io/vortex/messier:$TAG - < crates/messier/Dockerfile
docker build -t ghcr.io/vortex/saggitarius:$TAG - < crates/saggitarius/Dockerfile

git restore Cargo.toml
