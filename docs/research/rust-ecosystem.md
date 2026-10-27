# 조사 노트: 기존 구현과 Rust 생태계

[roadmap](../roadmap.md)

- 조사일: 2026-10-27
- 표시: [확인] 저장소 / 문서 페이지에서 확인, [요약] 검색 요약만 봄, [미확인] 확인 못 했거나 정보가 엇갈림

## 1. 순수 Rust DDS 구현

### RustDDS (Atostek)
- 저장소: https://github.com/Atostek/RustDDS (jhelovuo/RustDDS도 같은 프로젝트) [확인]
- 라이선스 Apache-2.0. crate 0.15.0 (내부 이벤트 루프를 mio 1.x로 이전) [확인]
- 지원: RTPS over UDP, 디스커버리, reliable / best-effort, HISTORY, 키 있는 / 없는 토픽, 단편화 [확인]. RTPS 버전, XTypes는 명시 없음. DDS Security 언급 없음 [확인]
- 미구현: partition, time-based filter, presentation, deadline, zero-copy, 공유 메모리 [확인]
- 구조: rtps-rs에서 출발. 코드 생성 없음. Serde 기반 제네릭 DataReader / DataWriter, 기본 CDR. 키 토픽은 `Keyed` trait. mio `Poll` API와 async API 둘 다 제공 [확인]
- 상호운용: ROS 2 데이터 교환 주장. 릴리스 노트에 Cyclone / Fast DDS 관련 수정 있음. 상호운용 결과 저장소 jhelovuo/dds-rtps [확인]
- 읽을 가치: 한 crate 안에서 Rust다운 구조를 보기 좋다.

### dust_dds (S2E Software Systems)
- 저장소: https://github.com/s2e-systems/dust-dds , crate 0.15.0. OMG 상호운용 테스트 릴리스에는 0.16.1이 올라와 있다 [확인]
- 라이선스 Apache-2.0 [확인]
- DDS 1.4와 DDSI-RTPS 2.5를 목표로 하고 "minimum DDS profile"이라고 소개 [확인]
- XTypes 지원이 가장 충실: final / appendable / mutable, TypeObject, IDL 코드 생성 (dust_dds_gen) [확인]. DDS Security 언급 없음
- 동기 / 비동기 API. Tokio 안에서는 async 필수. 마이크로컨트롤러는 Embassy. unsafe 없음 주장 [확인]
- 워크스페이스에 `interoperability_tests`와 shapes_demo가 있다 [확인]
- 읽을 가치: XTypes 구현과 워크스페이스 구성 참고

### 그 밖
- rtps-rs (Klapeyron): https://github.com/Klapeyron/rtps-rs , 마지막 릴리스 0.1.2 (2020-12), 중단 상태 [요약]
- xdds/rtps: 2016년이 마지막, 빌드 어려움 [요약]
- ros2-client: https://github.com/jhelovuo/ros2-client , RustDDS 위의 순수 Rust ROS 2 클라이언트 [요약]

## 2. C/C++ 구현의 Rust 바인딩
- cyclors (ZettaScaleLabs): https://github.com/ZettaScaleLabs/cyclors , Cyclone DDS C API 저수준 바인딩, Apache-2.0 [확인]
- cyclonedds-rs: 상위 API, 마지막 릴리스 0.6.5 (2023-01) [요약]
- rclrs, r2r, rclrust: C rcl에 링크, ROS 2 설치 필요 [요약]

## 3. 참고할 C/C++ 구현
- **Eclipse Cyclone DDS**: https://github.com/eclipse-cyclonedds/cyclonedds . EPL-2.0 / EDL-1.0. DDSI-RTPS 2.5, XTypes 1.3 (일부 제약), Security 1.1 지원 주장. `ddsperf` 포함 [확인]. RTPS 코드를 읽기에 가장 좋은 C 코드로 평가된다.
- **eProsima Fast DDS**: https://github.com/eProsima/Fast-DDS . Apache-2.0, C++. ROS 2 기본 구현 [확인]. RTPS 계층이 분리돼 있지만 코드가 크다.
- **OpenDDS**: https://github.com/OpenDDS/OpenDDS . 허용적 라이선스. DDS-RTPS 2.3 구현 [요약]
- **RTI Connext**: 상용. 코드 읽기 용도로는 쓸 수 없다. Shapes Demo는 무료 [요약]

## 4. 상호운용 테스트
- **omg-dds/dds-rtps**: https://github.com/omg-dds/dds-rtps [확인]
  - 벤더별 `shape_main` 실행 파일을 서로 붙여 시험한다. 플래그: -P, -S, -d, -t, -r / -b, -D, -x (1=XCDR, 2=XCDR2), -k
  - 실행: `python3 interoperability_report.py -P <pub> -S <sub>`, 결과는 JUnit XML
  - 결과는 `shape_main`의 stdout 문자열로 판정하므로 출력 형식을 바꾸면 안 된다.
  - 참여 방법: 자기 스택으로 shape_main을 빌드해 `<product>_shape_main_linux`로 이름 짓고 zip으로 최신 GitHub 릴리스에 첨부한 뒤, master에서 "Testing Interoperability" Action을 실행한다. GLIBC 2.31 이하에서 빌드한다.
  - 최신 릴리스 v2.1.2026 (2026-02-27): connext_dds-7.7.0, dust_dds-0.16.1, eclipse_cyclone-11.0.1, eprosima_fastdds-3.6.1, hdds-1.4.8, intercom_dds-4.3.1, opendds-3.35.0-dev, toc_coredx_dds-6.15.3, zzdds-0.3.2 [확인]
  - Cyclone, Fast DDS 실행 파일은 **x86-64 Linux ELF**다 (2026-10-28 `file`로 확인). Apple Silicon에서는 `--platform linux/amd64` 에뮬레이션이 필요하다.
- **Wireshark**: RTPS 분석기 내장, UDP에서 동작 [요약]
- **RTI Shapes Demo**: https://www.rti.com/downloads/shapes-demo (무료, RTI 라이선스) [요약]

## 5. 빌딩 블록 crate
- CDR: `cdr` (0.2.4, 2021), `cdr-encoding` (RustDDS용) 모두 고전 CDR만 지원. XCDR2를 제대로 지원하는 crate는 없다 [요약]. 직접 구현한다.
- 직렬화 일반: `speedy` [요약]
- 네트워크: `tokio`, `mio`, `socket2` (멀티캐스트 설정) [미확인]
- 버퍼: `bytes` [미확인]

## 6. 권장
- RTPS 상태 머신과 디스커버리 로직은 Cyclone DDS를 읽는다.
- Rust다운 구조는 RustDDS, XTypes와 워크스페이스 구성은 dust_dds를 본다.
- CDR / XCDR2 계층은 직접 만든다.
- 초기부터 shape_main과 Wireshark로 Cyclone / Fast DDS와 대조한다.
