# Cloudflare DDNS updater [![CI](https://github.com/DanNixon/cloudflare-ddns-updater/actions/workflows/ci.yml/badge.svg)](https://github.com/DanNixon/cloudflare-ddns-updater/actions/workflows/ci.yml)

Tool to keep Cloudflare DNS records up to date with a dynamic/residential IP address.

There may be several other tools that do similar things, but this one is mine.
As such it is very opinionated in several ways:

- you must be using Cloudflare
- you must only care about IPv4
- you must receive notifications via Matrix

TL;DR: probably don't use this.

## Configuration

Basic example below:

```toml
[cloudflare]
token = "super_secret"

[[cloudflare.records]]
zone_id = "id"
name = "something.dan-nixon.com"

[[cloudflare.records]]
zone_id = "id"
name = "something-else.dan-nixon.com"

[matrix]
username = "@someone:somewhere.org"
password = "super_secret"
room_id = "!someroom:somewhere.org"
verbose = false
```

The Cloudflare API token should have only the DNS Edit permission for only the zones you want to update.
