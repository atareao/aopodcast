###############################################################################
## Backend builder
###############################################################################
FROM rust:alpine3.19 AS builder

LABEL maintainer="Lorenzo Carbonell <a.k.a. atareao> lorenzo.carbonell.cerezo@gmail.com"

RUN apk add --update --no-cache \
            autoconf \
            gcc \
            gdb \
            git \
            libdrm-dev \
            libepoxy-dev \
            make \
            mesa-dev \
            strace \
            musl-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src src

RUN cargo build --release && \
    cp /app/target/release/aopodcast /app/aopodcast

###############################################################################
## Final image
###############################################################################
FROM alpine:3.19

RUN apk add --update --no-cache \
            su-exec~=0.2 \
            tzdata~=2024 && \
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
