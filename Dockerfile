FROM rust:1.70 AS build
WORKDIR /rust/src/cidar
COPY . .
RUN cargo install --path .

FROM alpine:latest
COPY --from=builder /usr/local/cargo/bin/cidar /usr/local/bin/cidar
CMD ["cidar"]