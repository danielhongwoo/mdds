# mdds 로드맵

목표: OMG DDS 표준을 **직접 구현하며 이해**한다. 단계마다 "읽을 규격 -> 구현 -> 완료 기준(검증)"을 둔다.
최종 검증 기준은 **타 벤더(Cyclone DDS, Fast DDS)와의 상호운용**이다.

이 문서는 전체 지도다. 단계별 세부 내용과 진행 기록은 `phases/` 아래 각 파일에 둔다. 용어는 [glossary](glossary.md), 규격 요약표와 조사 결과는 [research](research/)를 본다.

최초 조사일: 2026-10-27

---

## 1. 단계 현황

상태 값: 대기 / 진행 중 / 완료

| Phase | 내용 | 상태 |
|---|---|---|
| 00 | [준비 (구현 없음, 관찰 단계)](phases/phase-00-setup.md) | 대기 |
| 01 | [DCPS 개념 모델](phases/phase-01-dcps-concepts.md) | 대기 |
| 02 | [CDR 직렬화 (XCDR1)](phases/phase-02-cdr.md) | 대기 |
| 03 | [RTPS 메시지 계층](phases/phase-03-rtps-messages.md) | 대기 |
| 04 | [Best-effort 전송 + UDP](phases/phase-04-best-effort-udp.md) | 대기 |
| 05 | [SPDP (참가자 디스커버리)](phases/phase-05-spdp.md) | 대기 |
| 06 | [Reliable 프로토콜](phases/phase-06-reliable.md) | 대기 |
| 07 | [SEDP + 매칭 (첫 상호운용)](phases/phase-07-sedp-matching.md) | 대기 |
| 08 | [ROS 2 상호운용 (권장)](phases/phase-08-ros2-interop.md) | 대기 |
| 09 | [키 인스턴스와 수명주기](phases/phase-09-instances.md) | 대기 |
| 10 | [DCPS API 완성 + OMG 상호운용 테스트](phases/phase-10-dcps-api.md) | 대기 |
| 11 | [단편화 + 성능](phases/phase-11-fragmentation-perf.md) | 대기 |
| 12 | [DDS-XTypes](phases/phase-12-xtypes.md) | 대기 |
| 13 | [IDL 4.2 컴파일러](phases/phase-13-idl-compiler.md) | 대기 |
| 14 | [DDS-Security (선택)](phases/phase-14-security.md) | 대기 |
| 15 | [기타 (선택)](phases/phase-15-extras.md) | 대기 |

---

## 2. 대상 규격 지도

| 규격 | 최신 정식판 | 역할 | 이 프로젝트에서 |
|---|---|---|---|
| DDS (DCPS) | 1.4 (2015) | API/엔티티/QoS 모델 | 핵심 |
| DDSI-RTPS | 2.5 (2022) | 와이어 프로토콜, 디스커버리, UDP 매핑 | 핵심 |
| CORBA CDR (CORBA 3.3 Part 2 Sec. 9.3) | 3.3 (2012) | 기본 직렬화 규칙 | 핵심 (XCDR1의 기반) |
| DDS-XTypes | 1.3 (2020), 1.4 beta 진행 중 | 타입 시스템, XCDR1/XCDR2, TypeObject | 후반 |
| IDL | 4.2 (2018) | 타입 정의 언어, 어노테이션 | 후반 (초기엔 derive 매크로로 대체) |
| DDS-Security | 1.2 (formal/25-03-06). 1.1은 2018 | 인증/접근제어/암호화 플러그인 | 선택 |
| DDS-RPC 1.0, DDS-XML 1.0, DDS-TSN, DDS-WEB | - | 부가 규격 | 선택 |

- 로컬 사본: `omg-specs/` (git 제외). PDF와 `pdftotext -layout` 추출본이 있다.
- 규격 PDF: https://www.omg.org/spec/DDS/1.4/PDF , https://www.omg.org/spec/DDSI-RTPS/2.5/PDF , https://www.omg.org/spec/DDS-XTypes/1.3 , https://www.omg.org/spec/IDL/4.2/PDF , https://www.omg.org/spec/DDS-SECURITY/

### 규격 간 의존 (읽는 순서)

```
DDS 1.4 Ch.2 (개념, QoS) --> DDSI-RTPS 2.5 --> DDS-XTypes 1.3 --> DDS-Security
CDR (CORBA, XTypes Sec. 7.4 XCDR1) --> DDSI-RTPS 2.5, DDS-XTypes 1.3 에서 사용
IDL 4.2 (기본 타입, @key 정도만 먼저) --> IDL 컴파일러 단계에서 본격적으로
```

### Rust 언어 매핑과 API 설계
- 공식 Rust PSM은 없다. OMG에서 "IDL4 -> Rust 언어 매핑" RFP(mars/2026-3-6)가 나와 있다. 참여 의향서(LoI) 마감은 2026-06-18, 1차 제안서 마감은 2026-08-17이었다. **아직 채택된 표준이 아니다.**
  - RFP는 "이런 표준을 만들 테니 제안해 달라"는 공모다. 이후 제안서 제출 -> 투표로 채택(beta) -> 확정 태스크포스(FTF) -> formal 순서로 진행되며, 보통 몇 년이 걸린다.
  - 범위는 IDL 4.2의 모든 구문을 Rust로 매핑하는 것(타입 매핑 중심)이다. RFP 문서에 DDS나 CORBA 언급은 없다. 즉 DDS의 Rust API(DCPS PSM) 자체를 정하는 규격이 아니다. 비슷한 선례로 IDL4 -> C++ 매핑, IDL4 -> Java 매핑이 있다.
- 이 프로젝트에서는 **API 설계를 자유롭게** 하되 DCPS PIM 개념은 그대로 따른다. Phase 13 IDL 컴파일러를 만들 때 RFP 진행 상황을 다시 확인한다.

---

## 3. 저장소 구조

### 처음 구성 (Phase 00 ~ 08)
경계를 모르는 상태에서 crate를 미리 잘게 나누면 의존 정리에 시간이 든다. 세 개로 시작하고, 경계가 분명해지면 나눈다.

```
mdds/
  crates/
    mdds-cdr        # CDR / XCDR 직렬화. 자체 trait (serde 사용 안 함: 정렬 / DHEADER / key-only 표현 불가)
    mdds-rtps       # RTPS 타입, 메시지 파서/인코더, Writer/Reader 상태 머신, 디스커버리 - I/O-free core
    mdds            # 사용자 API + UDP transport (드라이버)
  tools/
    mcast-check     # 컨테이너 사이 멀티캐스트 전달 확인 도구 (Phase 00)
    shape_main      # OMG 상호운용 테스트용 ShapeType 앱 (Phase 10)
  tests/
    fixtures/       # Phase 00에서 캡처한 pcap (정답 바이트)
  docker/           # Cyclone DDS / Fast DDS 실행 환경 (Phase 00에서 만들고 이후 계속 사용)
  docs/
    roadmap.md, glossary.md
    phases/         # 단계별 계획과 진행 기록
    research/       # 조사 노트 (규격 요약표, 생태계 조사)
    environment/    # 실행 환경 문서 (벤더 빌드, Docker 실행 호스트와 네트워크)
  omg-specs/        # OMG 규격 PDF와 텍스트 추출본 (git 제외: 재배포 조건 때문)
```

### 나중에 나눌 후보
- `mdds-derive`: derive 매크로 (proc-macro crate는 별도여야 하므로 Phase 02에서 바로 생길 수 있다)
- `mdds-discovery`, `mdds-transport`: mdds-rtps / mdds가 커지면 분리
- `mdds-xtypes` (Phase 12), `mdds-idl` (Phase 13), `mdds-security` (Phase 14)

## 4. 설계 원칙
- **I/O-free core** (I/O 없는 프로토콜 코어): 프로토콜 상태 머신이 소켓, 시계, 스레드, async 런타임을 직접 쓰지 않는다. 입력(받은 바이트, 현재 시각, 사용자 요청)을 받아 상태를 바꾸고, 출력(보낼 패킷과 목적지, 다음 타이머 시각, 이벤트)을 돌려주기만 한다. 실제 송수신과 대기는 바깥의 얇은 드라이버(tokio, mio, 테스트 하네스 등)가 맡는다.
  - 장점: 시간을 직접 주입하므로 타이머와 lease 만료를 즉시 테스트할 수 있다. 패킷 손실/재정렬/중복을 결정적으로 재현할 수 있다. 런타임에 독립적이고 fuzz 테스트가 쉽다. 규격의 상태 머신(RTPS 8.4)이 코드에 그대로 대응한다.
  - 단점: 코어와 드라이버 두 계층이 필요하다. 코어 안에서 `await`로 기다릴 수 없어서 모든 대기를 명시적인 상태와 타이머로 표현해야 한다.
  - 참고 구현: quinn-proto(QUIC), str0m(WebRTC), rustls(TLS). 커뮤니티에서는 이 패턴을 "sans-IO"라고도 부르므로 자료를 찾을 때는 이 이름으로 검색한다. 더 일반적인 설계 원칙으로는 "functional core, imperative shell"이 같은 생각이다.
  - tokio는 가장자리(transport, dcps async API)에서만 쓴다.
- **골든 바이트 테스트**: Cyclone/Fast DDS가 실제로 보낸 패킷(pcap)을 테스트 픽스처로 저장해 디코드 <-> 인코드 결과가 같은지 확인한다.
- **cargo-fuzz**: 메시지 파서와 ParameterList 파서에 적용한다.

---

## 5. 참고 자료
- 상호운용 테스트: https://github.com/omg-dds/dds-rtps (shape_main 규약: stdout 문자열 형식 고정, 플래그 -P/-S/-d/-t/-r/-b/-D/-x/-k)
- Rust 구현: https://github.com/Atostek/RustDDS , https://github.com/s2e-systems/dust-dds
- C/C++ 구현: https://github.com/eclipse-cyclonedds/cyclonedds , https://github.com/eProsima/Fast-DDS , https://github.com/OpenDDS/OpenDDS
- Wireshark RTPS 필드: https://www.wireshark.org/docs/dfref/r/rtps.html
