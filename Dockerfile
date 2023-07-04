FROM rust:1.70 AS build
WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN rm -rf /var/lib/apt/lists/*
COPY --from=build /usr/local/cargo/bin/cidar /usr/local/bin/cidar

CMD ["cidar"]