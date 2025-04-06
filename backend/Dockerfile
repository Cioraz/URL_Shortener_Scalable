# Use the Rust base image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /app

# Copy the manifest and lock file first for caching dependencies
COPY Cargo.toml Cargo.lock ./

# Copy the full source code
COPY src ./src

# Optionally expose the port your app listens on
EXPOSE 8000

# Run the application using cargo
CMD ["cargo", "run", "--release"]
