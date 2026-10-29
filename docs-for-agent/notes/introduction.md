# 저자 노트: 들어가며 (chapter 00, 구현 없음)

상태: 진행 중

[roadmap](../roadmap.md) | chapter: 00 ([목차](../../book/README.md#목차))

## 목표
DDS를 처음 보는 독자가 "왜 이런 것이 필요한가"와 "왜 직접 만들어 보는가"를 이해하고 chapter 01로 넘어가게 한다. 특히 브로커 없이 서로를 찾는 디스커버리가 DDS의 핵심이자 비용이라는 점을 앞에서 분명히 한다. 엔티티와 QoS의 자세한 내용은 chapter 05가 맡으므로 여기서는 큰 그림만 준다.

## 범위
하는 것
- DDS가 푸는 문제와 data-centric publish-subscribe의 뜻
- 디스커버리가 왜 중요한가 (브로커 없음, 동적 참여, 그 대가인 트래픽과 확장성 문제)
- 비슷한 미들웨어와의 비교: ZeroMQ, Zenoh, MQTT (브로커 방식의 대조군)
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
4. 다른 선택지와 비교: ZeroMQ, Zenoh, MQTT. 언제 무엇이 맞는지
5. 규격 지도와 이 책의 흐름: DCPS, RTPS, CDR, XTypes, IDL, Security와 그것을 다루는 chapter
6. 왜 직접 만드는가

디렉터리: `book/00-introduction/README.md`

## 읽을 자료
- DDS 1.4 Sec. 1 (Overview), Sec. 2.1 (Summary), Sec. 2.2.1.2 (Conceptual Outline)
- DDSI-RTPS 2.5 Sec. 7.2 (Requirements for a DDS Wire-protocol), Sec. 7.3, Sec. 8.5 (Discovery Module) 도입부
- [research/dds-dcps.md](../research/dds-dcps.md), [roadmap "대상 규격 지도"](../roadmap.md#2-대상-규격-지도)
- 이 저장소의 캡처: [captures/cyclone-fastdds/README.md](../../captures/cyclone-fastdds/README.md) (rel-cyclone-pub은 프레임 268개 중 디스커버리가 72개)
- 비교 대상의 공식 문서: ZeroMQ (zguide), Zenoh (zenoh.io), MQTT 5.0 (OASIS 규격)

## 작업 순서
- [ ] 비교 조사: ZeroMQ, Zenoh, MQTT를 같은 기준으로 정리해 `research/middleware-comparison.md`에 둔다. 기준: 브로커 유무, 디스커버리 방식, 데이터 모델 (타입, 키), QoS, 전송, 표준화 주체와 상호운용 구현 수. 원문으로 확인한 것과 추정을 구분하고 확인 날짜를 적는다.
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
- 비슷한 라이브러리 (ZeroMQ, Zenoh 등)와 비교되어 있다.

추가로 둔 것
- 독자가 data-centric (데이터와 그 상태를 공유)과 message-centric (메시지를 전달)의 차이를 한두 문장으로 말할 수 있다. DDS를 다른 미들웨어와 가르는 기준이다.
- 비교는 "DDS가 낫다"가 아니라 쓰임새별 선택 기준으로 끝난다. DDS의 약점 (복잡도, 디스커버리 트래픽, 무선과 대규모 환경)도 적는다.
- 독자가 규격 지도 (DCPS / RTPS / CDR / XTypes / IDL / Security)를 보고 각 규격을 어느 chapter에서 다루는지 찾을 수 있다.
- 표준과 상호운용이 왜 중요한지 (여러 벤더가 같은 와이어 규격으로 통신) 설명되어 있고, 그것이 이 책의 검증 기준임이 드러난다.
- 모든 사실 진술에 출처 (규격 절 번호 또는 공식 문서와 확인 날짜)가 있다.
- `book/README.md`, chapter 05와 내용이 겹치지 않는다.

## 진행 기록
- 2026-10-30: 노트 작성. chapter 00은 기능 chapter 틀을 쓰지 않고 위 "틀"의 절을 쓴다 -> [book.md](../conventions/book.md#chapter)
