# $ git log src/main.rs | head -n 3
# commit 75e8107dee10516aea845be9545ba46352e03a4a
# Author: Martin Claus <mclaus@geomar.de>
# Date:   Mon Feb 6 22:59:50 2023 +0100

# https://go.dev/doc/devel/release
# go1.20 (released 2023-02-01)
# go1.20.1 (released 2023-02-14)

# https://releases.rs/docs/1.67.0/ (Released on: 26 January, 2023)
# https://releases.rs/docs/1.67.1/ (Released on: 9 February, 2023)

ARG GO_VERSION=1.20.0
ARG RUST_VERSION=1.67.0

# Crane Builder
FROM golang:${GO_VERSION} as crane-builder
ARG CRANE_VERSION=v0.13
RUN CGO_ENABLED=0 go install github.com/google/go-containerregistry/cmd/crane@${CRANE_VERSION}

# App Development Env
FROM rust:${RUST_VERSION}
COPY --from=crane-builder /go/bin/crane /bin/crane

