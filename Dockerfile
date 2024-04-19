ARG GO_VERSION=1.22.2
ARG RUST_VERSION=1.77.2

# Crane Builder
FROM golang:${GO_VERSION} as crane-builder
ENV CRANE_VERSION=v0.19.1
RUN CGO_ENABLED=0 go install github.com/google/go-containerregistry/cmd/crane@${CRANE_VERSION}

# Rust builder
FROM rust:${RUST_VERSION} as rust-builder
WORKDIR /app
ADD image-registry-checker ./
RUN cargo build --release

# Actual container image
FROM gcr.io/distroless/cc-debian12
COPY --from=crane-builder /go/bin/crane /
COPY --from=rust-builder /app/target/release/image-registry-checker /
EXPOSE 80
ENTRYPOINT ["./image-registry-checker"]
CMD ["--port=8080", "--ip=0.0.0.0", "--crane-cmd=/crane"]
