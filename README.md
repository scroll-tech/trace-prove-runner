## usage

### build runner

```bash
cargo run -- --mock-prover --circuits-rev $rev
cd mock-runner
docker build -t mock-runner:$rev .
```

### run mock-runner

```bash
docker run \
  --rm \
  -v $path_to_traces:/tmp/traces \
  -v $output_path:/tmp/output \
  -v /tmp/worker-done:/tmp/worker-done \
  -e TRACES_PATH=/tmp/traces \
  -e OUTPUT_PATH=/tmp/output \
  -e TOTAL_WORKERS=$total_workers \
  -e WORKER_INDEX=$worker_index \
  mock-runner:$rev
```