# Use an official Rust image as the base
FROM rust:latest

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy the source code into the container
COPY . .

# Set the AUTH_TOKEN environment variable (use a hardcoded value or pass it when running)
ENV AUTH_TOKEN="DEFAULT_AUTH_TOKEN"

# Build the Rust project
RUN cargo build --release

# Expose the port that the server listens on
EXPOSE 7878

# Run the server
CMD ["cargo", "run", "--release"]