FROM docker.io/library/rust:alpine3.15 as builder

RUN apk add \
  libc-dev \
  openssl-dev

COPY . .
RUN RUSTFLAGS=-Ctarget-feature=-crt-static cargo install \
  --path . \
  --root /usr/local

FROM docker.io/library/alpine:3.15

RUN apk add \
  libgcc

COPY --from=builder \
  /usr/local/bin/cloudflare-ddns-updater \
  /usr/local/bin/cloudflare-ddns-updater

RUN mkdir /config

ENTRYPOINT ["/usr/local/bin/cloudflare-ddns-updater"]
CMD ["--config", "/config/config.toml"]
