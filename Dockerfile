FROM rust:1.58.1-slim-buster as builder

WORKDIR /usr/src/xzibot

# build dependencies
RUN USER=root cargo init
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

# cleanup
RUN rm src/*.rs
RUN rm target/release/deps/xzibot*

# build app
COPY src src
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder /usr/src/xzibot/target/release/xzibot /bin/xzibot
CMD xzibot