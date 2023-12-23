# middle-sock

A software for fowarding and dynamic-transforming DHCP packets in containers, using an unix domain socket or raw socket between a DHCP server.

# Usage

```sh
middle-sock -c "<DHCP server start command>"
```

# Build

MSRV (Minimum Supported rustc Version): 1.74.1 (only tested in this version)

```sh
cargo build --release
```

## Docker

Build:

```sh
docker build -t middle-sock:<your_tag> .
```
