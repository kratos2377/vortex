FROM rust:1.67-alpine as builder

# RUN apt-get update && \
#     apt-get install -y ca-certificates && \
#     apt-get clean
WORKDIR /cerotis
COPY . .


RUN cargo build --locked --release
# COPY --from=builder ../../target/release/cerotis /cerotis
# CMD ["cerotis"]
# EXPOSE 3005