###############################################################################
## Builder
###############################################################################
FROM rust:1.73 AS builder

LABEL maintainer="Lorenzo Carbonell <a.k.a. atareao> lorenzo.carbonell.cerezo@gmail.com"

ARG TARGET=x86_64-unknown-linux-musl
ENV RUST_MUSL_CROSS_TARGET=$TARGET \
    OPENSSL_LIB_DIR="/usr/lib/x86_64-linux-gnu" \
    OPENSSL_INCLUDE_DIR="/usr/include/openssl"

RUN rustup target add $TARGET && \
    apt-get update && \
    apt-get install -y \
        --no-install-recommends\
        pkg-config \
        musl-tools \
        build-essential \
        cmake \
        musl-dev \
        pkg-config \
        libssl-dev \
        && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src src

RUN cargo build --release --target $TARGET && \
    cp /app/target/$TARGET/release/aopodcast /app/aopodcast

###############################################################################
## Final image
###############################################################################
FROM alpine:3.18

RUN apk add --update --no-cache \
            su-exec~=0.2 \
            tzdata~=2023 && \
    rm -rf /var/cache/apk && \
    rm -rf /var/lib/app/lists*
# Copy the user

# Set the work dir
WORKDIR /app

COPY entrypoint.sh /app/
# Copy our build
COPY --from=builder /app/aopodcast /app/

ENTRYPOINT ["/bin/sh", "/app/entrypoint.sh"]
CMD ["/app/aopodcast"]
