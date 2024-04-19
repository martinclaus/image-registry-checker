#!/bin/sh

CRANE_VERSION=v0.13.0
SEVERITY=UNKNOWN,LOW,MEDIUM,HIGH,CRITICAL

# $ sh scan.sh image-registry-checker:latest

if [ ! -d trivy ]; then
 wget --quiet https://github.com/aquasecurity/trivy/releases/download/v0.50.1/trivy_0.50.1_Linux-64bit.tar.gz
 mkdir -p trivy && tar xf trivy_0.50.1_Linux-64bit.tar.gz -C trivy
fi

./trivy/trivy repo --skip-dirs "cmd/krane,pkg/authn" --tag=${CRANE_VERSION} --severity "$SEVERITY" github.com/google/go-containerregistry
./trivy/trivy repo --severity "$SEVERITY" image-registry-checker/

./trivy/trivy image --severity "$SEVERITY" "$1"

