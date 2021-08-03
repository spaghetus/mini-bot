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
COPY --from=builder /app/target/release/build/torch-sys* ./target/release/build/
ENV RUSTBERT_CACHE=/cache
RUN mv $(find target -name *.so*) /usr/lib/
RUN cp /usr/lib/libgomp-*.so.1 /usr/lib/libgomp.so.1
CMD "./mini-bot"