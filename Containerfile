FROM docker.io/library/rust:alpine3.18 as builder

RUN apk add \
  libc-dev \
  openssl-dev

ADD Cargo.toml Cargo.lock .
ADD src ./src
RUN RUSTFLAGS=-Ctarget-feature=-crt-static cargo install \
  --path . \
  --root /usr/local

FROM docker.io/library/alpine:3.18

RUN apk add \
  tini \
  libgcc

COPY --from=builder \
  /usr/local/bin/cloudflare-ddns-updater \
  /usr/local/bin/cloudflare-ddns-updater

RUN mkdir /config

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["/usr/local/bin/cloudflare-ddns-updater", "--config", "/config/config.toml"]
