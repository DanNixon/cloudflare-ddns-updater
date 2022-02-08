FROM docker.io/library/rust:alpine3.15 as builder

RUN apk add \
  libc-dev \
  openssl-dev

COPY . .
RUN cargo install \
  --path . \
  --root /usr/local

FROM docker.io/library/alpine:3.15

COPY --from=builder \
  /usr/local/bin/cloudflare-ddns-updater \
  /usr/local/bin/cloudflare-ddns-updater

RUN mkdir /config
WORKDIR /config

ENTRYPOINT ["/usr/local/bin/cloudflare-ddns-updater"]
