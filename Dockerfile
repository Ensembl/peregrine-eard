FROM rust:1.67 as builder

RUN apt-get update
RUN apt-get install -y curl

RUN rm -rf /usr/src/app
RUN mkdir /usr/src/app
COPY . /usr/src/app

WORKDIR /usr/src/app/compiler

RUN cargo build --release
