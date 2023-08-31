FROM scrolltech/go-rust-builder:go-1.19-rust-nightly-2022-12-10 as builder

COPY rust-toolchain /rust-toolchain
COPY .cargo /.cargo
COPY Cargo.toml /Cargo.toml
COPY src /src
RUN cargo build --release --bin mock-runner
RUN cp `find ./target/release/ | grep libzktrie.so` /usr/lib/

FROM ubuntu:20.04

WORKDIR /
COPY --from=builder /target/release/mock-runner /usr/local/bin/
COPY --from=builder /usr/lib/libzktrie.so /usr/lib/
COPY runner.sh /runner.sh

ENTRYPOINT ["/runner.sh"]