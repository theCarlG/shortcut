FROM docker.io/library/ubuntu:20.04

COPY scripts/install-deps.sh ./

ENV TZ=UTC
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

RUN apt-get update && apt-get install -y tzdata

RUN ./install-deps.sh

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s --  -y

ENV PATH /root/.cargo/bin/:$PATH

RUN set -eux; \
    rustup update; \
    rustup component add clippy rustfmt; \
    rustup target add x86_64-unknown-linux-gnu x86_64-unknown-linux-musl

RUN set -eux; \
    deny_version="0.12.1"; \
    curl --silent -L https://github.com/EmbarkStudios/cargo-deny/releases/download/$deny_version/cargo-deny-$deny_version-x86_64-unknown-linux-musl.tar.gz | tar -xzv -C /usr/bin --strip-components=1; \
    cargo-deny -V;

RUN set -eux; \
    apt-get remove -y --auto-remove; \
    rm -rf /var/lib/apt/lists/*;

ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="clang" \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=/usr/local/bin/mold"\
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="clang" \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C link-arg=-fuse-ld=/usr/local/bin/mold" 
