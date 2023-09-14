FROM scrolltech/go-rust-builder:go-1.19-rust-nightly-2022-12-10 as builder

COPY worker /worker
RUN cd /worker && cargo build --release
COPY mock-runner /mock-runner
RUN cd /mock-runner && \
    cargo build --release --bin mock-runner && \
    cp `find ./target/release/ | grep libzktrie.so` /usr/lib/

FROM ubuntu:20.04

ENV RUST_BACKTRACE=1
ENV RUST_LOG=info
ENV RUNNER_PATH=/usr/local/bin/mock-runner
ENV OUTPUT_PATH=/srv/tracetest/output
COPY --from=builder /worker/target/release/worker /usr/local/bin/
COPY --from=builder /target/release/mock-runner /usr/local/bin/
COPY --from=builder /usr/lib/libzktrie.so /usr/lib/

ENTRYPOINT ["/usr/local/bin/worker"]