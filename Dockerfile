FROM rust:1.53 as builder

WORKDIR /usr/src/korova

COPY Cargo.lock .
COPY Cargo.toml .

# First use a dummy `main.rs` for better layers caching.
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
RUN cargo build --release

# We have to update timestamps of source files.
COPY src/ src/
RUN touch src/*
# Then, finally, we can run a proper build.
RUN cargo build --release

FROM debian:buster-slim
RUN apt update && apt install -y openssl ca-certificates

COPY --from=builder /usr/src/korova/target/release/korova /usr/local/bin/korova
ENTRYPOINT ["korova"]

