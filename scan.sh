#!/bin/sh

CRANE_VERSION=v0.13.0

# $ sh scan.sh image-registry-checker:latest

if [ ! -d trivy ]; then
 wget --quiet https://github.com/aquasecurity/trivy/releases/download/v0.50.1/trivy_0.50.1_Linux-64bit.tar.gz
 mkdir -p trivy && tar xf trivy_0.50.1_Linux-64bit.tar.gz -C trivy
fi

./trivy/trivy image --severity MEDIUM,HIGH,CRITICAL "$1"
./trivy/trivy repo --severity MEDIUM,HIGH,CRITICAL image-registry-checker/
./trivy/trivy repo --tag=${CRANE_VERSION} --severity MEDIUM,HIGH,CRITICAL github.com/google/go-containerregistry

