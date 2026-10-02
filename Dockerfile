# ---- Frontend build -------------------------------------------------------
FROM node:24-bookworm-slim AS frontend
WORKDIR /app
COPY package.json package-lock.json .npmrc ./
RUN npm ci --no-audit --no-fund
COPY . .
RUN npm run build

# ---- Server build ---------------------------------------------------------
FROM rust:1-bookworm AS server
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates crates
RUN cargo build --release --locked -p vrcx-0-server --bin vrcx-0-server

# ---- Runtime --------------------------------------------------------------
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=server /app/target/release/vrcx-0-server /usr/local/bin/vrcx-0-server
COPY --from=frontend /app/dist /app/dist

ENV VRCX_CLOUD_DATA_DIR=/data \
    VRCX_CLOUD_DIST_DIR=/app/dist
VOLUME /data
EXPOSE 8800
ENTRYPOINT ["/usr/local/bin/vrcx-0-server"]
