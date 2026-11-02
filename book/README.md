# DDS를 직접 만들며 배우기

OMG DDS를 Rust로 처음부터 구현하면서 규격과 동작을 배우는 교재다. 참고 구현인 Cyclone DDS와 Fast DDS를 컨테이너로 실행해 실제 트래픽을 관찰하고, 규격을 읽고, 같은 기능을 하는 코드를 직접 작성한 뒤, 두 벤더와 통신할 수 있는지로 검증한다.

이 책에서 만드는 구현의 이름은 mdds (my DDS)다. 독자가 책을 따라 직접 만드는 자기만의 DDS라는 뜻이다.

본문 라이선스는 [CC BY-SA 4.0](LICENSE.md)이고, 코드 (chapter 안의 `code/` 포함)는 저장소와 같은 `MIT OR Apache-2.0`이다.

## 누구를 위한 책인가
통신 미들웨어가 안에서 어떻게 움직이는지 알고 싶은 소프트웨어 엔지니어를 위한 책이다.
- Rust 코드를 읽고 쓸 수 있다 (소유권, trait, 모듈 정도). Rust 언어 자체는 설명하지 않는다.
- IP, UDP, 멀티캐스트가 무엇인지 안다. 패킷을 Wireshark로 본 적이 있으면 좋지만 없어도 된다.
- DDS는 몰라도 된다. 이 책이 처음부터 설명한다.

## 이 책의 방식
chapter마다 같은 순서로 간다.
1. **왜 필요한가**: 이 chapter의 기능이 없으면 무엇이 안 되는지.
2. **캡처에서 보기**: 두 벤더가 주고받은 실제 패킷 ([captures](../captures/)) 에서 그 기능을 찾는다.
3. **규격 읽기**: 읽을 절을 정해 주고, 핵심 문장을 짧게 인용해 풀어 쓴다.
4. **구현**: mdds에 그 기능을 넣는다. chapter가 끝난 시점의 코드가 `code/`에 있다.
5. **벤더와 맞춰 보기**: Cyclone DDS, Fast DDS와 실제로 통신시켜 확인한다.
6. **두 벤더는 어떻게 하는가**: 같은 기능을 두 벤더가 어떻게 구현했는지 소스와 동작을 살핀다.
7. **연습 문제**: 캡처에서 보이지만 본문이 다루지 않은 것들.

직접 실행해 보는 부분은 **lab**으로 나눈다. Lab 1-1은 chapter 1의 첫 번째 lab이다. lab마다 목표, 준비, 단계, 기대 결과, 확인 질문이 있어서 무엇이 보이면 성공인지 알 수 있다.

이 책은 DDS가 가장 좋은 기술이라고 주장하지 않는다. DDS가 어떤 문제를 잘 풀고 어떤 대가를 치르는지, 어떤 경우에 다른 기술이 더 맞는지를 출처와 함께 적는다. 독자가 자기 문제에 맞는 기술을 고를 때 참고할 수 있는 객관적인 자료가 되는 것이 목표다.

벤더 DDS는 관찰 대상이자 검증 기준이다. 벤더 사용법을 가르치는 책은 아니다. 규격이 정하지 않은 벤더 기능 (정적 디스커버리, 디스커버리 서버, TCP / WAN 전송, 중계)은 chapter 19-21에서 다루는데, 이때도 사용법이 아니라 어떤 문제를 풀고 무엇을 잃는지를 본다.

## 준비물
- Docker Desktop (Mac arm64 기준. 다른 Docker 호스트도 쓸 수 있다): 벤더 DDS 두 개를 컨테이너로 실행한다.
- Deno 2: 캡처 시나리오 스크립트 (TypeScript) 실행.
- Rust: 설치되어 있다고 본다. 버전은 저장소의 `rust-toolchain.toml`이 정한다.
- Wireshark: 캡처를 열어 본다.
Rust를 뺀 나머지의 설치와 설정은 chapter 01에서 한다.

## 목차
뒤쪽 chapter의 제목과 나눔은 그 chapter의 작업을 하면서 바뀔 수 있다. 저자 노트는 chapter의 작업 계획이다 (독자는 읽지 않아도 된다). 책에 나오는 용어는 [용어 정리](glossary.md)에 모아 두었다.

| chapter | 제목 | 저자 노트 |
|---|---|---|
| 00 | [들어가며: DDS란 무엇이고 왜 직접 만드는가](00-introduction/README.md) | [00-introduction](../docs-for-agent/notes/00-introduction.md) |
| 01 | [실습 환경: 벤더 DDS 두 개를 컨테이너로](01-environment/README.md) | [01-environment](../docs-for-agent/notes/01-environment.md) |
| 02 | [멀티캐스트가 오가는 길](02-multicast/README.md) | [02-multicast](../docs-for-agent/notes/02-multicast.md) |
| 03 | 첫 DDS 통신 들여다보기 | [03-first-communication](../docs-for-agent/notes/03-first-communication.md) |
| 04 | 같은 규격, 다른 구현 | [04-vendor-differences](../docs-for-agent/notes/04-vendor-differences.md) |
| 05 | DCPS 개념 모델 | [05-dcps-concepts](../docs-for-agent/notes/05-dcps-concepts.md) |
| 06 | CDR 직렬화 | [06-cdr](../docs-for-agent/notes/06-cdr.md) |
| 07 | RTPS 메시지 | [07-rtps-messages](../docs-for-agent/notes/07-rtps-messages.md) |
| 08 | Best-effort 전송과 UDP | [08-best-effort-udp](../docs-for-agent/notes/08-best-effort-udp.md) |
| 09 | 참가자 디스커버리 (SPDP) | [09-spdp](../docs-for-agent/notes/09-spdp.md) |
| 10 | Reliable 프로토콜 | [10-reliable](../docs-for-agent/notes/10-reliable.md) |
| 11 | 엔드포인트 디스커버리와 매칭 (SEDP): 첫 상호운용 | [11-sedp-matching](../docs-for-agent/notes/11-sedp-matching.md) |
| 12 | ROS 2와 통신하기 | [12-ros2-interop](../docs-for-agent/notes/12-ros2-interop.md) |
| 13 | key 인스턴스와 수명주기 | [13-instances](../docs-for-agent/notes/13-instances.md) |
| 14 | DCPS API와 OMG 상호운용 테스트 | [14-dcps-api](../docs-for-agent/notes/14-dcps-api.md) |
| 15 | 단편화와 성능 | [15-fragmentation-perf](../docs-for-agent/notes/15-fragmentation-perf.md) |
| 16 | 타입 시스템 (XTypes) | [16-xtypes](../docs-for-agent/notes/16-xtypes.md) |
| 17 | IDL 컴파일러 | [17-idl-compiler](../docs-for-agent/notes/17-idl-compiler.md) |
| 18 | DDS-Security | [18-security](../docs-for-agent/notes/18-security.md) |
| 19 | 디스커버리를 바꾸는 벤더 기능: 정적 디스커버리와 디스커버리 서버 | [19-discovery-extensions](../docs-for-agent/notes/19-discovery-extensions.md) |
| 20 | UDP 멀티캐스트 밖으로: TCP와 WAN 전송 | [20-tcp-wan](../docs-for-agent/notes/20-tcp-wan.md) |
| 21 | 도메인과 네트워크를 잇는 중계와 브리지 | [21-bridges](../docs-for-agent/notes/21-bridges.md) |
| 22 | 그 밖의 것 (선택) | [22-extras](../docs-for-agent/notes/22-extras.md) |

## chapter별 내용
chapter마다 다루는 소제목과, 뒤 chapter로 미루는 것을 적는다. 앞 chapter에서 뒤의 내용을 깊이 다루면 같은 설명을 두 번 하게 되므로, 각 chapter는 "미루는 것"을 넘지 않는다. 계획이므로 집필하면서 바뀔 수 있다. 기능 chapter (05 이후)는 모두 같은 틀 (왜 필요한가 / 캡처에서 보기 / 규격 읽기 / 구현 / 벤더와 맞춰 보기 / 두 벤더는 어떻게 하는가 / 연습 문제)을 따르고, 아래 소제목은 그 안에서 다루는 주제다.

### 00. 들어가며: DDS란 무엇이고 왜 직접 만드는가
- 1 DDS가 푸는 문제
- 2 DDS란 무엇인가: message-centric과 data-centric (누가 그 일을 하는가), 토픽과 key (상태와 사건), QoS, DCPS와 RTPS
- 3 디스커버리: 규격의 요구, SPDP와 SEDP, 그 대가
- 4 다른 선택지와 비교: MCU, 미션크리티컬, 로보틱스, DDS가 맞는 곳과 다른 선택이 나은 곳
- 5 규격 지도와 이 책의 흐름
- 6 왜 직접 만드는가
- 7 스스로 확인하기
- 미루는 것: QoS 정책 전체와 매칭 규칙 (05, 11), SPDP / SEDP의 동작과 메시지 (09, 11), 디스커버리 비용을 줄이는 벤더 기능의 동작 (19 ~ 21. 00에서는 이름과 대가만), OWNERSHIP의 자세한 규칙 (14. 00에서는 failover 예 하나만)

### 01. 실습 환경: 벤더 DDS 두 개를 컨테이너로
- 1 왜 이런 환경인가: shape_main으로 하는 통신, Docker가 하는 일, compose를 쓰는 이유
- 2 준비물과 설치: Docker Desktop, Deno, Wireshark, 저장소 clone
- 3 벤더 이미지: dds-rtps 저장소, 소스에서 빌드하는 이유, 이미지 구성 (Cyclone DDS, Fast DDS, shape_main), 버전 고정
- 4 네트워크: compose와 고정 주소의 bridge
- 5 직접 실행해 보기: Lab 1-1 벤더 이미지 빌드와 컨테이너 실행, Lab 1-2 shape_main 확인
- 6 스스로 확인하기
- 미루는 것: 멀티캐스트가 오가는 원리 (02), DDS 통신과 캡처 (03), 워크스페이스와 crate (06 ~ 08)

### 02. 멀티캐스트가 오가는 길
- 1 왜 필요한가: 상대 주소를 모를 때 찾는 방법, RTPS의 기본 멀티캐스트 주소, lab 전에 알아 둘 것, mcast-check가 하는 일
- 2 캡처에서 보기: Lab 2-1 멀티캐스트 보내고 받기, Lab 2-2 가입하지 않으면
- 3 규격 읽기: 멀티캐스트 주소와 MAC 주소, 그룹 가입 (IGMPv3), 스위치가 멀티캐스트를 전달하는 방식 (snooping, querier, flooding), 네트워크를 먼저 의심할 때
- 4 스스로 확인하기
- 미루는 것: DDS가 멀티캐스트를 쓰는 방식 (03에서 관찰, 09에서 구현), 벤더별 SPDP TTL (04), RTPS 포트 공식과 포트 공유 (08)

### 03. 첫 DDS 통신 들여다보기
- shape_main과 시나리오 스크립트 (`capture-shape-scenario.ts`)
- Lab 3-1 첫 DDS 통신
- 캡처를 읽는 법: Wireshark `rtps` 필터, 프레임 번호, 흐름도
- Lab 3-2 캡처에서 RTPS 메시지 찾기 (SPDP, SEDP, DATA, HEARTBEAT, ACKNACK), 디스커버리가 차지하는 몫
- Lab 3-3 best-effort와 비교
- 미루는 것: 메시지의 바이트 해석 (07), reliable 동작 원리 (10), 매칭 규칙 (11)

### 04. 같은 규격, 다른 구현
- Lab 4-1 방향을 바꾸면: 벤더 ID, 프로토콜 버전, 포트, lease, 벤더 전용 PID
- Lab 4-2 끝날 때 dispose
- Lab 4-3 말없이 사라진 참가자
- Lab 4-4 큰 샘플
- Lab 4-5 XCDR2
- 캡처에 남긴 질문: 이후 chapter의 숙제
- 미루는 것: 각 현상의 규격 근거와 구현. dispose (13), lease (09), 단편화 (15), XCDR2 (16)

### 05. DCPS 개념 모델
- 엔티티 계층: DomainParticipant, Publisher / Subscriber, Topic, DataWriter / DataReader
- 샘플, 뷰, 인스턴스 상태 (개요)
- QoS 정책 전체 지도: 적용 엔티티, RxO, 변경 가능 여부, 와이어 전송 여부
- RxO 호환성 검사 구현
- 적합성 프로파일과 이 책의 범위
- 미루는 것: 인스턴스 수명주기 구현 (13), Listener / WaitSet과 나머지 QoS 동작 (14)

### 06. CDR 직렬화
- 워크스페이스와 crate 세 개, `mdds-cdr`
- CDR의 정렬 규칙과 엔디안
- 캡슐화 헤더 (CDR_BE / CDR_LE, PL_CDR)
- 원시 타입, string, sequence, array, enum, union
- `CdrWriter` / `CdrReader`와 derive 매크로 (serde를 쓰지 않는 이유)
- 캡처의 ShapeType 바이트와 맞춰 보기
- 미루는 것: ParameterList 해석 (07, 09), XCDR2와 appendable / mutable 타입 (16)

### 07. RTPS 메시지
- `mdds-rtps` crate: I/O-free core
- 메시지 구조: 헤더와 서브메시지, 서브메시지마다 정하는 엔디안
- 기본 타입: GUID, SequenceNumber, SequenceNumberSet, Locator, Time
- 서브메시지: DATA, HEARTBEAT, ACKNACK, GAP, INFO_TS, INFO_DST, INFO_SRC
- Receiver 상태
- ParameterList와 모르는 것을 건너뛰는 규칙
- 캡처 전부를 파싱하고 다시 인코딩하기, fuzz
- 미루는 것: 각 서브메시지를 언제 보내는가 (08, 10), 단편화 서브메시지 (15)

### 08. Best-effort 전송과 UDP
- 상호운용을 위한 행동 요구사항 (RTPS 8.4.2)
- HistoryCache와 CacheChange
- StatelessWriter와 best-effort reader
- UDP 포트 공식, participantId, 멀티캐스트 가입
- I/O-free core (`mdds-rtps`)와 UDP 드라이버 (`mdds`) 나누기
- 미루는 것: 상대를 자동으로 찾기 (09, 이 chapter는 상대 주소를 직접 적는다), UDP 밖의 전송 (20)

### 09. 참가자 디스커버리 (SPDP)
- SPDPdiscoveredParticipantData와 PID
- 주기 공지와 로케이터
- lease와 사라진 참가자 정리
- 벤더마다 다른 공지 방식 (캡처)
- 미루는 것: 엔드포인트 정보 교환 (11), 멀티캐스트를 쓰지 않는 디스커버리와 디스커버리 서버 (19)

### 10. Reliable 프로토콜
- StatefulWriter와 ReaderProxy의 상태
- StatefulReader와 WriterProxy의 상태
- HEARTBEAT, ACKNACK, GAP 주고받기
- 타이머: heartbeatPeriod, nackResponseDelay, heartbeatResponseDelay
- 손실, 중복, 재정렬을 넣은 property test
- 미루는 것: 단편 재전송 NACK_FRAG (15), liveliness (14)

### 11. 엔드포인트 디스커버리와 매칭 (SEDP): 첫 상호운용
- SEDP 내장 엔드포인트 (reliable, transient-local)
- DiscoveredWriterData / DiscoveredReaderData와 PID
- 매칭: 토픽, 타입, RxO, 파티션
- 최소 사용자 API
- ShapesDemo의 Square 토픽으로 두 벤더와 양방향 통신
- 미루는 것: key 인스턴스 관리 (13), 타입 정보 교환 PID_TYPE_INFORMATION (16), SEDP를 설정으로 대신하는 정적 디스커버리 (19)

### 12. ROS 2와 통신하기
- ROS 2의 토픽 이름과 타입 이름 규칙
- ROS 2의 기본 QoS
- talker / listener와 양방향 통신 (Fast DDS, Cyclone DDS RMW)
- 미루는 것: ROS 2에서 쓰는 Discovery Server (19), Zenoh 브리지 (21)

### 13. key 인스턴스와 수명주기
- 인스턴스와 KeyHash
- register / unregister / dispose와 STATUS_INFO
- read / take와 SampleInfo
- HISTORY, RESOURCE_LIMITS
- DURABILITY TRANSIENT_LOCAL: 늦게 들어온 reader

### 14. DCPS API와 OMG 상호운용 테스트
- 엔티티 계층 완성과 QoS 변경 규칙
- Status, Listener, Condition, WaitSet
- DEADLINE, LIFESPAN, LIVELINESS, OWNERSHIP
- 내장 토픽
- `tools/shape_main`과 OMG 상호운용 테스트

### 15. 단편화와 성능
- DATA_FRAG, HEARTBEAT_FRAG, NACK_FRAG
- 큰 샘플 상호운용 (1MB)
- 지연과 처리량 측정

### 16. 타입 시스템 (XTypes)
- 확장성: final, appendable, mutable
- XCDR2: DHEADER, EMHEADER, @optional, memberId
- TypeObject와 디스커버리 연동 (TypeInformation, TypeLookup)
- 할당 가능성과 TypeConsistencyEnforcement
- Dynamic API

### 17. IDL 컴파일러
- IDL 4.2의 어휘, 전처리, 이름과 스코프
- 데이터 타입과 어노테이션
- Rust 코드와 TypeObject 생성
- IDL4 -> Rust 언어 매핑 RFP

### 18. DDS-Security (선택)
- 플러그인 구조: 인증, 접근 제어, 암호화
- 와이어 보안 서브메시지
- 인증 핸드셰이크와 키 교환
- Governance / Permissions
- 보안 디스커버리

### 19. 디스커버리를 바꾸는 벤더 기능: 정적 디스커버리와 디스커버리 서버
- 표준 디스커버리가 막히는 곳: 규모, 멀티캐스트가 없는 네트워크
- 프로토콜은 그대로, 주소만 바꾸기: 유니캐스트 peer (Cyclone, Fast DDS)
- Lab 19-1 멀티캐스트 없이 표준 디스커버리
- 정적 디스커버리: Fast DDS STATIC EDP, RTI Connext Micro DPSE (비교)
- Lab 19-2 정적 디스커버리
- 중앙 디스커버리 서버: Fast DDS Discovery Server, RTI Cloud Discovery Service (비교)
- Lab 19-3 Discovery Server
- Lab 19-4 서버가 죽으면: 단일 장애점과 backup 서버
- 무엇을 잃는가: 동적 참여, 벤더 사이 상호운용. 표준화 상태 (OMG 이슈)
- 미루는 것: 멀티캐스트가 없는 네트워크 너머의 전송 (20), 네트워크를 잇는 중계 (21)

### 20. UDP 멀티캐스트 밖으로: TCP와 WAN 전송
- 규격이 정한 전송: UDP PSM과 로케이터
- NAT와 conntrack이 디스커버리를 막는 이유
- Lab 20-1 NAT 너머에서는
- Fast DDS TCP transport, Cyclone DDS의 TCP
- Lab 20-2 TCP로 넘기
- Lab 20-3 TCP를 모르는 상대
- RTI Real-Time WAN Transport (비교)
- Lab 20-4 나쁜 링크: 지연과 손실
- 미루는 것: 중계로 네트워크 잇기 (21), 보안 채널 (18)

### 21. 도메인과 네트워크를 잇는 중계와 브리지
- 중계는 보통의 참가자다: 한쪽은 reader, 한쪽은 writer
- DDS <-> DDS: eProsima DDS Router, RTI Routing Service (비교)
- Lab 21-1 도메인 잇기
- DDS <-> Zenoh: zenoh-bridge-dds, zenoh-bridge-ros2dds
- Lab 21-2 섬 잇기
- 중계가 바꾸는 것: GUID, 타임스탬프, 구간 단위 reliable, TRANSIENT_LOCAL, OWNERSHIP
- Lab 21-3 중계가 바꾸는 것
- Lab 21-4 mdds를 한쪽에

### 22. 그 밖의 것 (선택)
- ContentFilteredTopic, QueryCondition
- DDS-XML QoS 프로파일
- DDS-RPC
- 공유 메모리 전송

## 저자를 위한 안내
chapter를 쓰는 규칙 (틀, 번호, 코드 스냅샷, 언어, 라이선스)은 [conventions/book.md](../docs-for-agent/conventions/book.md)에 있다. 작업 계획과 결정은 [docs-for-agent/notes/](../docs-for-agent/notes/)에 두고, 이 디렉터리에는 독자용 본문만 둔다.
