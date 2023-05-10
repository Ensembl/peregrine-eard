FROM rust:1.67.1-alpine3.17 as builder

RUN apt-get update
RUN apt-get install -y curl

RUN rm -rf /app
RUN mkdir /app
COPY . /app

WORKDIR /app/compiler

RUN cargo build --release

ENV PATH=$PATH:/app/compiler/target/release/

ENTRYPOINT ["eard-compiler"]
