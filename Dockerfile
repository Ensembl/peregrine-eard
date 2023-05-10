FROM rust:1.67 as builder

RUN apt update

RUN rm -rf /app
RUN mkdir /app
COPY . /app

WORKDIR /app/compiler

RUN cargo build --release

ENV PATH=$PATH:/app/compiler/target/release/

ENTRYPOINT ["eard-compiler"]
