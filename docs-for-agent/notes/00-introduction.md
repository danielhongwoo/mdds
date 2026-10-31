# 저자 노트: 들어가며 (chapter 00, 구현 없음)

상태: 완료

[roadmap](../roadmap.md) | chapter: 00 ([목차](../../book/README.md#목차))

## 목표
DDS를 처음 보는 독자가 맥락을 잡고 chapter 01로 넘어가게 한다. 세부 사항을 외우게 하는 것이 아니라 다음을 이해하게 한다.
- DDS가 어떤 문제를 푸는가, data-centric이 무엇인가 (그 일을 응용이 아니라 미들웨어가 한다)
- 브로커 없이 서로를 찾는 디스커버리가 DDS의 핵심이자 비용이라는 것
- 다른 기술과 비교해 DDS가 맞는 곳과 다른 선택이 나은 곳
- 이 책이 왜 DDS를 직접 만들며 배우는가

## 범위
하는 것
- DDS가 푸는 문제, message-centric과 data-centric, 토픽과 key, QoS, DCPS와 RTPS
- 디스커버리의 요구, 동작 (SPDP, SEDP), 대가와 현장의 대응
- 다른 미들웨어와 비교, 분야별 기준 (MCU, 미션크리티컬, 로보틱스), DDS가 맞는 곳
- OMG 규격 지도와 이 책의 흐름, 왜 직접 만드는가
- 스스로 확인하기

하지 않는 것
- `book/README.md`의 내용 (누구를 위한 책인가, 이 책의 방식, 준비물)을 되풀이하지 않는다. 예외 두 가지: 객관성 원칙은 첫 문단에서 짧게 밝힌다 (이 chapter의 비교가 그 원칙을 가장 직접 보여 주므로). 대상 독자는 6절 끝에서 한 문단으로 밝힌다 (왜 직접 만드는가에 이어, 누가 그 길을 따라올지).
- 엔티티 계층, QoS 정책 전체와 매칭 규칙 (chapter 05, 11), OWNERSHIP의 자세한 규칙 (chapter 14), 와이어 형식 (chapter 07 이후)
- 디스커버리 비용을 줄이는 벤더 기능의 동작 (chapter 19 ~ 21). 여기서는 이름과 대가만
- 코드와 lab, 캡처의 수치. 독자는 아직 실험 환경이 없고, 캡처는 chapter 03에서 처음 만든다.

## 틀
기능 chapter의 틀을 쓰지 않는다 ([book.md](../conventions/book.md#chapter)). 소절에 번호를 붙인다 (2.1, 2.2 ...).
1. DDS가 푸는 문제: 이동 로봇 예와 세 질문 (누가 누구에게, 무엇을, 어떻게). 2절과 3절이 이 세 질문에 답한다
2. DDS란 무엇인가
   - 2.1 message-centric과 data-centric: ZeroMQ, MQTT와 DDS. 늦게 켜진 관제 PC를 두 방식으로 그린 그림 두 개. 차이는 같은 일을 응용과 미들웨어 가운데 누가 하는가
   - 2.2 토픽, 타입, key: key는 보내는 쪽이 아니라 대상을 가리킨다. 상태와 사건
   - 2.3 QoS: 네 가지 정책의 예
   - 2.4 DCPS와 RTPS: 구현 바꾸기는 DCPS, 구현 사이 통신은 RTPS. 이 책의 검증 기준은 RTPS
3. 디스커버리: 규격의 요구 (단일 장애점 없음), SPDP와 SEDP, 대가 (규격의 동작으로만 설명)와 현장의 대응 (표준 밖)
4. 다른 선택지와 비교: 비교의 목적 (DDS가 낫다는 것이 아님), 표, MCU, 미션크리티컬 (OWNERSHIP으로 보는 failover), 로보틱스, DDS가 맞는 곳과 다른 선택이 나은 곳
5. 규격 지도와 이 책의 흐름 (규격 밖 벤더 기능 19 ~ 21, 그 밖의 것 22까지)
6. 왜 직접 만드는가: 끝에 대상 독자 (통신 미들웨어의 안을 알고 싶은 소프트웨어 엔지니어, Rust는 설명하지 않음)
7. 스스로 확인하기: 질문마다 "해설"을 접어 둔다 ([book.md](../conventions/book.md#스스로-확인하기))

디렉터리: `book/00-introduction/README.md`

## 비교 대상과 기준
**결정: 비교 대상은 아래 선정 기준을 모두 만족하는 것 가운데, 서로 다른 설계 선택을 하나씩 대표하는 것으로 고른다.**
- 선정 기준
  1. 같은 문제를 푼다: 프로세스나 기기 사이에서 데이터를 publish-subscribe 방식으로 나눈다.
  2. MCU, 미션크리티컬, 로보틱스 가운데 하나 이상에서 실제로 쓰인다 (출처로 확인).
  3. 규격이나 소스가 공개되어 있어 비교 내용을 원문으로 확인할 수 있다.
  4. 다른 후보와 겹치지 않는 설계 선택을 대표한다 (브로커 유무, message-centric / data-centric, 표준 / 단일 구현).
- 대상과 대표하는 설계 선택

| 대상 | 대표하는 것 | 주로 거론되는 분야 (조사로 확인) |
|---|---|---|
| ZeroMQ | 브로커 없는 메시지 라이브러리. 통신 패턴만 주고 디스커버리, 데이터 모델, QoS는 응용이 만든다 | 범용. 분야별 공식 사례는 찾지 못함 |
| Zenoh | 브로커 없는 pub/sub + storage + query. 무선, 대규모, 마이크로컨트롤러까지 겨냥 | 로보틱스 (ROS 2 rmw), MCU (zenoh-pico) |
| MQTT | 브로커 방식의 대조군. 표준 (OASIS)이고 제약 기기에 맞춤 | MCU, IoT (MQTT-SN은 센서망) |
| SOME/IP | 서비스 지향 통신. 차량 안 ECU 사이의 표준 (AUTOSAR) | 미션크리티컬 (자동차) |
| DDS-XRCE (micro-ROS) | DDS 쪽의 제약 기기 답. 에이전트를 거쳐 DDS에 붙는다 | MCU, 로보틱스 |

- 버린 대안: gRPC (요청-응답 RPC가 중심이라 pub/sub 비교에 맞지 않음), Kafka (데이터센터 로그 스트리밍이라 세 분야와 거리가 멂), OPC UA (산업 자동화의 data-centric 표준이지만 지면 때문에 본문에서 뺌. 비교는 [research/opc-ua.md](../research/opc-ua.md)).

**비교 기준: 공통 기준에 분야별 기준을 더한다.**
- 분야의 범위
  - MCU: 마이크로컨트롤러 위의 펌웨어 (bare-metal)나 RTOS (FreeRTOS, Zephyr, NuttX 등). 수십~수백 KB의 RAM, MMU와 범용 OS가 없는 환경이다. Linux가 도는 edge 기기 (Jetson Orin NX, Raspberry Pi 등)는 일반 Linux 호스트와 같은 조건이라 넣지 않는다. 이 범위 설명은 노트에만 두고 본문에는 쓰지 않는다.
  - 버린 표현: "임베디드". edge 기기까지 넓게 읽혀서 기준 (코드 크기, 동적 할당, RTOS)과 맞지 않는다.
- 공통: 브로커 유무, 디스커버리 방식, 데이터 모델 (타입, key, 상태), QoS, 전송 (UDP, TCP, 공유 메모리, 비IP), 표준화 주체와 상호운용 구현 수
- MCU: 코드와 메모리 크기, 지원하는 RTOS, 동적 할당 없이 쓸 수 있는지, 비IP 전송 (시리얼, CAN 등), 좁은 대역폭과 끊기는 링크에서의 동작
- 미션크리티컬: 단일 장애점 유무, 시간 제약 표현 (deadline, liveliness), 이중화와 failover (ownership 등), 보안, 안전 인증을 받은 구현이 있는지, 여러 공급자
- 로보틱스: ROS 2 지원 방식, 타입 정의와 타입 진화, 큰 데이터를 위한 공유 메모리, 무선과 다수 로봇 환경에서의 디스커버리 비용

## 읽을 자료
- DDS 1.4 Sec. 1 (Overview), Sec. 1.2 (Purpose, 적용 분야), Sec. 2.2.1.2 (Conceptual Outline), Sec. 2.2.3 (QoS), Sec. 2.2.3.9.2 (OWNERSHIP)
- DDSI-RTPS 2.5 Sec. 7.2, Sec. 8.5.1, Sec. 8.5.3.1, Sec. 8.5.4.2
- DDS-XRCE 1.0 Sec. 8.1
- 조사 노트: [middleware-comparison.md](../research/middleware-comparison.md) (비교의 근거), [discovery-scaling.md](../research/discovery-scaling.md) (디스커버리 비용과 대안), [opc-ua.md](../research/opc-ua.md) (본문 밖 비교)
- 캡처: [captures/cyclone-fastdds/README.md](../../captures/cyclone-fastdds/README.md). 저자가 디스커버리 비용을 가늠하는 데만 쓴다. 본문에는 수치를 쓰지 않는다 (범위 참고).

## 완료 기준
- 독자가 DDS의 필요성과 data-centric의 뜻을 "그 일을 누가 하는가"로 설명할 수 있다.
- 독자가 디스커버리를 왜 중요하게 다루는지 (장점이자 비용) 설명할 수 있다.
- 비슷한 기술 (ZeroMQ, Zenoh, MQTT, SOME/IP, DDS-XRCE)과 비교되어 있고, MCU, 미션크리티컬, 로보틱스 각각에서 무엇을 기준으로 고르는지, DDS가 맞는 곳과 다른 선택이 나은 곳이 드러난다.
- 비교가 객관성 원칙 ([book.md](../conventions/book.md#작성-원칙-객관성))을 지킨다: 쓰임새별 결론, DDS의 약점, 출처, 벤더 주장 표시.
- 독자가 규격 지도를 보고 각 규격과 벤더 기능을 어느 chapter에서 다루는지 찾을 수 있다.
- 모든 사실 진술에 출처가 있다 (규격 절 번호 또는 공식 문서).
- 스스로 확인하기의 질문을 이 chapter만 읽고 풀 수 있다.

## 진행 기록
- 2026-10-30: chapter 00은 기능 chapter 틀을 쓰지 않고 위 "틀"의 절을 쓴다 -> [book.md](../conventions/book.md#chapter)
- 2026-10-30: 비교 대상과 기준, 분야 이름을 "MCU"로 좁힘 -> [비교 대상과 기준](#비교-대상과-기준)
- 2026-10-30: 비교 조사와 디스커버리 비용 조사를 마침 -> [middleware-comparison.md](../research/middleware-comparison.md), [discovery-scaling.md](../research/discovery-scaling.md)
- 2026-10-30: 본문 초안 작성, 저자 리뷰로 구성을 다시 잡음 (OWNERSHIP을 4.2로, 캡처 수치 제외, 비교 그림, 스스로 확인하기, 소절 번호)
- 2026-10-30: 객관성 원칙을 정하고 첫 문단에 밝힘 -> [book.md](../conventions/book.md#작성-원칙-객관성)
