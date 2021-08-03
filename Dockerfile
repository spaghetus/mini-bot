FROM rustlang/rust:nightly AS builder
ARG GITHUB_SHA
ENV GITHUB_SHA=$GITHUB_SHA
ARG GITHUB_REPOSITORY
ENV GITHUB_REPOSITORY=$GITHUB_REPOSITORY
WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu
WORKDIR /app
RUN apt-get update && apt-get install -y libssl-dev ca-certificates curl && rm -rf /var/lib/apt/lists
RUN update-ca-certificates
COPY --from=builder /app/target/release/mini-bot ./
ENV RUSTBERT_CACHE=/cache
CMD ["./mini-bot"]