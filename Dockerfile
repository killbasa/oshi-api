FROM rust:1.92.0-slim-bookworm AS builder

WORKDIR /temp

RUN apt-get update && \
	apt-get install -y libssl-dev pkg-config && \
	rm -rf /var/lib/apt/lists/*

COPY ./src ./src/
COPY ./Cargo.toml ./Cargo.lock ./

RUN cargo build --locked --release

FROM debian:13.1-slim

WORKDIR /etc/oshi-api

RUN apt-get update -y && \
	apt-get install -y openssl ca-certificates && \
	update-ca-certificates && \
	rm -rf /var/lib/apt/lists/* && \
	mkdir -p /etc/oshi-api/data

ENV HOST=0.0.0.0

COPY --from=builder /temp/target/release/oshi-api /etc/oshi-api/app

EXPOSE 3000

ENTRYPOINT [ "/etc/oshi-api/app" ]
