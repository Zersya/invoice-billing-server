FROM rust:1.61 as builder

WORKDIR /var/www
COPY . /var/www

# cargo build rust
RUN cargo build --release --bin invoice-billing-server

FROM debian:buster-slim as runtime

RUN apt-get update && apt-get install -y libssl1.1 libpq-dev ca-certificates

ENV LD_LIBRARY_PATH /usr/local/pgsql/lib

COPY --from=builder /var/www/target/release/invoice-billing-server /usr/local/bin/invoice-billing-server

RUN groupadd -r inving && useradd -r -g inving inving
RUN chown -R inving:inving /usr/local/bin/invoice-billing-server

RUN mkdir -p /var/www/storage/temp && mkdir -p /var/www/storage/logs && chown -R inving:inving /var/www/storage

USER inving

CMD ["/usr/local/bin/invoice-billing-server"]

EXPOSE 9000