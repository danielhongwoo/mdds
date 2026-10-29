# 조사 노트: Docker로 Cyclone DDS / Fast DDS 실행

[roadmap](../roadmap.md) | [1-4장 노트](../notes/lab-setup.md)

- 조사일: 2026-10-28
- 환경 전제 (조사 당시): Apple Silicon Mac (arm64) + Docker Desktop. 현재 실행 환경은 [environment/docker-network.md](../../docs/environment/docker-network.md)를 본다. GUI 없이 CLI 도구만 사용
- 표시: [확인] 페이지나 파일에서 직접 확인, [추정] 미확인

## 1. OMG 상호운용 테스트의 shape_main [확인]
- 릴리스: https://github.com/omg-dds/dds-rtps/releases/tag/v2.1.2026 (2026-02-27)
- 첨부: connext_dds-7.7.0, dust_dds-0.16.1, eclipse_cyclone-11.0.1, eprosima_fastdds-3.6.1, hdds-1.4.8, intercom_dds-4.3.1, opendds-3.35.0-dev, toc_coredx_dds-6.15.3, zzdds-0.3.2 (각각 `<이름>_shape_main_linux.zip`)
- Cyclone, Fast DDS 실행 파일은 zip 하나에 파일 하나이고, `file` 결과는 **ELF 64-bit x86-64, 동적 링크** (2026-10-28 확인)
  - x86_64 Linux 머신에서는 네이티브로 돈다. Apple Silicon에서 돌리려면 `--platform linux/amd64` 에뮬레이션이 필요하다.
  - 동적 링크라 실행에 필요한 공유 라이브러리를 이미지에 맞춰야 한다. `ldd`로 확인한다 [추정].
- 사용법 (README 2026-10-28 확인): -P / -S (발행 / 구독), -d (도메인), -t (토픽), -c (색), -p (파티션), -r / -b (reliable / best-effort), -D [v|l|t|p] (durability), -k (history depth), -x [1|2] (XCDR / XCDR2), -z (도형 크기 값, 페이로드 크기 아님), `--additional-payload-size <bytes>` (샘플에 바이트 추가, 큰 샘플용), `--final-instance-state [u|d]` (종료 전 unregister / dispose), `--num-iterations <n>`, `--write-period <ms>` (기본 33), `--periodic-announcement <ms>`
- 소스 `srcCxx/shape_main.cxx`는 SIGINT 핸들러만 설치한다. SIGTERM으로는 정리 없이 죽는다 (1회차 검토에서 확인).
- 실행 파일 이름이 15자를 넘어 `pkill shape_main`이 걸리지 않는다. 15자 이하의 짧은 이름으로 복사하고 `pkill -x`를 쓴다.
- Fast DDS 실행 파일 의존성: libstdc++ (GLIBCXX_3.4.30), zlib, bzip2, glibc 2.34 이상. Ubuntu 22.04에서 충족된다 (실행은 1장 환경에서 확인).
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
- `network_mode: host`는 Docker Desktop 4.34부터 설정으로 켜는 선택 기능이다 (https://docs.docker.com/engine/network/drivers/host/). Mac이 아니라 VM의 네트워크에 붙는지는 공식 문서에 명시가 없다 [추정].
- 캡처: `nicolaka/netshoot` 같은 컨테이너를 `--net container:<이름>`으로 붙이고 `tcpdump -i any -w <파일>` [추정]

## 6. lease 기본값 [확인]
- Cyclone: `Discovery/LeaseDuration` 10초, SPDP 주기는 lease의 약 80% (8초). 멀티캐스트 발견 시 `Discovery/ParticipantIndex`가 `none`과 같아 유니캐스트 포트를 임의로 고른다. https://cyclonedds.io/docs/cyclonedds/latest/config/config_file_reference.html
- Fast DDS: lease 20초, 공지 주기 3초. https://fast-dds.docs.eprosima.com/en/latest/fastdds/discovery/general_disc_settings.html

## 7. 결론 (채택한 방식)
- 2026-10-28 변경: OMG 배포 실행 파일 대신 **Docker 안에서 소스로 빌드**한다. 위 1절의 실행 파일 조사 (x86-64, 의존 라이브러리)는 배경 정보로만 남긴다. 빌드 설계는 [environment/vendor-build.md](../../docs/environment/vendor-build.md).
  - 빌드 파일 위치 (확인함): Cyclone은 `srcC/` (C판. 릴리스 태그 v2.1.2026에는 없고 2026-04 이후 master에만 있다), Fast DDS는 `srcCxx/fast-dds-cmake/CMakeLists.txt` (C++판, `fastddsgen`). 상세는 vendor-build.md
  - 버전: Cyclone 11.0.1, Fast DDS v3.6.1 (Fast-CDR v2.3.5, foonathan_memory_vendor v1.4.1, Fast-DDS-Gen v4.3.0. Fast DDS의 `fastdds.repos` 기준)
0. 실행 호스트는 별도 x86_64 Linux 머신 + Docker Engine이다 (빌드 부하 때문). Mac은 편집, 커밋, Wireshark 용도다. 같은 Dockerfile이 Mac (arm64)에서도 네이티브로 빌드된다.
1. 모든 시나리오를 shape_main 이미지 하나 (Cyclone, Fast DDS 실행 파일 두 개)로 만든다. 큰 샘플은 `--additional-payload-size`로 만들므로 `ddsperf`와 arm64 이미지는 필요 없다.
2. ROS 2 (12장): `ros:jazzy-ros-base` + Cyclone RMW
3. 가장 먼저 확인할 것: 컨테이너 간 멀티캐스트, shape_main 실행에 필요한 라이브러리
4. 상세 계획: [notes/lab-setup.md](../notes/lab-setup.md)
