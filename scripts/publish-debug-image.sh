#!/bin/bash
if [ "$#" -eq 0 ]; then
    echo "No arguments provided"
    echo "Usage: scripts/build-current-arch.sh 20230826-1"
    exit 1
fi

echo "[profile.release]" >> Cargo.toml
echo "debug = true" >> Cargo.toml

TAG=$1-debug
echo "Building images, will tag for ghcr.io with $TAG-debug!"
docker build -t ghcr.io/kratos2377/base:latest -f Dockerfile.useCurrentArch .
docker build -t ghcr.io/kratos2377/cerotis:$TAG - < crates/cerotis/Dockerfile
docker build -t ghcr.io/kratos2377/messier:$TAG - < crates/messier/Dockerfile
docker build -t ghcr.io/kratos2377/nebula:$TAG - < crates/nebula/Dockerfile

git restore Cargo.toml