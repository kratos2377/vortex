# Build Stage
FROM --platform="${BUILDPLATFORM}" rust:1.67.0-alpine
USER 0:0
WORKDIR /home/rust/src

ARG TARGETARCH

# Install build requirements
RUN dpkg --add-architecture "${TARGETARCH}"
RUN apt-get update && apt-get install -y make pkg-config libssl-dev:"${TARGETARCH}"
COPY scripts/build-image-layer.sh /tmp/
RUN sh /tmp/build-image-layer.sh tools

# Build all dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates/cerotis/Cargo.toml ./crates/cerotis/
COPY crates/messier/Cargo.toml ./crates/messier/
COPY crates/migration/Cargo.toml ./crates/migration/
COPY crates/orion/Cargo.toml ./crates/orion/
COPY crates/saggitarius/Cargo.toml ./crates/saggitarius/
COPY crates/ton/Cargo.toml ./crates/ton/
RUN sh /tmp/build-image-layer.sh deps

# Build all apps
COPY crates ./crates
RUN sh /tmp/build-image-layer.sh apps
