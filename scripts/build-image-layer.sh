#!/bin/sh

if [ -z "$TARGETARCH" ]; then
  :
else
  case "${TARGETARCH}" in
    "amd64")
      LINKER_NAME="x86_64-linux-gnu-gcc"
      LINKER_PACKAGE="gcc-x86-64-linux-gnu"
      BUILD_TARGET="x86_64-unknown-linux-gnu" ;;
    "arm64")
      LINKER_NAME="aarch64-linux-gnu-gcc"
      LINKER_PACKAGE="gcc-aarch64-linux-gnu"
      BUILD_TARGET="aarch64-unknown-linux-gnu" ;;
  esac
fi

tools() {
  apt-get install -y "${LINKER_PACKAGE}"
  rustup target add "${BUILD_TARGET}"
}

deps() {
  mkdir -p \
    crates/cerotis/src \
    crates/messier/src \
    crates/migration/src \
    crates/orion/src \
    crates/nebula/src \
    crates/ton/src 
  echo 'fn main() { panic!("stub"); }' |
    tee crates/cerotis/src/main.rs |
    tee crates/messier/src/main.rs
    tee crates/nebula/src/main.rs
  echo '' |
    tee crates/migration/src/lib.rs |
    tee crates/orion/src/lib.rs |
    tee crates/ton/src/lib.rs 
  
  if [ -z "$TARGETARCH" ]; then
    cargo build --locked --release
  else
    cargo build --locked --release --target "${BUILD_TARGET}"
  fi
}

apps() {
  touch -am \
    crates/cerotis/src/main.rs \
    crates/messier/src/main.rs \
    crates/migration/src/lib.rs \
    crates/orion/src/lib.rs \
    crates/nebula/src/main.rs \
    crates/ton/src/lib.rs 
  
  if [ -z "$TARGETARCH" ]; then
    cargo build --locked --release
  else
    cargo build --locked --release --target "${BUILD_TARGET}"
    mv target _target && mv _target/"${BUILD_TARGET}" target
  fi
}

if [ -z "$TARGETARCH" ]; then
  :
else
  export RUSTFLAGS="-C linker=${LINKER_NAME}"
  export PKG_CONFIG_ALLOW_CROSS="1"
  export PKG_CONFIG_PATH="/usr/lib/pkgconfig:/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/lib/x86_64-linux-gnu/pkgconfig"
fi
