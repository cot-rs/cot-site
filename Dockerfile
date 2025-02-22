FROM docker.io/library/rust:1.85 AS builder
WORKDIR /usr/src/cot-site
COPY . .
RUN cargo install --path . --locked

FROM docker.io/library/debian:12-slim
COPY --from=builder /usr/local/cargo/bin/cot-site /usr/local/bin/cot-site
RUN mkdir /app
COPY entrypoint.sh /app

RUN apt-get update &&  \
    apt-get install -y --no-install-recommends tini=0.19.* && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/app/entrypoint.sh", "-l", "0.0.0.0:8000"]
