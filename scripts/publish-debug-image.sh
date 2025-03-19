#!/usr/bin/env bash

# fail asap
set -e

# Check if an argument was provided
if [ $# -eq 0 ]; then
    echo "No arguments provided"
    echo "Usage: scripts/publish-debug-image.sh 20230826-1 true"
    echo ""
    echo "Last argument specifies whether we should have a debug build as opposed to release build."
    exit 1
fi

DEBUG=$2
if [ "$DEBUG" = "true" ]; then
  echo "[profile.release]" >> Cargo.toml
  echo "debug = true" >> Cargo.toml
fi

TAG=$1-version
echo "Building images, will tag for ghcr.io with tag $TAG!"
docker build -t ghcr.io/kratos2377/base:latest -f Dockerfile.useCurrentArch .
docker build -t ghcr.io/kratos2377/messier:$TAG - < crates/messier/Dockerfile
docker build -t ghcr.io/kratos2377/cerotis:$TAG - < crates/cerotis/Dockerfile
docker build -t ghcr.io/kratos2377/nebula:$TAG - < crates/nebula/Dockerfile
docker build -t ghcr.io/kratos2377/nova:$TAG - < crates/nova/Dockerfile

if [ "$DEBUG" = "true" ]; then
  git restore Cargo.toml
fi

docker push ghcr.io/kratos2377/messier:$TAG
docker push ghcr.io/kratos2377/cerotis:$TAG
docker push ghcr.io/kratos2377/nebula:$TAG
docker push ghcr.io/kratos2377/nova:$TAG