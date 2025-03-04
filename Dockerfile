# Using official rust base image
FROM rust:alpine3.21

# Set the application directory
WORKDIR /app

# Install musl-tools to make many crates compile successfully
RUN apk add --no-cache musl-dev curl

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | sh

# Install cargo-watch
RUN cargo binstall cargo-watch

# Copy the files to the Docker image
COPY ./ ./

RUN cargo build --release

RUN cp target/release/cqrl /usr/local/bin/cqrl

CMD ["cqrl"]