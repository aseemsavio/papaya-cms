# Papaya CMS — tiny Docker image (otia-ai style)
#
# Build stage: cross-compile a fully static musl Rust binary using
# messense/rust-musl-cross, then run it on `FROM scratch` (empty image).
#
# This mirrors otia-ai's approach (musl static binary + minimal final image).
# papaya uses `git2` (libgit2, a C lib) to clone templates over https://, and
# libgit2 needs OpenSSL + zlib. Those are built statically from source for musl
# in the builder so the final binary is fully self-contained and needs no
# shared libraries at runtime — only CA certificates (for TLS verification).

# ---------------- Build stage ----------------
FROM messense/rust-musl-cross:x86_64-musl AS builder

WORKDIR /app

# Build tools for compiling zlib + OpenSSL + libgit2 from source for musl
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        curl ca-certificates make perl pkg-config cmake \
    && rm -rf /var/lib/apt/lists/*

# --- zlib from source for musl (static) ---
# Use zlib's own configure (cross-friendly via CC=) rather than CMake, which
# tries to link/run test binaries and fails in cross-mode.
ARG ZLIB_VERSION=1.3.1
RUN set -eux; \
    curl -fsSL "https://zlib.net/fossils/zlib-${ZLIB_VERSION}.tar.gz" -o /tmp/zlib.tar.gz; \
    mkdir -p /tmp/zlib-src; \
    tar -xzf /tmp/zlib.tar.gz -C /tmp/zlib-src --strip-components=1; \
    cd /tmp/zlib-src; \
    CC=x86_64-unknown-linux-musl-gcc \
    AR=x86_64-unknown-linux-musl-ar \
      ./configure --prefix=/usr/local/musl --static; \
    make CFLAGS="-O3 -fPIC" -j"$(nproc)"; \
    make install; \
    rm -rf /tmp/zlib.tar.gz /tmp/zlib-src

# --- OpenSSL from source for musl (static) ---
ARG OPENSSL_VERSION=1.1.1w
RUN set -eux; \
    curl -fsSL "https://www.openssl.org/source/openssl-${OPENSSL_VERSION}.tar.gz" -o /tmp/openssl.tar.gz; \
    mkdir -p /tmp/openssl-src; \
    tar -xzf /tmp/openssl.tar.gz -C /tmp/openssl-src --strip-components=1; \
    cd /tmp/openssl-src; \
    CC=x86_64-unknown-linux-musl-gcc \
    AR=x86_64-unknown-linux-musl-ar \
    RANLIB=x86_64-unknown-linux-musl-ranlib \
      ./Configure no-shared no-zlib linux-x86_64 --prefix=/usr/local/musl --openssldir=/usr/local/musl; \
    make -j"$(nproc)"; \
    make install_sw; \
    rm -rf /tmp/openssl.tar.gz /tmp/openssl-src /var/lib/apt/lists/*

# --- libgit2 from source for musl (static) ---
# The git2 crate links libgit2 statically, so we build a static musl libgit2
# (linked against the static OpenSSL + zlib above) and ship a fully-static
# binary — nothing but CA certificates needed at runtime.
ARG LIBGIT2_VERSION=1.8.1
RUN set -eux; \
    curl -fsSL "https://github.com/libgit2/libgit2/archive/refs/tags/v${LIBGIT2_VERSION}.tar.gz" -o /tmp/libgit2.tar.gz; \
    mkdir -p /tmp/libgit2-src; \
    tar -xzf /tmp/libgit2.tar.gz -C /tmp/libgit2-src --strip-components=1; \
    cd /tmp/libgit2-src; \
    cmake -S . -B build \
      -DCMAKE_C_COMPILER=x86_64-unknown-linux-musl-gcc \
      -DCMAKE_SYSTEM_NAME=Linux \
      -DCMAKE_SYSTEM_PROCESSOR=x86_64 \
      -DCMAKE_CROSSCOMPILING=ON \
      -DCMAKE_INSTALL_PREFIX=/usr/local/musl \
      -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_C_FLAGS=-fPIC \
      -DBUILD_SHARED_LIBS=OFF \
      -DBUILD_TESTS=OFF \
      -DBUILD_CLI=OFF \
      -DUSE_THREADS=ON \
      -DTHREADSAFE=ON \
      -DREGEX_BACKEND=builtin \
      -DUSE_SSH=OFF \
      -DHTTPS_BACKEND=OpenSSL \
      -DOPENSSL_ROOT_DIR=/usr/local/musl \
      -DZLIB_ROOT=/usr/local/musl; \
    cmake --build build --target libgit2package -j"$(nproc)"; \
    cmake --install build; \
    rm -rf /tmp/libgit2.tar.gz /tmp/libgit2-src /var/lib/apt/lists/*

ENV PKG_CONFIG_ALLOW_CROSS=1 \
    PKG_CONFIG_PATH=/usr/local/musl/lib/pkgconfig \
    OPENSSL_DIR=/usr/local/musl \
    OPENSSL_LIB_DIR=/usr/local/musl/lib \
    OPENSSL_INCLUDE_DIR=/usr/local/musl/include \
    OPENSSL_STATIC=1 \
    OPENSSL_NO_VENDOR=1 \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-unknown-linux-musl-gcc

# Build the Rust binary for musl (fully static: rustls for AWS, static libgit2
# + OpenSSL + zlib for template cloning)
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY backend backend

RUN rustup target add x86_64-unknown-linux-musl \
    && cargo build --release --target x86_64-unknown-linux-musl --bin papaya-backend

# ---------------- Runtime stage ----------------
# `scratch` is an empty image (~0 MB). The binary is fully static (musl libc +
# all C deps linked in), so it runs with only CA certificates for TLS.
FROM scratch

WORKDIR /app

# CA certificates (TLS verification for git2/https template clones and AWS SDK)
COPY --from=builder /etc/ssl/certs/ /etc/ssl/certs/

# /tmp is needed by `tempfile` (used when cloning templates) — scratch has none
COPY --from=builder /tmp /tmp

# The fully-static Rust binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/papaya-backend /app/papaya-backend

EXPOSE 8080

ENTRYPOINT ["/app/papaya-backend"]
