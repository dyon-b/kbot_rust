#FROM rust:latest as builder
FROM rustlang/rust:nightly as builder
WORKDIR /usr/src/kbot
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update \
      && apt-get install -y --no-install-recommends \
        libssl1.1 ca-certificates \
      && apt-get clean \
      && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/kbot /usr/local/bin/kbot
CMD ["kbot"]
