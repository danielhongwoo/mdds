# syntax=docker/dockerfile:1
#
# mdds 워크스페이스의 진단 도구 (지금은 mcast-check 하나) 를 담는 이미지.
# 설계: docs/environment/docker-network.md "검증 명령"
#
# 빌드 컨텍스트는 저장소 루트다 (.dockerignore 가 워크스페이스만 남긴다).
# 빌드: docker compose build tools   (docker/ 에서)
# 실행: docker/mcast-check.sh <cyclone|fastdds> recv|send
#       벤더 컨테이너의 네트워크 네임스페이스를 빌려 같은 eth0 에서 돈다.

# Rust 버전은 저장소 루트 rust-toolchain.toml 의 channel 과 같아야 한다.
# 이미지에 그 버전이 이미 있으므로 빌드 중에 툴체인을 받지 않는다.
# bookworm (glibc 2.36) 에서 빌드해 runtime 의 Ubuntu 24.04 (glibc 2.39) 에서도 돈다.
ARG RUST_IMAGE=rust:1.99.1-slim-bookworm@sha256:61e86bac21b4c1aeea7100008e1902d0048f9842fc94c7fb9c5f01baecbcb475

# 벤더 이미지 (docker/cyclone, docker/fastdds) 와 같은 베이스를 쓴다.
ARG BASE_IMAGE=ubuntu:noble-20261024.1@sha256:90b3a804b4ba0a340cfa4ba8950a543464e15d97d4fc33a74c9613d239661f10

# ===========================================================================
# stage build
# ===========================================================================
FROM ${RUST_IMAGE} AS build
WORKDIR /src
COPY . .
# 외부 의존성이 없어서 네트워크 없이 빌드된다. --locked: Cargo.lock 과 다르면 멈춘다.
RUN test "$(rustc --version | cut -d' ' -f2)" = "$(sed -n 's/^channel = "\(.*\)"/\1/p' rust-toolchain.toml)" \
 && cargo build --release --locked -p mcast-check

# ===========================================================================
# stage runtime
# ===========================================================================
FROM ${BASE_IMAGE} AS runtime
COPY --from=build /src/target/release/mcast-check /usr/local/bin/mcast-check
LABEL org.opencontainers.image.title="mdds tools" \
      org.opencontainers.image.description="mdds diagnostic tools (mcast-check)"
CMD ["mcast-check", "--help"]
