# Rust Compiller
FROM rust:latest as builder

WORKDIR /app

COPY Cargo.toml Cargo.toml
COPY src/ src/

RUN cargo build --release

# fronten builder
FROM node:latest as frontend-builder

WORKDIR /app

COPY frontend/package.json .

RUN npm install

COPY frontend .


RUN npm run build

# Runtime
FROM debian:stable-slim

WORKDIR /app

COPY --from=builder /app/target/release/clavis /app/clavis
COPY --from=frontend-builder /app/dist /app/static

EXPOSE 3000

ENV RUST_LOG=info
ENV LDAP_URL=ldap://ldap.example.com
ENV LDAP_BASE_DN=dc=example,dc=com
ENV LDAP_SERVICE_USER=cn=admin,dc=example,dc=com
ENV LDAP_SERVICE_PASSWORD=admin
ENV BIND_ADDRESS=0.0.0.0:4000
ENV PRODUCTION=true

CMD ["/app/clavis"]
