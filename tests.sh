#!/bin/bash

# image-registry-checker integration tests

# heavily inspired by
# Python's "assert" syntax
# https://github.com/pytest-dev/pytest
# https://github.com/sstephenson/bats
# https://github.com/torokmark/assert.sh

# relevant docs
# https://www.gnu.org/software/bash/manual/html_node/Bash-Builtins.html
# https://www.gnu.org/software/bash/manual/html_node/Bash-Conditional-Expressions.html
# https://www.gnu.org/software/bash/manual/html_node/Bash-Variables.html
# https://www.gnu.org/software/bash/manual/html_node/Bourne-Shell-Builtins.html
# https://www.gnu.org/software/bash/manual/html_node/Command-Substitution.html
# https://www.gnu.org/software/bash/manual/html_node/Conditional-Constructs.html
# https://www.gnu.org/software/bash/manual/html_node/Redirections.html
# https://www.gnu.org/software/bash/manual/html_node/Shell-Functions.html
# https://www.gnu.org/software/bash/manual/html_node/Special-Parameters.html
# https://www.gnu.org/software/gawk/manual/html_node/Fields.html

# see also: https://www.shellcheck.net/

assert() {

 # https://stackoverflow.com/a/372120
 # https://stackoverflow.com/a/36585074

 "$@"; EXITCODE=$?
 
 # declare -p "EXITCODE" 2>/dev/null # debugging
 
 printf '%s ... ' "${FUNCNAME[1]}"
 printf '%s ' "$@" # optional
 printf '... ' # optional

 if [ "$EXITCODE" -eq 0 ]; then
  echo "PASSED"
 else
  echo "FAILED"
 fi

}

response_only() {
 "$@" | awk '{ print $NF }'
}

selftest_assert_string_conditionals() {
  RESPONSE=$(response_only assert [ "ABC" = "ABC" ])
  if [ "$RESPONSE" = 'PASSED' ]; then echo "${FUNCNAME[0]} ... PASSED"; else echo "${FUNCNAME[0]} ... FAILED" && exit 1; fi
  RESPONSE=$(response_only assert [ "ABC" = "DEF" ])
  if [ "$RESPONSE" = 'FAILED' ]; then echo "${FUNCNAME[0]} ... PASSED"; else echo "${FUNCNAME[0]} ... FAILED" && exit 1; fi
}

selftest_assert_arithmetic_conditionals() {
  RESPONSE=$(response_only assert [ 0 -eq 0 ])
  if [ "$RESPONSE" = 'PASSED' ]; then echo "${FUNCNAME[0]} ... PASSED"; else echo "${FUNCNAME[0]} ... FAILED" && exit 1; fi
  RESPONSE=$(response_only assert [ 0 -eq 1 ])
  if [ "$RESPONSE" = 'FAILED' ]; then echo "${FUNCNAME[0]} ... PASSED"; else echo "${FUNCNAME[0]} ... FAILED" && exit 1; fi
}

test_if_service_is_running() {
  DOCKER_CONTAINER_STATE=$(sleep 10 && docker inspect --format "{{.State.Status}}" "$CONTAINER")
  assert [ "$DOCKER_CONTAINER_STATE" = "running" ]
}

get_http_status_code() {
  # https://superuser.com/a/442395
  curl "$1" -s -o /dev/null -w '%{http_code}'
}

test_if_health_endpoint_is_served() {
  HTTP_STATUS_CODE=$(get_http_status_code "http://localhost:8080/health")
  assert [ "$HTTP_STATUS_CODE" = "200" ]
}

test_if_swagger_endpoint_is_served() {
  HTTP_STATUS_CODE=$(get_http_status_code "http://localhost:8080/swagger-ui/index.html")
  assert [ "$HTTP_STATUS_CODE" = "200" ]
}

test_if_apidoc_endpoint_is_served() {
  HTTP_STATUS_CODE=$(get_http_status_code "http://localhost:8080/api-doc.json")
  assert [ "$HTTP_STATUS_CODE" = "200" ]
}

test_http_status_return_codes() {
  # https://github.com/martinclaus/image-registry-checker/blob/03b4491500b4e7f8e44faa22c4aebd8eb46f1026/image-registry-checker/src/main.rs#L231
  HTTP_STATUS_CODE=$(get_http_status_code "http://localhost:8080/exists?image=docker.io/alpine")
  assert [ "$HTTP_STATUS_CODE" = "200" ]
  # https://github.com/martinclaus/image-registry-checker/blob/03b4491500b4e7f8e44faa22c4aebd8eb46f1026/image-registry-checker/src/main.rs#L240
  HTTP_STATUS_CODE=$(get_http_status_code "http://localhost:8080/exists?image=docker.io/non-existent")
  assert [ "$HTTP_STATUS_CODE" = "404" ]
}

test_openolat_return_code_interoperability() {
  # HTTP return codes expected by OpenOLAT JupyterHub course element:
  # https://github.com/OpenOLAT/OpenOLAT/blob/OpenOLAT_18.2.3/src/main/java/org/olat/modules/jupyterhub/ui/JupyterHubConfigTabController.java#L186-L216
  HTTP_STATUS_CODE=$(get_http_status_code "http://localhost:8080/exists?image=docker.io/alpine")
  assert [ "$HTTP_STATUS_CODE" -ge 200 -a "$HTTP_STATUS_CODE" -lt 300 ]
  HTTP_STATUS_CODE=$(get_http_status_code "http://localhost:8080/exists?image=docker.io/non-existent")
  assert [ "$HTTP_STATUS_CODE" = "404" ]
}

echo ""; echo "Running selftests..."
selftest_assert_string_conditionals
selftest_assert_arithmetic_conditionals

echo ""; echo "Building container image..."
docker build -t image-registry-checker:latest .

CONTAINER=image-registry-checker

echo ""; echo "Starting service..."
docker run --detach --name "$CONTAINER" -p 8080:8080 image-registry-checker:latest 1>/dev/null
docker logs "$CONTAINER"

echo ""; echo "Ensuring that service is running..."
test_if_service_is_running

echo ""; echo "Starting integration tests..."
test_if_health_endpoint_is_served
test_if_swagger_endpoint_is_served
test_if_apidoc_endpoint_is_served
test_http_status_return_codes
test_openolat_return_code_interoperability

echo ""; echo "Fetching service logs..."
docker logs "$CONTAINER"

echo ""; echo "Stopping service..."
docker container stop "$CONTAINER" 1>/dev/null
docker container rm "$CONTAINER" 1>/dev/null

echo ""; echo "Done."

