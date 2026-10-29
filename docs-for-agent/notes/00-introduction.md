# 저자 노트: 들어가며 (chapter 00, 구현 없음)

상태: 진행 중

[roadmap](../roadmap.md) | chapter: 00 ([목차](../../book/README.md#목차))

## 목표
DDS를 처음 보는 독자가 "왜 이런 것이 필요한가"와 "왜 직접 만들어 보는가"를 이해하고 chapter 01로 넘어가게 한다. 특히 브로커 없이 서로를 찾는 디스커버리가 DDS의 핵심이자 비용이라는 점을 앞에서 분명히 한다. 엔티티와 QoS의 자세한 내용은 chapter 05가 맡으므로 여기서는 큰 그림만 준다.

## 범위
하는 것
- DDS가 푸는 문제와 data-centric publish-subscribe의 뜻
- 디스커버리가 왜 중요한가 (브로커 없음, 동적 참여, 그 대가인 트래픽과 확장성 문제)
- 비슷한 미들웨어와의 비교: 임베디드, 미션크리티컬, 로보틱스 관점에서 ("비교 대상과 기준")
- OMG 규격 지도와 이 책의 chapter 흐름
- 왜 직접 만드는가

하지 않는 것
- `book/README.md`에 이미 있는 내용 (누구를 위한 책인가, 이 책의 방식, 준비물)을 되풀이하지 않는다. 필요하면 링크한다.
- 엔티티, QoS 정책 목록, RxO (chapter 05), 와이어 형식 (chapter 07 이후)
- 코드와 lab. 환경 준비는 chapter 01부터다.

## 틀
chapter 00은 기능 chapter의 틀 (왜 필요한가 / 캡처에서 보기 / ... / 연습 문제)을 쓰지 않는다 ([book.md](../conventions/book.md#chapter)). 절 구성 초안:
1. DDS가 푸는 문제: 여러 프로세스와 기기가 센서 값이나 명령 같은 데이터를 실시간으로 나누는 상황
2. DDS란 무엇인가: data-centric publish-subscribe, global data space, 토픽과 키, QoS 계약
3. 디스커버리: 브로커 없이 서로를 찾는 방법과 그 비용. 이 저장소의 캡처에서 디스커버리가 차지하는 비중을 예로 든다
4. 다른 선택지와 비교: 임베디드, 미션크리티컬, 로보틱스에서 각각 무엇이 맞는지
5. 규격 지도와 이 책의 흐름: DCPS, RTPS, CDR, XTypes, IDL, Security와 그것을 다루는 chapter
6. 왜 직접 만드는가

디렉터리: `book/00-introduction/README.md`

## 비교 대상과 기준
**결정: 비교 대상은 아래 선정 기준을 모두 만족하는 것 가운데, 서로 다른 설계 선택을 하나씩 대표하는 것으로 고른다.**
- 선정 기준
  1. 같은 문제를 푼다: 프로세스나 기기 사이에서 데이터를 publish-subscribe 방식으로 나눈다.
  2. 임베디드, 미션크리티컬, 로보틱스 가운데 하나 이상에서 실제로 쓰인다 (출처로 확인).
  3. 규격이나 소스가 공개되어 있어 비교 내용을 원문으로 확인할 수 있다.
  4. 다른 후보와 겹치지 않는 설계 선택을 대표한다 (브로커 유무, message-centric / data-centric, 표준 / 단일 구현).
- 대상과 대표하는 설계 선택

| 대상 | 대표하는 것 | 주로 거론되는 분야 (조사로 확인) |
|---|---|---|
| ZeroMQ | 브로커 없는 메시지 라이브러리. 통신 패턴만 주고 디스커버리, 데이터 모델, QoS는 응용이 만든다 | 범용. 분야별 공식 사례는 찾지 못함 |
| Zenoh | 브로커 없는 pub/sub + 저장 + 질의. 무선, 대규모, 마이크로컨트롤러까지 겨냥 | 로보틱스 (ROS 2 rmw), 임베디드 (zenoh-pico) |
| MQTT | 브로커 방식의 대조군. 표준 (OASIS)이고 제약 기기에 맞춤 | 임베디드, IoT (MQTT-SN은 센서망) |
| SOME/IP | 서비스 지향 통신. 차량 안 ECU 사이의 표준 (AUTOSAR) | 미션크리티컬 (자동차) |
| DDS-XRCE (micro-ROS) | DDS 쪽의 제약 기기 답. 에이전트를 거쳐 DDS에 붙는다 | 임베디드, 로보틱스 |

- 처음 후보 (ZeroMQ, Zenoh)는 저자가 예로 든 것이고, MQTT는 브로커 방식의 대조군으로 더했다. 분야 관점을 넣으면서 SOME/IP (자동차)와 DDS-XRCE (제약 기기)를 더했다.
- 버린 대안: gRPC (요청-응답 RPC가 중심이라 pub/sub 비교에 맞지 않음), Kafka (데이터센터 로그 스트리밍이라 세 분야와 거리가 멂), OPC UA PubSub (산업 자동화. 지면이 허락하면 추가 후보).

**비교 기준: 공통 기준에 분야별 기준을 더한다.**
- 공통: 브로커 유무, 디스커버리 방식, 데이터 모델 (타입, 키, 상태), QoS, 전송 (UDP, TCP, 공유 메모리, 비IP), 표준화 주체와 상호운용 구현 수
- 임베디드: 코드와 메모리 크기, MCU / RTOS 지원, 동적 할당 없이 쓸 수 있는지, 비IP 전송 (시리얼, CAN 등), 좁은 대역폭과 끊기는 링크에서의 동작
- 미션크리티컬: 단일 장애점 유무, 시간 제약 표현 (deadline, latency budget, liveliness), 이중화와 장애 조치 (ownership 등), 보안 (인증, 접근 제어, 암호화), 안전 인증 (ISO 26262, DO-178C 등)을 받은 구현이 있는지, 여러 공급자와 장기 지원
- 로보틱스: ROS 2 지원 방식, 타입 정의와 타입 진화, 큰 데이터 (이미지, 점군)를 위한 공유 메모리나 zero-copy, 무선과 다수 로봇 환경에서의 디스커버리 비용

## 읽을 자료
- DDS 1.4 Sec. 1 (Overview), Sec. 2.1 (Summary), Sec. 2.2.1.2 (Conceptual Outline)
- DDSI-RTPS 2.5 Sec. 7.2 (Requirements for a DDS Wire-protocol), Sec. 7.3, Sec. 8.5 (Discovery Module) 도입부
- [research/dds-dcps.md](../research/dds-dcps.md), [roadmap "대상 규격 지도"](../roadmap.md#2-대상-규격-지도)
- 이 저장소의 캡처: [captures/cyclone-fastdds/README.md](../../captures/cyclone-fastdds/README.md) (rel-cyclone-pub은 프레임 268개 중 디스커버리가 72개)
- 비교 대상의 공식 문서: ZeroMQ (zguide), Zenoh (zenoh.io), MQTT 5.0 (OASIS 규격), SOME/IP (AUTOSAR 규격), DDS-XRCE 1.0 (OMG 규격)과 micro-ROS

## 작업 순서
- [x] 비교 조사: "비교 대상과 기준"의 대상을 공통 기준과 분야별 기준으로 정리해 `research/middleware-comparison.md`에 둔다. 표의 "주로 거론되는 분야"도 출처로 확인한다. 원문으로 확인한 것과 추정을 구분하고 확인 날짜를 적는다. -> [research/middleware-comparison.md](../research/middleware-comparison.md)
- [ ] 디스커버리 비용 근거: 캡처에서 디스커버리 패킷 비중을 세고, 규모가 커질 때 문제가 되는 이유 (참가자 수에 따른 SEDP 교환, 멀티캐스트 부담)와 알려진 대안 (Fast DDS Discovery Server, Zenoh의 방식 등)을 출처와 함께 정리한다.
- [ ] 규격 사실 확인: 규격 이름, 판, 계층 관계를 `omg-specs/` 원문으로 확인한다.
- [ ] 본문 초안 (`book/00-introduction/README.md`), 목차 상태를 "초안"으로.

## 확인할 것
- 저장소 루트의 int2DDS 논문 PDF (무선 다중 기기 환경의 디스커버리 트래픽 문제)를 근거로 인용할지 (저자에게 확인). 인용하면 서지 정보와 짧은 인용만 쓰고 PDF는 저장소에 넣지 않는다.
- ROS 2가 DDS를 기본 미들웨어로 쓰다가 Zenoh 기반 rmw를 추가한 경위. 공식 출처로 확인되면 비교 절의 예로 쓴다.

## 완료 기준
저자가 정한 것
- 독자가 DDS의 필요성을 이해할 수 있다.
- 독자가 디스커버리를 왜 중요하게 다루는지 설명할 수 있다.
- 비슷한 라이브러리 (ZeroMQ, Zenoh 등)와 비교되어 있다. 임베디드, 미션크리티컬, 로보틱스 각각에서 무엇을 기준으로 고르는지 드러난다.

추가로 둔 것
- 독자가 data-centric (데이터와 그 상태를 공유)과 message-centric (메시지를 전달)의 차이를 한두 문장으로 말할 수 있다. DDS를 다른 미들웨어와 가르는 기준이다.
- 비교는 "DDS가 낫다"가 아니라 쓰임새별 선택 기준으로 끝난다. DDS의 약점 (복잡도, 디스커버리 트래픽, 무선과 대규모 환경)도 적는다.
- 독자가 규격 지도 (DCPS / RTPS / CDR / XTypes / IDL / Security)를 보고 각 규격을 어느 chapter에서 다루는지 찾을 수 있다.
- 표준과 상호운용이 왜 중요한지 (여러 벤더가 같은 와이어 규격으로 통신) 설명되어 있고, 그것이 이 책의 검증 기준임이 드러난다.
- 모든 사실 진술에 출처 (규격 절 번호 또는 공식 문서와 확인 날짜)가 있다.
- `book/README.md`, chapter 05와 내용이 겹치지 않는다.

## 진행 기록
- 2026-10-30: 노트 작성. chapter 00은 기능 chapter 틀을 쓰지 않고 위 "틀"의 절을 쓴다 -> [book.md](../conventions/book.md#chapter)
- 2026-10-30: 비교 대상 선정 기준과 분야별 (임베디드, 미션크리티컬, 로보틱스) 비교 기준을 정함. 노트 이름을 book 디렉터리와 같게 바꿈 (`00-introduction.md`) -> [비교 대상과 기준](#비교-대상과-기준), [book.md](../conventions/book.md)
- 2026-10-30: 비교 조사 완료. 핵심 근거 (RTPS 2.5 Sec. 7.2, 8.5.1, Zenoh 디스커버리 측정, ROS 2 alternative middleware report, Kilted Tier 1)는 원문으로 재확인. 남은 빈칸은 조사 노트 7절 -> [middleware-comparison.md](../research/middleware-comparison.md)
