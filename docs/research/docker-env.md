# 조사 노트: Docker로 Cyclone DDS / Fast DDS 실행

[roadmap](../roadmap.md) | [Phase 00](../phases/phase-00-setup.md)

- 조사일: 2026-10-28
- 환경 전제: Apple Silicon Mac (arm64) + Docker Desktop, GUI 없이 CLI 도구만 사용
- 표시: [확인] 페이지나 파일에서 직접 확인, [추정] 미확인

## 1. OMG 상호운용 테스트의 shape_main [확인]
- 릴리스: https://github.com/omg-dds/dds-rtps/releases/tag/v2.1.2026 (2026-02-27)
- 첨부: connext_dds-7.7.0, dust_dds-0.16.1, eclipse_cyclone-11.0.1, eprosima_fastdds-3.6.1, hdds-1.4.8, intercom_dds-4.3.1, opendds-3.35.0-dev, toc_coredx_dds-6.15.3, zzdds-0.3.2 (각각 `<이름>_shape_main_linux.zip`)
- Cyclone, Fast DDS 실행 파일은 zip 하나에 파일 하나이고, `file` 결과는 **ELF 64-bit x86-64, 동적 링크** (2026-10-28 확인)
  - Apple Silicon에서는 `--platform linux/amd64`로 실행한다.
  - 동적 링크라 실행에 필요한 공유 라이브러리를 이미지에 맞춰야 한다. `ldd`로 확인한다 [추정].
- 사용법 (플래그): -P / -S (발행 / 구독), -d (도메인), -t (토픽), -r / -b (reliable / best-effort), -D (durability), -x (1 = XCDR, 2 = XCDR2), -k (history depth). 자세한 사용법은 `-h`로 확인한다 [추정].
- "eclipse_cyclone-11.0.1"의 버전 번호 의미는 확인하지 못했다 [추정].

## 2. Cyclone DDS
- 공식 Docker 이미지는 찾지 못했다 [추정].
- Ubuntu 24.04 apt: `cyclonedds-tools` 0.10.4 (universe), `libddsc0t64` 의존 [확인]. Ubuntu 22.04는 0.8.2만 있다.
- CLI
  - Python CLI `cyclonedds` (`pip install cyclonedds`): `ls`, `subscribe`, `publish`, `ps`, `typeof`, `performance` [확인]
  - `ddsperf`, `idlc`, Roundtrip 예제 (빌드 옵션) [확인]
- 소스 빌드: `cmake -DCMAKE_INSTALL_PREFIX=... -DBUILD_DDSPERF=ON -DBUILD_EXAMPLES=ON ..` 후 `cmake --build . --target install`. CMake 3.16 이상, OpenSSL / Iceoryx는 선택 [확인]

## 3. Fast DDS
- 최신 공식 Docker 이미지는 찾지 못했다. eProsima 이미지는 Ubuntu 20.04 기반의 예전 것 [추정].
- Ubuntu apt 패키지는 찾지 못했다 [추정].
- 소스 빌드: foonathan_memory, fastcdr, fastdds 순서. `libasio-dev`, `libtinyxml2-dev` 필요. Linux aarch64 지원 [요약]
- 그래서 Fast DDS 쪽은 **shape_main을 쓰는 것이 가장 간단하다.**

## 4. ROS 2 이미지
- Docker Hub `ros`: humble, jazzy, kilted, lyrical, rolling. `-ros-core`, `-ros-base`, `-perception` 변형. **arm64 지원** [확인]
- Cyclone RMW: `ros-jazzy-rmw-cyclonedds-cpp` 설치 후 `RMW_IMPLEMENTATION=rmw_cyclonedds_cpp` [확인]
- jazzy의 기본 RMW가 Fast DDS라는 점은 이번 조사에서 직접 확인하지 못했다 [추정, 널리 알려진 사실]
- 컨테이너 두 개에서 RMW를 다르게 두고 `ros2 topic pub` / `ros2 topic echo`로 교차 통신을 시험할 수 있다.

## 5. Docker 네트워크
- 사용자 정의 bridge: 컨테이너 간 통신 기본 허용 [확인]. 멀티캐스트에 대한 언급은 문서에 없다.
- 컨테이너 간 멀티캐스트: Linux에서는 보통 된다. Docker Desktop for Mac은 Linux VM 안에서 돌므로 직접 시험해야 한다 [추정].
- `network_mode: host`는 Mac이 아니라 Docker Desktop의 VM에 붙는다 [추정].
- 캡처: `nicolaka/netshoot` 같은 컨테이너를 `--net container:<이름>`으로 붙이고 `tcpdump -i any -w <파일>` [추정]

## 6. 결론 (Phase 00 방식)
1. Cyclone / Fast DDS 상호운용 관찰: amd64 Ubuntu 이미지 + OMG shape_main 두 개
2. 큰 샘플 / 부하: arm64 Ubuntu 24.04 + `cyclonedds-tools` (`ddsperf`)
3. ROS 2 (Phase 08): `ros:jazzy-ros-base` + Cyclone RMW
4. 가장 먼저 확인할 것: 컨테이너 간 멀티캐스트, shape_main 실행에 필요한 라이브러리
