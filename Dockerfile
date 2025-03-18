FROM --platform="${BUILDPLATFORM}" rust:1.77.2-slim-bookworm
USER 0:0
WORKDIR /home/rust/src

ARG TARGETARCH

# Install build requirements
RUN dpkg --add-architecture "${TARGETARCH}"
RUN apt-get update && \
    apt-get install -y \
    make \
    pkg-config \
    libssl-dev:"${TARGETARCH}"
COPY scripts/build-image-layer.sh /tmp/
RUN sh /tmp/build-image-layer.sh tools

# Build all dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates/messier/Cargo.toml ./crates/messier/
COPY crates/cerotis/Cargo.toml ./crates/cerotis/
COPY crates/nebula/Cargo.toml ./crates/nebula/
COPY crates/nova/Cargo.toml ./crates/nova/
COPY crates/migration/Cargo.toml ./crates/migration/
COPY crates/orion/Cargo.toml ./crates/orion/
COPY crates/ton/Cargo.toml ./crates/ton/
COPY crates/common-tracing/Cargo.toml ./crates/common-tracing/
RUN sh /tmp/build-image-layer.sh deps

# Build all apps


COPY crates ./crates

RUN sh /tmp/build-image-layer.sh apps