# syntax=docker/dockerfile:1
#
# OMG shape_main (C++판, srcCxx/) 을 Fast DDS 3.6.1 로 소스에서 빌드한다.
# 설계와 버전 근거: docs/environment/vendor-build.md
#
# 결과 이미지: /usr/local/bin/fastdds_shape
# 빌드: docker compose build fastdds   (docker/ 에서)
# 병렬 수 조절: docker compose build --build-arg JOBS=2 fastdds
#
# BASE_IMAGE, DDS_RTPS_COMMIT 은 docker/cyclone/Dockerfile 과 같은 값이어야 한다.
# 바꿀 때는 두 파일을 함께 고친다.

# ---------------------------------------------------------------------------
# 버전 고정. 태그로 clone 한 뒤 커밋 해시가 다르면 빌드를 멈춘다.
# 해시는 2026-10-28 git ls-remote 로 확인 (lightweight 태그라 태그 해시 = 커밋 해시).
# Fast-CDR, foonathan_memory_vendor, Fast-DDS-Gen 버전은 Fast DDS v3.6.1 의 fastdds.repos 기준.
# ---------------------------------------------------------------------------
# ubuntu:24.04 는 계속 새 스냅숏으로 옮겨지므로 날짜 태그와 다이제스트로 고정한다.
# 다이제스트는 멀티 아키텍처 인덱스라 x86_64 머신과 Mac (arm64) 모두 같은 값을 쓴다.
ARG BASE_IMAGE=ubuntu:noble-20261024.1@sha256:90b3a804b4ba0a340cfa4ba8950a543464e15d97d4fc33a74c9613d239661f10

# 두 벤더가 같은 shape_main 소스 커밋을 쓰도록 Cyclone 과 같은 커밋으로 고정한다.
ARG DDS_RTPS_COMMIT=526dc56eed8b1965a81ba5b7a644c4489337fe0a

ARG FOONATHAN_TAG=v1.4.1
ARG FOONATHAN_COMMIT=347cb67581e51273a612780eb256a3c134c10bae
ARG FASTCDR_TAG=v2.3.5
ARG FASTCDR_COMMIT=7d33a3b51a1585f5631b0a8d905bcc4f249d0f34
ARG FASTDDS_TAG=v3.6.1
ARG FASTDDS_COMMIT=4e81e8b71bcd6e7c5213c000503cba8e49d6022a
ARG FASTDDSGEN_TAG=v4.3.0
ARG FASTDDSGEN_COMMIT=cc0072b8849b35c67bd7e187990efad58e2871ae

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
# stage build: Fast DDS (정적 라이브러리) + fastddsgen + fastdds_shape
# ===========================================================================
FROM ${BASE_IMAGE} AS build
ARG FOONATHAN_TAG
ARG FOONATHAN_COMMIT
ARG FASTCDR_TAG
ARG FASTCDR_COMMIT
ARG FASTDDS_TAG
ARG FASTDDS_COMMIT
ARG FASTDDSGEN_TAG
ARG FASTDDSGEN_COMMIT
ARG JOBS
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      build-essential cmake git ca-certificates \
      libasio-dev libtinyxml2-dev libssl-dev \
      openjdk-17-jdk-headless \
 && rm -rf /var/lib/apt/lists/*

# 세 저장소 공통 옵션. BUILD_SHARED_LIBS=OFF 로 정적 라이브러리만 설치해야
# shape_main 의 find_package(fastdds) 가 정적 타깃을 고른다 (FASTDDS_STATIC 은 효과 없음).
ENV CMAKE_COMMON="-DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/opt/fastdds -DCMAKE_PREFIX_PATH=/opt/fastdds -DBUILD_SHARED_LIBS=OFF"

WORKDIR /build
# 1) foonathan_memory_vendor: configure 중에 foonathan/memory 소스를 GitHub 에서 받는다 (네트워크 필요)
RUN git clone --depth 1 -b "${FOONATHAN_TAG}" https://github.com/eProsima/foonathan_memory_vendor.git \
 && test "$(git -C foonathan_memory_vendor rev-parse HEAD)" = "${FOONATHAN_COMMIT}" \
 && cmake -S foonathan_memory_vendor -B foonathan_memory_vendor/build ${CMAKE_COMMON} \
 && cmake --build foonathan_memory_vendor/build --target install -j"$([ "${JOBS}" -gt 0 ] && echo "${JOBS}" || nproc)" \
 && rm -rf foonathan_memory_vendor

# 2) Fast-CDR
RUN git clone --depth 1 -b "${FASTCDR_TAG}" https://github.com/eProsima/Fast-CDR.git \
 && test "$(git -C Fast-CDR rev-parse HEAD)" = "${FASTCDR_COMMIT}" \
 && cmake -S Fast-CDR -B Fast-CDR/build ${CMAKE_COMMON} \
 && cmake --build Fast-CDR/build --target install -j"$([ "${JOBS}" -gt 0 ] && echo "${JOBS}" || nproc)" \
 && rm -rf Fast-CDR

# 3) Fast DDS. 도구와 예제는 빌드 시간을 줄이려고 끈다.
RUN git clone --depth 1 -b "${FASTDDS_TAG}" https://github.com/eProsima/Fast-DDS.git \
 && test "$(git -C Fast-DDS rev-parse HEAD)" = "${FASTDDS_COMMIT}" \
 && cmake -S Fast-DDS -B Fast-DDS/build ${CMAKE_COMMON} \
      -DCOMPILE_TOOLS=OFF \
      -DCOMPILE_EXAMPLES=OFF \
 && cmake --build Fast-DDS/build --target install -j"$([ "${JOBS}" -gt 0 ] && echo "${JOBS}" || nproc)" \
 && rm -rf Fast-DDS

# 4) Fast-DDS-Gen (IDL -> C++). thirdparty/idl-parser 서브모듈이 필요하다.
#    Gradle wrapper 가 Gradle 배포본과 Maven 의존성을 내려받는다 (네트워크 필요).
RUN git clone --depth 1 --recurse-submodules --shallow-submodules -b "${FASTDDSGEN_TAG}" \
      https://github.com/eProsima/Fast-DDS-Gen.git /opt/Fast-DDS-Gen \
 && test "$(git -C /opt/Fast-DDS-Gen rev-parse HEAD)" = "${FASTDDSGEN_COMMIT}" \
 && cd /opt/Fast-DDS-Gen \
 && ./gradlew --no-daemon assemble
ENV PATH="/opt/Fast-DDS-Gen/scripts:${PATH}"

# 5) shape_main (OMG fast-dds-cmake 를 그대로 쓴다).
#    생성 코드는 CMakeLists 가 정한 대로 /work/srcCxx/GeneratedCode 에 생긴다.
#    결과 이름: eprosima_fastdds-<fastdds_VERSION>_shape_main_linux (버전 확인용으로 이름을 남긴다)
COPY --from=src /src/dds-rtps/srcCxx /work/srcCxx
WORKDIR /work/srcCxx
RUN cmake -S fast-dds-cmake -B build \
      -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_PREFIX_PATH=/opt/fastdds \
 && cmake --build build -j"$([ "${JOBS}" -gt 0 ] && echo "${JOBS}" || nproc)" \
 && ls -l build/eprosima_fastdds-*_shape_main_linux \
 && cp build/eprosima_fastdds-*_shape_main_linux /work/fastdds_shape \
 && ls build/eprosima_fastdds-*_shape_main_linux | xargs -n1 basename > /work/fastdds_shape.origname

# ===========================================================================
# stage runtime: fastdds_shape + 캡처/진단 도구
# ===========================================================================
FROM ${BASE_IMAGE} AS runtime
ARG DDS_RTPS_COMMIT
ARG FASTDDS_TAG
ARG FASTDDS_COMMIT
ARG FASTCDR_TAG
ARG FOONATHAN_TAG
ARG FASTDDSGEN_TAG

# 패키지 이름은 Ubuntu 24.04 기준. 실제 필요한 것은 빌드 후 ldd 로 확인해 맞춘다.
# libtinyxml2-10, libssl3t64: 정적 링크한 Fast DDS 가 공유 라이브러리로 쓰는 것
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      tcpdump socat iputils-ping procps iproute2 \
      libtinyxml2-10 libssl3t64 \
 && rm -rf /var/lib/apt/lists/*

# 이름을 15자 이하로 둔다: pgrep -x / pkill -x 는 프로세스 이름 앞 15자만 비교한다.
COPY --from=build /work/fastdds_shape /usr/local/bin/fastdds_shape
COPY --from=build /work/fastdds_shape.origname /usr/local/share/fastdds_shape.origname

LABEL org.opencontainers.image.title="mdds fastdds_shape" \
      org.opencontainers.image.description="OMG dds-rtps shape_main on Fast DDS, built from source" \
      mdds.dds-rtps.commit="${DDS_RTPS_COMMIT}" \
      mdds.fastdds.tag="${FASTDDS_TAG}" \
      mdds.fastdds.commit="${FASTDDS_COMMIT}" \
      mdds.fastcdr.tag="${FASTCDR_TAG}" \
      mdds.foonathan-memory-vendor.tag="${FOONATHAN_TAG}" \
      mdds.fastddsgen.tag="${FASTDDSGEN_TAG}"

WORKDIR /captures
# 컨테이너는 대기만 하고, 시나리오는 docker compose exec 로 실행한다.
CMD ["sleep", "infinity"]
