FROM rust:1.67.1-alpine as builder

RUN apk add --no-cache musl-dev

RUN rm -rf /app
RUN mkdir /app
COPY . /app

WORKDIR /app/compiler

RUN cargo build --release

FROM alpine:3.16 as run

RUN mkdir /app

COPY --from=builder /app/compiler/target/release/eard-compiler /app

ENV PATH=$PATH:/app