FROM gitpod/workspace-full:2022-10-17-21-33-26

RUN mkdir -p ~/.local/bin
RUN curl -L https://github.com/stellar/soroban-cli/releases/download/v0.1.2/soroban-cli-0.1.2-x86_64-unknown-linux-gnu.tar.gz | tar xz -C ~/.local/bin soroban
RUN curl -L https://github.com/mozilla/sccache/releases/download/v0.3.0/sccache-v0.3.0-x86_64-unknown-linux-musl.tar.gz | tar xz --strip-components 1 -C ~/.local/bin sccache-v0.3.0-x86_64-unknown-linux-musl/sccache
RUN chmod +x ~/.local/bin/sccache

ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_CACHE_SIZE=2G

RUN rustup update
RUN rustup install nightly
RUN rustup target add --toolchain nightly wasm32-unknown-unknown
RUN rustup component add --toolchain nightly rust-src

RUN sudo apt-get update && sudo apt-get install -y binaryen

RUN docker pull stellar/quickstart:soroban-dev
