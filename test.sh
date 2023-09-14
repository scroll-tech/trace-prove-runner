#!/bin/bash

set -ex

docker run \
  --rm \
  --name mock-runner-test \
  -v /srv:/srv \
  -e TRACES_PATH=/srv/tracetest/traces/mainnet-0914/traces \
  -e WORKER_INDEX=0 \
  -e TOTAL_WORKERS=50 \
  mock-runner:b99974d2d37696562a1035c0e595dbc87fabaa62