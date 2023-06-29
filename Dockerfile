FROM rust:1.67 as builder

RUN mkdir -p /usr/kvstore/
WORKDIR /usr/kvstore
COPY . .

RUN apt-get update && \
    apt-get install -y protobuf-compiler
RUN rustup update && cargo build --release

FROM debian:bullseye-slim

RUN apt update && \
    apt -y install dnsutils && \
    apt -y install curl
RUN mkdir node

COPY --from=builder /usr/kvstore/target/release/kvstore /node
COPY --from=builder /usr/kvstore/start_node.sh /node
WORKDIR /node

CMD ["bash", "./start_node.sh"]
EXPOSE 8080 5000

# docker run -dp 8080:8080 --rm --name kvstore ermiashabtegabr/kvstore
