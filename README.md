# middle-sock

A software for fowarding and dynamic-transforming DHCP packets in containers, using network namespace and an unix domain socket or raw socket (UDP) between a DHCP server.

# Usage
## Raw command

```sh
middle-sock -c "<DHCP server start command>"
```

## Run with Docker

```sh
cat /proc/net/route > <file> && docker run --cap-add SYS_ADMIN --cap-add NET_ADMIN --security-opt apparmor=unconfined --security-opt seccomp=unconfined -v <file>:/mnt/route:ro -e SERVER_HOST=<host_ip> -p 67:67/udp --name <container_name> -itd middle-sock
```

(I would update this with examples.)

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
