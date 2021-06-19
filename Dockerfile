FROM rust:1.53 as builder

WORKDIR /usr/src/korova
# TODO: copy only files needed for a build
COPY . .
RUN cargo install --path .

FROM debian:buster-slim

COPY --from=builder /usr/local/cargo/bin/korova /usr/local/bin/korova
ENTRYPOINT ["korova"]

