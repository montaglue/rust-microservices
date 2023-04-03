FROM rust:latest as build

WORKDIR /usr/src/audit_backend/
COPY . .
RUN export CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN cargo build --release
RUN chmod +x ./setup.sh
CMD ["./setup.sh"]
