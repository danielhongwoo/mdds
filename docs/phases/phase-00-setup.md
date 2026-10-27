# Phase 00 - 준비 (구현 없음, 관찰 단계)

상태: 대기

[roadmap](../roadmap.md) | 다음: [Phase 01](phase-01-dcps-concepts.md)

## 목표
규격을 읽기 전에 실제 DDS 트래픽을 눈으로 본다. 이때 확보한 캡처가 이후 단계의 테스트 데이터(정답 바이트)가 된다. 프로토콜 코드는 쓰지 않는다.

## 실행 환경: Docker (기본)
상세 조사 결과: [research/docker-env.md](../research/docker-env.md)

- **주 도구: OMG 상호운용 테스트의 `shape_main`** (릴리스 v2.1.2026)
  - `eclipse_cyclone-11.0.1_shape_main_linux`, `eprosima_fastdds-3.6.1_shape_main_linux`
  - Phase 10의 공식 상호운용 테스트와 같은 도구이고, 토픽도 같은 ShapeType이다. GUI 없이 CLI로 돈다.
  - x86-64 실행 파일이라 Apple Silicon에서는 `--platform linux/amd64`로 실행한다 (Docker Desktop의 에뮬레이션).
- **보조: Ubuntu 24.04 `cyclonedds-tools`** (arm64 네이티브, 0.10.4): `ddsperf`로 큰 샘플과 부하 트래픽을 만든다. shape_main의 Cyclone (11.0.1)보다 오래된 버전이라는 점에 주의한다.
- **ROS 2 (Phase 08용)**: `ros:jazzy-ros-base` (arm64) + `ros-jazzy-rmw-cyclonedds-cpp`
- **네트워크**: 사용자 정의 bridge 네트워크. 컨테이너 사이 멀티캐스트가 되는지 가장 먼저 확인한다.
- **캡처**: 사이드카 컨테이너를 `--net container:<이름>`으로 붙여 `tcpdump -w`로 저장하고, Mac의 Wireshark로 연다.
- 구성 파일은 `docker/` 아래에 둔다. 외부 실행 파일은 저장소에 넣지 않고 이미지 빌드 때 내려받는다.

## 할 일
- [ ] `docker/` 구성: shape_main 이미지 (amd64), cyclonedds-tools 이미지, 캡처 사이드카, compose 파일
- [ ] 두 컨테이너 사이 멀티캐스트 확인 (239.255.0.1:7400)
- [ ] Cyclone shape_main <-> Fast DDS shape_main으로 Square 토픽 통신 확인 (양방향)
- [ ] 시나리오별 캡처를 `tests/fixtures/*.pcapng`로 저장
  - [ ] best-effort
  - [ ] reliable
  - [ ] keyed 토픽의 dispose
  - [ ] 참가자 강제 종료 (lease 만료)
  - [ ] 큰 샘플 (DATA_FRAG). shape_main에 크기 옵션이 있는지 확인하고, 없으면 `ddsperf`를 쓴다.
- [ ] 각 캡처에서 SPDP / SEDP / DATA / HEARTBEAT / ACKNACK를 식별하고 진행 기록에 메모한다.
- [ ] Cargo 워크스페이스 뼈대 (`mdds-cdr`, `mdds-rtps`, `mdds` 빈 crate)
- [ ] 프로젝트 규약 확정 (아래)

## 프로젝트 규약 (제안, 확정 시 이 절을 갱신)
- 라이선스: `MIT OR Apache-2.0` 이중 라이선스 (2026-10-28 결정). 파일: `LICENSE-MIT`, `LICENSE-APACHE`
  - Rust 생태계 표준 관례 (Rust 자체, tokio, serde 등). 사용자가 둘 중 하나를 고른다.
  - Apache-2.0은 특허 허락 조항이 있고, MIT는 GPLv2 프로젝트와도 섞어 쓸 수 있다.
  - 참고 구현 중 Cyclone DDS는 EPL-2.0이다. 읽고 배우는 것은 괜찮지만 코드를 그대로 옮겨 오지 않는다.
- Rust: edition 2024. 시작 시점의 stable 버전을 `rust-toolchain.toml`에 고정하고 그 버전을 MSRV로 삼는다.
- 형식과 정적 검사: `rustfmt` 기본 설정, `cargo clippy -- -D warnings` 통과
- 테스트 배치: 단위 테스트는 모듈 안 `#[cfg(test)]`, pcap 기반 골든 테스트와 상호운용 테스트는 `tests/`
- 코드 주석에 규격 근거를 적는다. 형식: `// RTPS 2.5 Sec. 9.4.5.3`
- 문서 / 커밋 / 주석은 ASCII와 한글만 쓴다 (전역 지침).
- CI: 원격 저장소를 연결하면 GitHub Actions로 fmt, clippy, test를 돌린다.
- 프로젝트 `CLAUDE.md`: 위 규약과 문서 갱신 규칙을 적는다.

## 참고 코드
- Cyclone DDS: RTPS 상태 머신, 디스커버리 로직 읽기 좋음
- RustDDS: Rust식 구조
- dust_dds: XTypes, 워크스페이스 구성

## 완료 기준
- Docker 환경 하나로 Cyclone <-> Fast DDS 통신을 재현할 수 있다 (명령 한두 줄).
- 위 시나리오의 캡처가 저장소에 들어 있고, 주요 서브메시지를 Wireshark에서 식별할 수 있다.
- 프로젝트 규약이 확정되어 이 문서와 `CLAUDE.md`에 반영되어 있다.

## 진행 기록
- (진행하면서 배운 점, 결정 사항, 막힌 점을 날짜와 함께 적는다)
