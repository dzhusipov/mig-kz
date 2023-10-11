FROM rust:latest as build
WORKDIR /app
COPY . .

RUN rustup target add aarch64-unknown-linux-musl
RUN rustup toolchain install stable-aarch64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools
RUN cargo build --release --target aarch64-unknown-linux-musl

FROM alpine:latest
WORKDIR /app
COPY --from=build /app/target/aarch64-unknown-linux-musl/release/mig-kz /app/mig-kz
COPY --from=build /app/config /app/config
COPY --from=build /app/.env_docker /app/.env
CMD ["/app/mig-kz"]