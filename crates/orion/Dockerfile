FROM rust:1.67-alpine as builder

WORKDIR /orion
COPY . .


RUN rustup default stable
RUN apk update && apk add --no-cache libpq musl-dev pkgconfig openssl-dev gcc
RUN cargo build --release
# COPY --from=builder ../../target/release/cerotis /cerotis
# CMD ["cerotis"]
# EXPOSE 3005