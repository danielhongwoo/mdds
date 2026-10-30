# 저자 노트: 실습 환경 (chapter 01, 구현 없음)

상태: 완료

[roadmap](../roadmap.md) | chapter: 01 ([목차](../../book/README.md#목차)) | 다음: [02-multicast](02-multicast.md)

## 목표
규격을 읽기 전에 실제 DDS 트래픽을 볼 수 있는 환경을 만든다. Cyclone DDS와 Fast DDS를 Docker 컨테이너 두 개에 띄우고, 이후 chapter에서 쓸 Rust 워크스페이스 뼈대와 프로젝트 규약을 정한다. 프로토콜 코드는 쓰지 않는다.

## 범위
하는 것
- Docker 기반 실행 환경 구성 (`docker/`): 벤더 이미지, compose, 고정 주소의 bridge 네트워크
- Cargo 워크스페이스 뼈대와 프로젝트 규약 확정

하지 않는 것
- 멀티캐스트 동작 확인 (chapter 02), DDS 통신과 캡처 (chapter 03, 04)
- RTPS / CDR 코드 작성 (chapter 06, 07부터), 규격 정독 (chapter 05부터)
- ROS 2 환경 (chapter 12)
- `ddsperf` 등 벤더 고유 도구. 모든 시나리오는 OMG `shape_main` 하나로 만든다.

## 산출물
| 산출물 | 위치 |
|---|---|
| Docker 이미지 정의, compose 파일, 사용법 | `docker/cyclone/`, `docker/fastdds/`, `docker/compose.yaml`, `docker/README.md` |
| 빈 crate 세 개가 있는 Cargo 워크스페이스 | `Cargo.toml`, `rust-toolchain.toml`, `crates/` |
| 확정된 프로젝트 규약 | [conventions/](../conventions/README.md), `CLAUDE.md` |
| 교재 chapter | [book/](../../book/README.md) 01 |

## 작업 순서

### Step 1. 실행 호스트 준비
**Mac (Apple Silicon, arm64)의 Docker Desktop에서 빌드, 실행, 캡처를 모두 하는 것을 기본**으로 한다 (2026-10-28 결정). shape_main과 두 벤더의 DDS 라이브러리를 이미지 안에서 소스로 빌드하므로 아키텍처 제약이 없고, arm64로 네이티브 빌드된다 (에뮬레이션 없음). 상세는 [environment/docker-network.md](../../docs/environment/docker-network.md#실행-호스트-mac-docker-desktop).

- [x] Docker Desktop 설치, `docker compose version`이 동작한다.
- [x] Docker Desktop 자원: 메모리 8 GB 이상 (Fast DDS C++ 빌드용, 16 GB 권장), 디스크 여유 10 GB 이상 (빌드 중간 산출물 포함).
- [x] Wireshark (`brew install --cask wireshark`). 캡처 파일을 여는 용도로만 쓴다.
- [x] 확인: `docker run --rm ubuntu:24.04 uname -m` 이 `aarch64`를 출력한다.

### Step 2. 저장소 뼈대
- [x] `rust-toolchain.toml` (1.99.1 고정), 루트 `Cargo.toml` (workspace), `crates/mdds-cdr`, `crates/mdds-rtps`, `crates/mdds` (각각 `lib.rs`만 있는 빈 crate)
- [x] `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` 통과
- [x] `docker/`, `captures/` 디렉터리

### Step 3. Docker 이미지와 compose
- [x] `docker/cyclone/Dockerfile`, `docker/fastdds/Dockerfile`: 벤더마다 shape_main을 **Docker 안에서 소스로 빌드**한다. 설계는 [environment/vendor-build.md](../../docs/environment/vendor-build.md).
  - [x] 첫 빌드 (`docker compose build`)가 끝나고, 빌드 시간과 이미지 크기를 vendor-build.md "빌드 결과"에 적는다.
  - [x] 각 컨테이너에서 `ldd /usr/local/bin/<벤더>_shape` 로 `not found`가 없는지 확인한다.
  - [x] `cyclone_shape -h`, `fastdds_shape -h` 로 옵션이 README와 같은지 확인한다.
  - [x] 버전 확인: Fast DDS 쪽 원래 실행 파일 이름에 라이브러리 버전이 들어 있다 (`eprosima_fastdds-3.6.1.0_shape_main_linux`). Cyclone 쪽은 직접 컴파일하므로 `docker/README.md`의 버전 표와 이미지 라벨로 확인한다.
- [x] `docker/compose.yaml`: [docker-network.md](../../docs/environment/docker-network.md#네트워크-구성)의 설계대로.
- [x] `docker compose up -d --build` 로 두 컨테이너가 뜬다.
- [x] `docker/README.md`: 사용법 (Mac의 Docker Desktop에서 빌드와 실행), 네트워크 요약과 docker-network.md 링크, 빌드에 쓴 소스 태그와 커밋 해시.

### Step 4. 규약 확정
- [x] 프로젝트 규약을 확정해 [conventions/](../conventions/README.md)에 두고 `CLAUDE.md`와 맞춘다. Deno / TypeScript 스크립트, Mermaid 다이어그램, 캡처 메모 규칙을 넣고 `editcap` 기준을 10MB로 정했다.

## lab
규칙은 [conventions/book.md](../conventions/book.md#lab).

| lab | 제목 | 하는 일 -> 기대 결과 | 쓰는 산출물 |
|---|---|---|---|
| Lab 1-1 | 벤더 이미지 빌드와 컨테이너 띄우기 | `docker compose up -d --build` -> 두 컨테이너가 Up, `uname -m`이 호스트 아키텍처 (Mac은 `aarch64`) | `docker/compose.yaml`, `docker/cyclone/`, `docker/fastdds/` |
| Lab 1-2 | shape_main 확인 | `ldd`, `-h`, 버전 확인 -> `not found` 없음, 옵션이 dds-rtps README와 같음 | [vendor-build.md](../../docs/environment/vendor-build.md) |

## 실행 환경
- 이미지 빌드 (소스 버전, Dockerfile 구성, 빌드 결과, 빌드 문제와 대응): [environment/vendor-build.md](../../docs/environment/vendor-build.md)
- 실행 호스트 (Mac Docker Desktop), 네트워크 구성: [environment/docker-network.md](../../docs/environment/docker-network.md)
- 명령어 위주 사용법: [docker/README.md](../../docker/README.md)

요약: 컨테이너 `cyclone` (172.28.0.10)과 `fastdds` (172.28.0.11)가 사용자 정의 bridge `dds-net`에 붙어 있다. `captures/`는 바인드 마운트로 컨테이너의 `/captures`에 붙는다.

## 참고 코드
- Cyclone DDS: RTPS 상태 머신, 디스커버리 로직 읽기 좋음
- RustDDS: Rust식 구조
- dust_dds: XTypes, 워크스페이스 구성

## 완료 기준
- Mac의 Docker Desktop (또는 어떤 Docker 호스트)에서 `docker compose up -d --build`만으로 두 벤더 컨테이너가 뜬다.
- Cargo 워크스페이스가 빌드되고 clippy, fmt를 통과한다.
- 프로젝트 규약이 확정되어 [conventions/](../conventions/README.md)와 `CLAUDE.md`에 반영되어 있다.

## 진행 기록
결정과 막힌 점만 한 줄씩 적고 근거 문서로 링크한다. 세부 경위는 커밋 메시지에 둔다.
- 2026-10-28: 벤더 DDS는 OMG 배포 실행 파일 대신 이미지 안에서 소스로 빌드하고, 이미지를 벤더별로 나눔 -> [vendor-build.md](../../docs/environment/vendor-build.md)
- 2026-10-28: 실행 호스트를 Mac (arm64) Docker Desktop으로 정함. x86_64 Linux 머신은 선택 -> [docker-network.md](../../docs/environment/docker-network.md#실행-호스트-mac-docker-desktop)
- 2026-10-28: Cyclone을 태그 11.0.1이 아닌 master `f2aaccb4`로 고정 (11.0.1로는 C판 shape_main 링크 실패) -> [vendor-build.md](../../docs/environment/vendor-build.md#cyclone을-master-커밋으로-고정하는-이유)
- 2026-10-28: 이미지 빌드 완료 (Mac에서 약 3분) -> [vendor-build.md "빌드 결과"](../../docs/environment/vendor-build.md#빌드-결과)
- 2026-10-28: 실행 환경 문서를 노트에서 `docs/environment/`로 분리 (이후 chapter에서도 계속 쓰므로)
- 2026-10-29: 프로젝트 규약을 노트에서 `conventions/` (Rust, TypeScript, Markdown, 캡처)로 분리 (모든 chapter에서 쓰므로). `editcap` 기준 10MB -> [conventions](../conventions/README.md)
- 2026-10-29: phase 구조를 없애고 chapter를 유일한 진행 단위로 함 -> [conventions/book.md](../conventions/book.md)
- 2026-10-30: 독자가 직접 해 보는 실습을 chapter 안의 lab으로 나눔 -> [conventions/book.md](../conventions/book.md#lab)
- 2026-10-30: chapter 01-04를 함께 맡던 노트를 chapter마다 하나씩 나눔 -> [conventions/book.md](../conventions/book.md)
