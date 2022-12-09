FROM rust:1.61 as builder

WORKDIR /var/www
COPY . /var/www

# cargo build rust
RUN cargo build --release --bin stock-opname-server

FROM debian:buster-slim as runtime

RUN apt-get update && apt-get install -y libssl1.1 libpq-dev ca-certificates

ENV LD_LIBRARY_PATH /usr/local/pgsql/lib

COPY --from=builder /var/www/target/release/stock-opname-server /usr/local/bin/stock-opname-server

RUN groupadd -r stock-opname && useradd -r -g stock-opname stock-opname
RUN chown -R stock-opname:stock-opname /usr/local/bin/stock-opname-server

RUN mkdir -p /var/www/storage/temp && mkdir -p /var/www/storage/logs && chown -R stock-opname:stock-opname /var/www/storage

USER stock-opname

CMD ["/usr/local/bin/stock-opname-server"]

EXPOSE 9001