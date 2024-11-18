# Stage 1: Build the application
FROM rust:latest as builder

# Set the working directory
WORKDIR /usr/src/app

# Copy Cargo files to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Dummy build to cache dependencies (build with a minimal main file)
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true

# Copy the source code and build
COPY . .
RUN cargo build --release

# Stage 2: Create the final image
FROM debian:buster-slim

# Set the working directory
WORKDIR /usr/src/app

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/app/target/release/flashdb /usr/local/bin/flashdb

# Set the AUTH_TOKEN environment variable
ENV AUTH_TOKEN="DEFAULT_AUTH_TOKEN"
ENV RUST_LOG=info

# Expose the server port
EXPOSE 7878

# Run the application
CMD ["nanodb"]