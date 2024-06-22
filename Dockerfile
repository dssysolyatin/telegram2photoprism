FROM rust:latest

# Create a new empty shell project
RUN USER=root cargo new --bin telegram2photoprism
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

# Set the startup command
CMD ["./target/release/telegram2photoprism"]
