FROM rust:1.67-alpine as builder

WORKDIR /cerotis
COPY . .

RUN cargo build --release
# COPY --from=builder ../../target/release/cerotis /cerotis
# CMD ["cerotis"]
# EXPOSE 3005