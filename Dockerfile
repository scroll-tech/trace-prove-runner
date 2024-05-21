FROM scrolltech/go-rust-builder:go-1.21-rust-nightly-2023-12-03 as builder

COPY worker /worker
RUN cd /worker && cargo build --release
COPY mock-runner /mock-runner
RUN cd /mock-runner && \
    cargo build --release --bin mock-runner

FROM ubuntu:20.04

ENV RUST_BACKTRACE=1
ENV RUST_LOG=info
ENV RUNNER_PATH=/usr/local/bin/mock-runner
ENV OUTPUT_PATH=/srv/tracetest/output
COPY --from=builder /worker/target/release/worker /usr/local/bin/
COPY --from=builder /mock-runner/target/release/mock-runner /usr/local/bin/

ENTRYPOINT ["/usr/local/bin/worker"]