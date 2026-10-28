# syntax=docker/dockerfile:1
#
# OMG shape_main (C판, srcC/) 을 Cyclone DDS master (f2aaccb4, 버전 번호 11.0.1) 로 소스에서 빌드한다.
# 설계와 버전 근거: docs/environment/vendor-build.md
#
# 결과 이미지: /usr/local/bin/cyclone_shape
# 빌드: docker compose build cyclone   (docker/ 에서)
# 병렬 수 조절: docker compose build --build-arg JOBS=2 cyclone
#
# BASE_IMAGE, DDS_RTPS_COMMIT 은 docker/fastdds/Dockerfile 과 같은 값이어야 한다.
# 바꿀 때는 두 파일을 함께 고친다.

# ---------------------------------------------------------------------------
# 버전 고정. 태그로 clone 한 뒤 커밋 해시가 다르면 빌드를 멈춘다.
# 해시는 2026-10-28 git ls-remote 로 확인 (lightweight 태그라 태그 해시 = 커밋 해시).
# ---------------------------------------------------------------------------
# ubuntu:24.04 는 계속 새 스냅숏으로 옮겨지므로 날짜 태그와 다이제스트로 고정한다.
# 다이제스트는 멀티 아키텍처 인덱스라 x86_64 머신과 Mac (arm64) 모두 같은 값을 쓴다.
ARG BASE_IMAGE=ubuntu:noble-20261024.1@sha256:90b3a804b4ba0a340cfa4ba8950a543464e15d97d4fc33a74c9613d239661f10

# 릴리스 태그 v2.1.2026 에는 srcC/ 가 없어서 커밋으로 고정한다.
ARG DDS_RTPS_COMMIT=526dc56eed8b1965a81ba5b7a644c4489337fe0a

# 릴리스 태그가 아니라 master 커밋으로 고정한다. srcC/shape_main.c 가 쓰는
# dds_read_next_instance / dds_take_next_instance 는 11.0.1 이후 master 에만 있다
# (2026-03-31 #2376 추가, 2026-05-25 a47f209f 수정). 이 커밋은 dds-rtps #87 (2026-06-09)
# 직전의 master (2026-06-02) 이고, CMake 버전 번호는 여전히 11.0.1 이다 (태그 11.0.1 보다 121 커밋 앞).
ARG CYCLONE_COMMIT=f2aaccb4873c4bc33d59ee45c8568a4653900f78

# 0이면 nproc 를 쓴다
ARG JOBS=0

# ===========================================================================
# stage src: shape_main 소스 (omg-dds/dds-rtps)
# ===========================================================================
FROM ${BASE_IMAGE} AS src
ARG DDS_RTPS_COMMIT
RUN apt-get update \
 && apt-get install -y --no-install-recommends git ca-certificates \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /src
RUN git clone https://github.com/omg-dds/dds-rtps.git \
 && cd dds-rtps \
 && git checkout --detach "${DDS_RTPS_COMMIT}"

# ===========================================================================
# stage build: Cyclone DDS 라이브러리 + cyclone_shape
# ===========================================================================
FROM ${BASE_IMAGE} AS build
ARG CYCLONE_COMMIT
ARG JOBS
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      build-essential cmake git ca-certificates libssl-dev \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /build
# libssl-dev 가 있으면 ENABLE_SSL (기본 AUTO) 과 보안 플러그인이 켜진다. OMG 릴리스와 기능을 맞춘다.
# INSTALL_LIBDIR 를 lib 으로 고정해 runtime 단계의 COPY 경로를 확정한다.
# 태그가 없으므로 커밋 하나만 받는다 (GitHub 은 해시로 fetch 를 허용한다).
RUN git init cyclonedds \
 && git -C cyclonedds fetch --depth 1 https://github.com/eclipse-cyclonedds/cyclonedds.git "${CYCLONE_COMMIT}" \
 && git -C cyclonedds checkout --detach FETCH_HEAD \
 && test "$(git -C cyclonedds rev-parse HEAD)" = "${CYCLONE_COMMIT}" \
 && cmake -S cyclonedds -B cyclonedds/build \
      -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_INSTALL_PREFIX=/opt/cyclone \
      -DCMAKE_INSTALL_LIBDIR=lib \
      -DBUILD_IDLC=ON \
      -DBUILD_EXAMPLES=OFF \
      -DBUILD_TESTING=OFF \
 && cmake --build cyclonedds/build --target install -j"$([ "${JOBS}" -gt 0 ] && echo "${JOBS}" || nproc)" \
 && rm -rf cyclonedds

# OMG cyclone-dds-cmake 대신 idlc + gcc 로 직접 빌드한다 (vendor-build.md "설계 메모").
# -Wno-implicit-extensibility: OMG CMakeLists 의 WARNINGS no-implicit-extensibility 와 같다
#   (Cyclone 11.0.1 Generate.cmake 가 WARNINGS 를 "-W<값>" 으로 넘기는 것을 확인).
# -DCYCLONE_DDS: shape_main.c 가 이 매크로로 Cyclone 용 코드를 고른다.
COPY --from=src /src/dds-rtps/srcC /work/srcC
WORKDIR /work/srcC
RUN /opt/cyclone/bin/idlc -Wno-implicit-extensibility shape.idl \
 && gcc -O2 -DCYCLONE_DDS -I. -I/opt/cyclone/include \
      shape_main.c shape.c \
      -L/opt/cyclone/lib -Wl,-rpath,/opt/cyclone/lib -lddsc -lm \
      -o cyclone_shape

# ===========================================================================
# stage runtime: cyclone_shape + 캡처/진단 도구
# ===========================================================================
FROM ${BASE_IMAGE} AS runtime
ARG DDS_RTPS_COMMIT
ARG CYCLONE_COMMIT

# 패키지 이름은 Ubuntu 24.04 기준. 실제 필요한 것은 빌드 후 ldd 로 확인해 맞춘다.
# libssl3t64: libddsc 와 보안 플러그인이 쓰는 OpenSSL
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      tcpdump socat iputils-ping procps iproute2 \
      libssl3t64 \
 && rm -rf /var/lib/apt/lists/*

# Cyclone 은 공유 라이브러리 (libddsc.so 와 보안 플러그인)
COPY --from=build /opt/cyclone/lib /opt/cyclone/lib
RUN echo /opt/cyclone/lib > /etc/ld.so.conf.d/cyclone.conf && ldconfig

# 이름을 15자 이하로 둔다: pgrep -x / pkill -x 는 프로세스 이름 앞 15자만 비교한다.
COPY --from=build /work/srcC/cyclone_shape /usr/local/bin/cyclone_shape

LABEL org.opencontainers.image.title="mdds cyclone_shape" \
      org.opencontainers.image.description="OMG dds-rtps shape_main on Cyclone DDS, built from source" \
      mdds.dds-rtps.commit="${DDS_RTPS_COMMIT}" \
      mdds.cyclonedds.version="11.0.1 (master)" \
      mdds.cyclonedds.commit="${CYCLONE_COMMIT}"

WORKDIR /captures
# 컨테이너는 대기만 하고, 시나리오는 docker compose exec 로 실행한다.
CMD ["sleep", "infinity"]
