# Source code

This folder contains the actual `image-registry-checker` code.
See below for how to run the specified unit tests.

## Start development environment

```
$ docker build -t rustdev:latest .
$ docker run -it --rm -v $PWD:/app -w /app rustdev:latest bash
```

## Run specified unit tests

```
$ cargo test
```

