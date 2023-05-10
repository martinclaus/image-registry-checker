# image-registry-checker

This repository contains a web server which serves an API to check if a container image exists in a
public registry. For the lookup, [crane](https://github.com/google/go-containerregistry/blob/main/cmd/crane/doc/crane.md) is spawned as a subprocess.

Currently, the server does not support encryption or authentication with private registries.

## Build from Source

1. [Install rust](https://www.rust-lang.org/tools/install)
2. Clone this repository
3. Build from crate root
```bash
cd image-repository-checker
cargo build --release
```
4. Copy binary from target directory
```bash
cp target/release/image-registry-checker /SOME/PLACE/YOU/WANT/IT/TO/BE
```

## Build Container
Alternatively, you can also build a docker container from this repository. Just clone it and run the following command from within the repository root:
```bash
docker build -t <YOUR_IMAGE_NAME> .
```
This will take care of installing any build-time and run-time dependencies, so you do have to install rust or `crane`.

The container will expose port 8080 to accept incoming http requests. After starting the container, e.g. like this
```bash
docker run --rm -p8080:8080 <YOUR_IMAGE_NAME>
```
you can check, if the service is up by visiting http://localhost:8080/health, which should return "OK".

## Use
See help for usage information
```bash
image-registry-checker --help

This webserver serves an API to check whether a container image is present in a registry or not. Currently, it only allows to query public registries (no authentication implemented) and serves only http (no encription).

To query for the image `docker.io/nginx`, run

curl "http://<SERVER_URL>/exists?image=docker.io/nginx"

Usage: image-registry-checker [OPTIONS]

Options:
  -i, --ip <IP>
          IP adress to bind to
          
          [default: 127.0.0.1]

  -p, --port <PORT>
          Port to listen on
          
          [default: 8080]

  -c, --crane-cmd <CRANE_CMD>
          Path and name of the crane executable
          
          [env: CRANE_CMD=]
          [default: crane]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

Note that the [crane](https://github.com/google/go-containerregistry/blob/main/cmd/crane/doc/crane.md) executable needs to be installed separately.

The server serves API docs with Swagger UI at `http://<SERVER_URL>/swagger-ui` and OpenAPI docs at `http://<SERVER_URL>/api-doc.json`.


