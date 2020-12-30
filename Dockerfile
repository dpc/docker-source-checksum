# rust:1.48.0
FROM rust@sha256:85efc99ac7527e431834c05bd40df33c385bf8606ae3c8b27a6be864b9982b8d AS dsc-builder
RUN apt-get update && apt-get install -y musl-tools curl llvm clang
RUN rustup target add x86_64-unknown-linux-musl
COPY . /home/docker-source-checksum
WORKDIR /home/docker-source-checksum
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=dsc-builder /home/docker-source-checksum/target/x86_64-unknown-linux-musl/release/docker-source-checksum /usr/bin/docker-source-checksum
ENTRYPOINT ["/usr/bin/docker-source-checksum"]
CMD [""]
