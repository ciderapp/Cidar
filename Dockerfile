FROM debian:bullseye-slim
FROM rust:1.70 AS build

# Work directory
WORKDIR /app

# Build Phase
ENTRYPOINT sh -c "if [ -d .git ]; then git pull; else git clone https://github.com/ciderapp/Cidar.git .; fi && cargo run --release || true"
