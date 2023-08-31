#!/bin/bash

set -ex

JOB_FILE=${JOB_FILE:"traces.txt"}
TOTAL_JOBS=$(wc -l < $JOB_FILE)
TOTAL_WORKERS=${TOTAL_WORKERS:10}
JOB_COUNT_PER_WORKER=$(( TOTAL_JOBS / TOTAL_WORKERS ))

START_LINE=$(( WORKER_INDEX * JOB_COUNT_PER_WORKER + 1 ))

END_LINE=$(( START_LINE + JOB_COUNT_PER_WORKER - 1 ))

worker_jobs=$(sed -n "${START_LINE},${END_LINE}p" $JOB_FILE)
mkdir -p "$OUTPUT_PATH"

echo "Jobs for worker $WORKER_INDEX:"
for job in $worker_jobs; do
  # extract filename from job path
  # eg. 'hermez-traces/uniswapv2-router-benchmark_0.json' -> 'uniswapv2-router-benchmark_0'
  name=$(echo "$job" | sed 's/.*\///' | sed 's/\.[^.]*$//')
  trace-prover "$job" --output "$OUTPUT_PATH/$name-result.json"
done