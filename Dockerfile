FROM rust:alpine as builder

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
COPY resources resources

# Build the application
RUN cargo build --release

FROM alpine
COPY --from=builder /telegram2photoprism/target/release/telegram2photoprism /telegram2photoprism
CMD ["/telegram2photoprism"]
