# Build Stage
FROM ghcr.io/kratos2377/base:latest AS builder
FROM debian:12 AS debian

# Bundle Stage
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /home/rust/src/target/release/cerotis ./
COPY --from=debian /usr/bin/uname /usr/bin/uname

EXPOSE 3005
USER nonroot
CMD ["./cerotis"]