FROM docker.io/library/rust:1.84 AS builder
WORKDIR /usr/src/cot-site
COPY . .
RUN cargo install --path . --locked

FROM docker.io/library/debian:12-slim
COPY --from=builder /usr/local/cargo/bin/cot-site /usr/local/bin/cot-site
RUN mkdir /app
COPY entrypoint.sh /app

CMD ["/app/entrypoint.sh", "-l", "0.0.0.0:8000"]
