FROM scrolltech/cuda-go-rust-builder:cuda-11.7.1-go-1.21-rust-nightly-2023-12-03 as builder

ENV LD_LIBRARY_PATH /usr/local/cuda/lib64:$LD_LIBRARY_PATH
COPY halo2-gpu /halo2-gpu
RUN mkdir /.cargo && echo 'paths = ["/halo2-gpu/halo2_proofs"]' > /.cargo/config
COPY worker /worker
RUN cd /worker && cargo build --release
COPY mock-runner /mock-runner
RUN cd /mock-runner && \
    cargo build --release --bin mock-runner && \
    cp `find ./target/release/ | grep libzktrie.so` /usr/lib/
RUN apt update && apt install -y curl
RUN mkdir /configs
RUN curl -o /configs/layer1.config https://circuit-release.s3.us-west-2.amazonaws.com/release-v0.9.4/layer1.config
RUN curl -o /configs/layer2.config https://circuit-release.s3.us-west-2.amazonaws.com/release-v0.9.4/layer2.config

FROM nvidia/cuda:11.7.1-runtime-ubuntu22.04

WORKDIR /opt
ENV LD_LIBRARY_PATH /usr/local/cuda/lib64:$LD_LIBRARY_PATH
ENV SCROLL_PROVER_PARAMS_DIR /opt/test_params
ENV RUST_MIN_STACK 100000000
ENV RUST_BACKTRACE=1
ENV RUST_LOG=info
ENV RUNNER_PATH=/usr/local/bin/mock-runner
ENV OUTPUT_PATH=/srv/tracetest/output
COPY --from=builder /worker/target/release/worker /usr/local/bin/
COPY --from=builder /mock-runner/target/release/mock-runner /usr/local/bin/
COPY --from=builder /usr/lib/libzktrie.so /usr/lib/
COPY --from=builder /configs/ /opt/configs/

ENTRYPOINT ["/usr/local/bin/worker"]
