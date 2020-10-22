FROM rust:latest

WORKDIR /usr/src/kbot
COPY ./ .

RUN cargo install --path .

CMD ["kbot"]