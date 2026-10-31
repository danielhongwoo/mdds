# 저자 노트: 실습 환경 (chapter 01, 구현 없음)

상태: 완료

[roadmap](../roadmap.md) | chapter: 01 ([목차](../../book/README.md#목차)) | 다음: [02-multicast](02-multicast.md)

## 목표
규격을 읽기 전에 실제 DDS 트래픽을 볼 수 있는 환경을 만든다. Cyclone DDS와 Fast DDS를 Docker 컨테이너 두 개에 띄우고, 이후 chapter에서 쓸 Rust 워크스페이스 뼈대와 프로젝트 규약을 정한다. 프로토콜 코드는 쓰지 않는다.

본문의 목표는 독자가 저장소를 받아 lab만 따라 같은 환경을 띄우고, 왜 이렇게 구성했는지 설명할 수 있게 하는 것이다.

## 범위
하는 것
- Docker 기반 실행 환경 구성 (`docker/`): 벤더 이미지, compose, 고정 주소의 bridge 네트워크
- Cargo 워크스페이스 뼈대와 프로젝트 규약 확정 (규약은 저자의 작업이다. 본문에서 다루지 않는다)
- 본문: 준비물 설치 (Docker Desktop, Deno, Wireshark), 환경의 설계 이유, lab

하지 않는 것
- Rust 설치와 Rust 언어 설명. 독자가 Rust를 쓸 수 있다고 본다 ([book/README.md](../../book/README.md#누구를-위한-책인가)).
- 버전 표, 빌드 문제 대응, 검토한 다른 네트워크 방식 같은 세부 사항을 본문에 옮기지 않는다. `docs/environment/`와 `docker/README.md`로 링크한다.
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
| 교재 chapter | [book/01-environment](../../book/01-environment/README.md) |

## 비교 대상 벤더
**결정: 관찰과 상호운용 검증의 기준으로 Cyclone DDS와 Fast DDS를 쓴다. 원칙은 공개용 (오픈 소스) DDS만 쓰는 것이다.** (2026-10-30 확인)
- 왜
  1. 공개용: 둘 다 허용적 오픈 소스 라이선스다 (Fast DDS Apache-2.0, Cyclone DDS EPL-2.0 또는 EDL-1.0. 각 저장소의 LICENSE). 소스에서 빌드하고 들여다볼 수 있다. Fast DDS 라이브러리에 따로 상용판은 없고, eProsima는 유료 지원과 모니터링 도구의 Pro판을 판다.
  2. 시험 참여: 둘 다 OMG 상호운용 시험 릴리스 (omg-dds/dds-rtps v2.1.2026)에 실행 파일이 있다.
  3. ROS 2: REP 2000에서 Jazzy, Kilted 모두 `rmw_fastrtps_cpp` (기본)와 `rmw_cyclonedds_cpp`가 Tier 1이다 (2026-10 확인).
  4. 서로 독립된 두 구현: 만든 곳 (eProsima라는 회사, Eclipse 재단 프로젝트 "Eclipse Cyclone DDS")과 언어 (C++, C)가 다르다.
- 버린 대안
  - RTI Connext, CoreDX, InterCom DDS: 상용이라 소스에서 빌드할 수 없다. Connext는 ROS 2 Tier 1이고 "DDS 시장 점유율 70% 이상"을 주장하지만 (RTI 자료, 원 출처는 2014년 무렵 Embedded Market Forecasters) 공개용 원칙에 맞지 않는다.
  - OpenDDS: 오픈 소스이고 시험에도 참여하지만 REP 2000의 ROS 2 지원 목록에 없다.
  - Dust DDS, RustDDS: 오픈 소스 Rust 구현이지만 REP 2000 목록에 없고, RustDDS는 OMG 시험 릴리스에도 없다. Rust 구조의 참고 코드로만 본다 ([rust-ecosystem.md](../research/rust-ecosystem.md)).
- 버린 근거: "가장 널리 쓰인다", "인지도가 가장 높다". 구현별 사용량의 독립 통계가 없다. GitHub 별 (2026-10-30)은 Fast DDS 2940, OpenDDS 1520, Cyclone DDS 1356으로 약한 지표일 뿐이다.

## 독자가 시작하는 곳
**결정: 독자는 이 저장소를 clone해서 시작한다.** `docker/`, `captures/`, 빈 crate 세 개가 있는 워크스페이스를 그대로 쓰고, chapter 06부터 그 crate에 자기 코드를 쓰며 각 chapter의 `code/` 스냅샷과 비교한다.
- 왜: 실행 환경은 관찰 도구일 뿐 학습 대상이 아니다. 독자가 처음부터 만들게 하면 DDS와 관계없는 Docker와 Cargo 설정에 시간을 쓴다.
- 버린 대안: 빈 디렉터리에서 워크스페이스와 Docker 환경을 직접 만들게 하기. 위 이유로 버림.
- 01 본문은 워크스페이스와 crate를 다루지 않는다. 각 crate는 처음 코드를 쓰는 chapter에서 소개한다: 워크스페이스와 `mdds-cdr`는 06, `mdds-rtps`는 07, `mdds`와 I/O-free core로 나눈 이유는 08. 01에서 독자가 할 일이 없고 (빈 crate를 빌드해 볼 뿐), 역할을 미리 설명하면 쓸 때 다시 설명하게 된다.

## 틀
기능 chapter의 틀을 쓰지 않는다 ([book.md](../conventions/book.md#chapter)). 01에는 아직 캡처가 없고 (03에서 처음 만든다) 읽을 규격도 없어서 "캡처에서 보기", "규격 읽기"가 빈다.
1. 왜 이런 환경인가: 두 벤더를 같은 프로그램 (shape_main)으로 띄워 비교하고, 패킷을 직접 볼 수 있어야 한다. shape_main으로 하는 통신 (Cyclone 발행, Fast DDS 구독, 디스커버리 뒤 주기적 샘플을 단순한 sequenceDiagram으로. 옵션과 세부는 03에서), Docker가 하는 일 (이미지, 컨테이너)과 compose를 쓰는 이유를 짧게 설명한다
2. 준비물과 설치: macOS (Docker Desktop)와 Ubuntu 24.04 (Docker Engine) 두 환경을 나란히 쓴다. 2.1 Docker, 2.2 Deno, 2.3 Wireshark, 2.4 프로젝트 받기. clone한 디렉터리는 "프로젝트 디렉터리"라고 부른다 (특정 OS의 경로를 쓰지 않는다). Deno는 짧게 소개한다: TypeScript를 바로 실행하는 런타임이고, 셸 스크립트의 떨어지는 가독성 때문에 시나리오 스크립트에 썼다 ([03 노트](03-first-communication.md)). 시스템 소프트웨어 엔지니어에게 익숙하지 않을 수 있다
3. 벤더 이미지: 3.1 dds-rtps 저장소 (무엇을 위한 저장소인지, 실행 파일이 DDS 라이브러리 + IDL 생성 코드 + shape_main 소스로 만들어진다는 것만. 폴더 구성과 소스 분석은 넣지 않는다: 환경 이해에 도움이 되지 않는다), 3.2 소스에서 빌드하는 이유, 3.3 이미지 구성 (벤더별 이미지, 멀티스테이지, 진단 도구, `sleep infinity`와 `exec`), 3.4 버전 고정
4. 네트워크: bridge `dds-net`, 고정 주소 (.10 Cyclone, .11 Fast DDS), 호스트는 DDS 통신에 참여하지 않음, `captures/` 바인드 마운트. 구성도는 [docker-network.md](../../docs/environment/docker-network.md#구성도)의 Mermaid
5. 직접 실행해 보기: Lab 1-1, Lab 1-2 (lab 번호가 소절 번호를 대신한다)
6. 스스로 확인하기: 저자가 이해도를 점검한 질문 네 개 (shape_main을 쓰는 이유, 컴퓨터에 바로 설치하지 않는 이유, 대기하는 컨테이너, 고정 주소). 질문마다 "해설"을 접어 둔다. 커밋 고정과 `ldd` 결과 해석은 질문으로 넣지 않는다: 실습 진행에 필요 없는 세부 사항이고, 내용은 3.4절과 Lab 1-2에 있다

디렉터리: `book/01-environment/README.md`

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
| Lab 1-1 | 벤더 이미지 빌드와 컨테이너 실행 | `docker compose up -d --build` -> 두 컨테이너가 Up, `uname -m`이 호스트 아키텍처 (Mac은 `aarch64`) | `docker/compose.yaml`, `docker/cyclone/`, `docker/fastdds/` |
| Lab 1-2 | shape_main 확인 | `ldd`, `-h`, 버전 확인 -> `not found` 없음, 두 실행 파일의 옵션이 같고 dds-rtps README의 옵션을 모두 포함 (README에 아직 없는 옵션도 있다. [vendor-build.md](../../docs/environment/vendor-build.md#빌드-결과)) | [vendor-build.md](../../docs/environment/vendor-build.md) |

## 실행 환경
- 이미지 빌드 (소스 버전, Dockerfile 구성, 빌드 결과, 빌드 문제와 대응): [environment/vendor-build.md](../../docs/environment/vendor-build.md)
- 실행 호스트 (Mac Docker Desktop), 네트워크 구성: [environment/docker-network.md](../../docs/environment/docker-network.md)
- 명령어 위주 사용법: [docker/README.md](../../docker/README.md)

요약: 컨테이너 `cyclone` (172.28.0.10)과 `fastdds` (172.28.0.11)가 사용자 정의 bridge `dds-net`에 붙어 있다. `captures/`는 바인드 마운트로 컨테이너의 `/captures`에 붙는다.

## 확인할 것
- Ubuntu 24.04 (Docker Engine)에서 2절의 설치와 Lab 1-1, Lab 1-2를 직접 돌려 보지 않았다. 본문의 Ubuntu 절차는 Docker, Deno 공식 문서와 [docker-network.md](../../docs/environment/docker-network.md#선택-별도-x86_64-linux-머신)에 근거한다. 돌려 보고 빌드 시간과 출력이 다르면 본문을 고친다.

## 완료 기준
- Mac의 Docker Desktop (또는 어떤 Docker 호스트)에서 `docker compose up -d --build`만으로 두 벤더 컨테이너가 뜬다.
- Cargo 워크스페이스가 빌드되고 clippy, fmt를 통과한다.
- 프로젝트 규약이 확정되어 [conventions/](../conventions/README.md)와 `CLAUDE.md`에 반영되어 있다.

독자 기준 (본문)
- 독자가 저장소를 clone하고 본문의 설치와 lab만 따라 두 벤더 컨테이너를 띄우고, Lab 1-2의 기대 결과를 확인한다.
- 독자가 설계 이유를 설명할 수 있다: 벤더를 소스에서 빌드하는 이유, 주소를 고정하는 이유, 컨테이너는 대기만 하고 DDS를 `exec`로 띄우는 이유, 호스트가 DDS 통신에 참여하지 않는 이유.
- 본문의 명령과 기대 결과가 실제로 돌린 출력과 맞다.

## 진행 기록
결정과 막힌 점만 한 줄씩 적고 근거 문서로 링크한다. 세부 경위는 커밋 메시지에 둔다.
- 2026-10-28: 벤더 DDS는 OMG 배포 실행 파일 대신 이미지 안에서 소스로 빌드하고, 이미지를 벤더별로 나눔 -> [vendor-build.md](../../docs/environment/vendor-build.md)
- 2026-10-28: 실행 호스트를 Mac (arm64) Docker Desktop으로 정함. x86_64 Linux 머신은 선택 -> [docker-network.md](../../docs/environment/docker-network.md#실행-호스트-mac-docker-desktop)
- 2026-10-28: Cyclone을 태그 11.0.1이 아닌 master `f2aaccb4`로 고정 (11.0.1로는 shape_main C 언어 구현의 링크 실패) -> [vendor-build.md](../../docs/environment/vendor-build.md#cyclone을-master-커밋으로-고정하는-이유)
- 2026-10-28: 이미지 빌드 완료 (Mac에서 약 3분) -> [vendor-build.md "빌드 결과"](../../docs/environment/vendor-build.md#빌드-결과)
- 2026-10-28: 실행 환경 문서를 노트에서 `docs/environment/`로 분리 (이후 chapter에서도 계속 쓰므로)
- 2026-10-29: 프로젝트 규약을 노트에서 `conventions/` (Rust, TypeScript, Markdown, 캡처)로 분리 (모든 chapter에서 쓰므로). `editcap` 기준 10MB -> [conventions](../conventions/README.md)
- 2026-10-30: 스스로 확인하기를 둠 (저자 이해도 점검에서 나온 질문) -> [틀](#틀)
- 2026-10-30: 워크스페이스 절을 01에서 빼고 crate를 처음 쓰는 chapter (06, 07, 08)로 넘김 -> [독자가 시작하는 곳](#독자가-시작하는-곳)
- 2026-10-30: 비교 대상 벤더를 고른 근거를 기록 -> [비교 대상 벤더](#비교-대상-벤더)
- 2026-10-30: 본문 초안. lab 명령을 다시 돌려 출력을 맞추고, 캐시 없는 빌드 시간 (4분 50초)을 반영 -> [book/01-environment](../../book/01-environment/README.md)
- 2026-10-30: 독자는 저장소를 clone해서 시작하고, 본문은 기능 chapter 틀 대신 위 "틀"을 쓴다 -> [독자가 시작하는 곳](#독자가-시작하는-곳), [틀](#틀)
- 2026-10-31: 3.1 dds-rtps 저장소 추가. 고정 커밋에 Rust shape_main (srcRs/, RustDDS/)도 있어 "구현은 두 개"였던 용어집 문장을 고침
