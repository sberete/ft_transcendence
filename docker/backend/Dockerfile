FROM rust:1-slim-bookworm AS builder
WORKDIR /app

# Avoids OOM on small Docker VMs (parallel compilation = RAM spike)
ENV CARGO_BUILD_JOBS=2

# Caches dependencies separately from the source code
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/ft_transcendence /usr/local/bin/ft_transcendence

EXPOSE 8080
CMD ["ft_transcendence"]
