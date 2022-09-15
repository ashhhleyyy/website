# syntax = docker/dockerfile:1.4

FROM debian:11.3-slim

RUN set -eux; \
	export DEBIAN_FRONTEND=noninteractive; \
	apt update; \
	apt install --yes --no-install-recommends bind9-dnsutils iputils-ping iproute2 curl ca-certificates htop; \
	apt clean autoclean; \
	apt autoremove --yes; \
	rm -rf /var/lib/{apt,dpkg,cache,log}/; \
	echo "Installed base utils!"

WORKDIR /app

COPY target/release/aarch64-unknown-linux-gnu/website ./website
CMD ["./website"]
