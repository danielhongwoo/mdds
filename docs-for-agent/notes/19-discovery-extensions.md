# 저자 노트: 디스커버리를 바꾸는 벤더 기능 (chapter 19)

상태: 대기

[roadmap](../roadmap.md) | chapter: 19 ([목차](../../book/README.md#목차))

## 목표
표준 SPDP / SEDP를 벤더들이 왜, 어떻게 바꾸는지 본다. 대상은 endpoint 정보를 미리 설정해 두는 정적 디스커버리와, 디스커버리를 한곳에 모으는 중앙 디스커버리 서버다. 두 방식이 무엇을 줄이고 무엇을 잃는지 (동적 참여, 단일 장애점, 벤더 사이 상호운용)를 캡처로 확인한다. chapter 00에서 "디스커버리의 대가"로 예고한 이야기를 여기서 마무리한다.

## 범위
하는 것
- 정적 디스커버리: Fast DDS STATIC EDP (XML), RTI Connext Micro DPSE와 LBED / LBPD (비교)
- 중앙 디스커버리 서버: Fast DDS Discovery Server (server, client, super client, backup), RTI Cloud Discovery Service (비교)
- 멀티캐스트를 줄이는 설정: Cyclone `AllowMulticast`, `Discovery/Peers`, Fast DDS initial peers. 표준 프로토콜 안에서 할 수 있는 것과 프로토콜을 바꾸는 것을 나눠 보이기 위해
- 표준화 상태: OMG 이슈 DDSIRTP26-37

하지 않는 것
- SPDP / SEDP 자체의 설명과 구현 (chapter 09, 11). 이 chapter는 둘을 안다고 가정한다
- 멀티캐스트가 없는 네트워크 너머로 가는 전송 (chapter 20), 네트워크를 잇는 중계 (chapter 21)
- 벤더 설정 파일의 모든 옵션. 사용법 안내서가 아니다

## 읽을 규격과 문서
- RTPS 2.5 Sec. 8.5 (디스커버리 모듈, "small to medium scale"), Sec. 8.5.1, Sec. 7.2 (단일 장애점이 없어야 함). 벤더 기능이 이 중 무엇을 바꾸는지 대조한다
- Fast DDS 문서: Discovery Server, STATIC EDP. 출처는 [research/discovery-scaling.md 5절](../research/discovery-scaling.md#5-벤더와-ros-2의-대안)
- RTI Cloud Discovery Service, Connext Micro DPSE 문서 (같은 곳)
- OMG 이슈 DDSIRTP26-37

## 다룰 기능
| 기능 | 벤더 | 무엇을 바꾸나 | 실행 가능 | lab |
|---|---|---|---|---|
| STATIC EDP | Fast DDS | SEDP를 XML 설정으로 대신. SPDP는 그대로 | 예 (오픈 소스) | 예 |
| Discovery Server | Fast DDS | SPDP, SEDP를 서버 경유 유니캐스트로 | 예 (오픈 소스, `fastdds discovery` 도구) | 예 |
| DPSE, LBED / LBPD | RTI Connext (Micro) | endpoint (와 참가자 일부)를 응용이 선언 | 아니오 (상용) | 아니오, 문서로 비교 |
| Cloud Discovery Service | RTI Connext | SPDP만 중계, SEDP는 참가자끼리 | 아니오 (상용) | 아니오, 문서로 비교 |
| 유니캐스트 peer | Cyclone, Fast DDS | 보낼 주소만 바꾸고 프로토콜은 표준 | 예 | 예 |

## 구현 후보 (결정 전)
- mdds에 정적 endpoint 설정: SEDP 없이 상대 endpoint를 설정으로 받아 매칭한다. I/O-free core에서는 "SEDP로 받은 것"과 "설정으로 받은 것"이 같은 입력이 되게 한다. Fast DDS STATIC EDP와 상호운용이 되는지는 아래 "확인할 것"에 달렸다.
- mdds를 Fast DDS Discovery Server의 client로 붙이기: 서버 로케이터로 SPDP를 유니캐스트로 보내는 것만으로 되는지, 벤더 전용 PID가 필요한지에 달렸다.
- 둘 다 안 되면 이 chapter는 관찰 chapter (틀의 4, 5를 뺌)로 둔다.

## lab (계획)
규칙은 [conventions/book.md](../conventions/book.md#lab). 번호는 확정 전이다.

| lab | 제목 | 하는 일 -> 기대 결과 |
|---|---|---|
| Lab 19-1 | 멀티캐스트 없이 표준 디스커버리 | Cyclone `Discovery/Peers`와 Fast DDS initial peers로 유니캐스트 SPDP -> 통신은 되고, SPDP가 상대 주소로만 감 |
| Lab 19-2 | 정적 디스커버리 | Fast DDS 두 참가자를 STATIC EDP로 -> 캡처에 SEDP DATA가 없음. 설정에 없는 reader는 매칭되지 않음 |
| Lab 19-3 | Discovery Server | 서버 하나와 client 둘 -> 디스커버리가 서버를 거침, 멀티캐스트 SPDP가 없음. 같은 구성에 Cyclone 참가자를 더하면 찾지 못함 |
| Lab 19-4 | 서버가 죽으면 | 통신 중 서버를 멈춤 -> 이미 맺은 통신은 계속, 새 참가자는 못 찾음. backup 서버를 두면 어떻게 되는지 |

## 확인할 것
- Fast DDS STATIC EDP가 SPDP에 무엇을 싣는지 (static 여부를 알리는 PID나 property), 상대도 STATIC이어야 하는지
- Discovery Server의 client가 보내는 SPDP에 벤더 전용 PID가 있는지, 표준 SPDP만 하는 참가자 (Cyclone, mdds)가 client가 될 수 있는지
- Discovery Server v2의 매칭 범위 축소, super client, backup 서버 동작 (discovery-scaling.md의 (재확인) 항목)
- `fastdds discovery` 도구가 우리 Fast DDS 이미지에 들어 있는지
- RTI 기능을 평가판 라이선스로 실행해 볼 수 있는지와 그 조건 (안 되면 문서 비교만 한다)
- DPSE가 QoS 일치를 검사하지 않는다는 문서 경고의 원문

## 완료 기준
- Lab 19-1 ~ 19-4의 캡처가 `captures/`에 있고, 표준 디스커버리와 패킷 수, 바이트를 비교한 표가 있다.
- 각 방식이 무엇을 줄이고 무엇을 잃는지 (동적 참여, 단일 장애점, 벤더 사이 상호운용)를 캡처 근거로 설명할 수 있다.
- 구현 후보 둘에 대해 하거나 하지 않기로 결정하고 이유를 적었다.

## 진행 기록
- 2026-10-30: 벤더 확장을 extras 후보에서 chapter 19-21로 독립시킴 -> [conventions/book.md](../conventions/book.md#벤더-확장-chapter)
