FROM rust:1.67 as builder
WORKDIR /usr/kvstore
COPY . .
RUN apt-get update && \
    apt-get install -y protobuf-compiler 
RUN rustup update && cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /usr/kvstore/target/release/kvstore /usr/local/bin/kvstore
EXPOSE 8080/tcp
CMD ["kvstore"]

# docker run -dp 8080:8080 --rm --name kvstore ermiashabtegabr/kvstore
