FROM rust AS builder

    # Create a new empty shell project
RUN USER=root cargo new --bin telegram2photoprism && \
    # Dependencies for building openssl
    apk add pkgconfig openssl-dev musl-dev perl make
WORKDIR /telegram2photoprism

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Build the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# Copy the source code
COPY src ./src

# Build the application
RUN cargo build --release

FROM scratch
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /telegram2photoprism/target/release/telegram2photoprism /
CMD ["/telegram2photoprism"]
