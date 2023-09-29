## usage

### mock-runner

#### build

```bash
cargo run -- --mock-prover --circuits-rev $rev
docker build -t mock-runner:$rev .
```

#### run

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

### inner-runner

#### build

```bash
git clone -b v0.6.0 git@github.com:scroll-tech/halo2-gpu.git
cargo run -- --inner-prover --circuits-rev $rev
docker build -f gpu.Dockerfile -t inner-runner:$rev .
```

#### run

```bash
docker run \
  --rm \
  --runtime nvidia \
  -v $path_to_traces:/tmp/traces \
  -v $output_path:/tmp/output \
  -v /tmp/worker-done:/tmp/worker-done \
  -v /home/scroll/go-prover-docker/volume/params:/opt/test_params \
  -e TRACES_PATH=/tmp/traces \
  -e OUTPUT_PATH=/tmp/output \
  -e TOTAL_WORKERS=$total_workers \
  -e WORKER_INDEX=$worker_index \
  inner-runner:$rev
```

### chunk-runner

#### build

```bash
git clone -b v0.6.0 git@github.com:scroll-tech/halo2-gpu.git
cargo run -- --chunk-prover --circuits-rev $rev
docker build -f gpu.Dockerfile -t chunk-runner:$rev .
```

#### run

```bash
docker run \
  --rm \
  --runtime nvidia \
  -v $path_to_traces:/tmp/traces \
  -v $output_path:/tmp/output \
  -v /tmp/worker-done:/tmp/worker-done \
  -v /home/scroll/go-prover-docker/volume/params:/opt/test_params \
  -e TRACES_PATH=/tmp/traces \
  -e OUTPUT_PATH=/tmp/output \
  -e TOTAL_WORKERS=$total_workers \
  -e WORKER_INDEX=$worker_index \
  chunk-runner:$rev
```
