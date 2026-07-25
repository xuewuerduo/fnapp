FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY static/ static/
RUN cargo build --release

FROM alpine:latest
LABEL maintainer="fn-wol"
LABEL description="飞牛 WOL 远程唤醒工具"
COPY --from=builder /build/target/release/fn-wol /usr/local/bin/fn-wol
WORKDIR /data
VOLUME /data
EXPOSE 10101
CMD ["fn-wol"]
