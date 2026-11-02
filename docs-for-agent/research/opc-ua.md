# 조사 노트: OPC UA와 DDS 비교

[roadmap](../roadmap.md) | 사용처: 없음 (참고용) | 함께 보기: [middleware-comparison.md](middleware-comparison.md), [chapter 0 노트](../notes/00-introduction.md)

확인일: 2026-10-30. 목적: data-centric에 가장 가까운 다른 산업 표준인 OPC UA를 DDS와 비교해 둔다. chapter 0 본문에는 넣지 않는다 (분량). 표시는 [확인] (원문에서 읽음), [추정] (기억, 검색 요약, 간접 근거), [벤더 주장].

## 1. 요약
- OPC UA는 OPC Foundation이 만드는 산업 자동화용 통신 표준이고 IEC 62541로도 나온다. 중심은 **서버의 주소 공간 (address space)** 이다. 서버가 장비를 타입이 있는 노드의 그래프로 드러내고, 클라이언트가 노드의 현재 값을 읽거나 구독한다.
- data-centric 기준 ([chapter 0 2.1절](../../book/00-introduction/README.md))으로 보면 데이터에 이름을 붙이고, 타입을 알고, 대상 (노드)마다 현재 값을 들고 있다는 점은 DDS와 같다. 다른 점은 **값을 들고 있는 것이 서버**라는 것이다. DDS는 같은 일을 서버 없이 참가자들이 나눠 한다.
- 1.04 무렵 (2018년경) PubSub (Part 14)이 더해져 서버 없이 UDP 멀티캐스트나 MQTT 브로커로 값을 내보낼 수 있다 (시기는 [추정]). 다만 DDS의 SEDP처럼 writer와 reader를 자동으로 짝짓는 일반 메커니즘은 아니다 (4절).
- 두 표준을 잇는 gateway 규격이 OMG에 있다 (DDS-OPCUA 1.0, 2020).

## 2. 비교표
| 항목 | DDS | OPC UA |
|---|---|---|
| 표준 주체 | OMG (DDS 1.4, DDSI-RTPS 2.5) | OPC Foundation (OPC 10000-x, 1.05.x), IEC 62541 [확인] |
| 주된 분야 | 로보틱스, 국방, 차량, 분산 제어 | 공장 자동화, 프로세스 산업, 에너지 [확인, R1] |
| 기본 구조 | 브로커 없는 peer-to-peer pub/sub | 클라이언트-서버. PubSub은 나중에 더함 (Part 14) |
| 데이터 모델 | 토픽 = 이름 + 타입. 키로 인스턴스 구분 | 주소 공간: 노드와 reference의 그래프. 노드는 NodeId로 구분하고 attribute를 가짐 (Part 3) [확인, R3]. Variable 노드의 Value에 StatusCode와 timestamp가 붙음 [추정] |
| 타입 | IDL, XTypes (TypeObject) | DataType, VariableType, ObjectType. 분야별 companion specification (OPC Foundation은 150개 이상이라고 함) [벤더 주장, R1] |
| 대상 구분 | 키 필드 -> 인스턴스 | 노드 하나가 대상 하나 (NodeId) |
| 늦게 온 쪽에 지금 값 | DURABILITY (TRANSIENT_LOCAL 등) + HISTORY | 서버가 현재 값을 들고 있어 Read로 바로 읽음. MonitoredItem을 만들면 첫 값을 곧바로 보냄 [확인, R5] |
| 구독 | 매칭된 writer가 reader에게 직접 보냄 | Subscription과 MonitoredItem. sampling interval, queueSize, discardOldest, deadband, publishing interval [확인, R5, R6] |
| 디스커버리 | SPDP + SEDP (내장, 서버 없음) | 서버 찾기: LDS (호스트마다), LDS-ME (mDNS로 서브넷), FindServers / FindServersOnNetwork, GetEndpoints [확인, R7]. 데이터 항목은 클라이언트가 주소 공간을 browse해서 찾음 |
| 단일 장애점 | 규격이 두지 않음 (RTPS 2.5 Sec. 7.2) | 서버가 데이터의 중심. 그래서 서버 redundancy 모드 (Transparent, Cold, Warm, Hot, HotAndMirrored)와 ServiceLevel로 클라이언트가 건강한 서버를 고른다 [확인, R8] |
| QoS | RELIABILITY, DURABILITY, HISTORY, DEADLINE, LIVELINESS, OWNERSHIP 등 | 같은 묶음은 없음. 대응: TCP 위 세션 (신뢰성), queueSize (history), Subscription의 keep-alive / lifetime count (상대가 살아 있는지), Historical Access (Part 11) [확인, R6, R9]. DEADLINE, OWNERSHIP에 해당하는 것은 찾지 못함 [추정] |
| 이력 | HISTORY (메모리), 규격 밖 persistence 서비스 | Historical Access (Part 11): 서버에 지난 값을 질의 [확인, R9] |
| 전송 | UDP (규격 PSM). 구현마다 공유 메모리, TCP | 클라이언트-서버: opc.tcp, HTTPS, WebSocket. 인코딩 UA Binary, XML, JSON [확인, R10]. PubSub: UDP (UADP), Ethernet, MQTT, AMQP [확인, R11] |
| 보안 | DDS-Security (별도 규격, 플러그인) | 핵심 규격에 내장 (Part 2): application instance certificate, SecurityPolicy, security mode, 사용자 인증 [확인, R12]. PubSub은 Security Key Service (SKS)가 그룹 키를 나눔 [확인, R13] |
| 기능 안전 | 상용 안전 에디션 (Connext Cert 등) | OPC UA Safety (Part 15, IEC 62541-15). 목표 SIL은 자료마다 달라 미확인 |
| 실시간, 필드 | DDS-TSN (부가 규격) | OPC UA FX (Part 80 ~ 84, Field eXchange). Part 80 v1.00.03 (2025-07). 컨트롤러 사이 (C2C) 먼저, 장치 수준은 진행 중 [추정, R14] |
| 오픈 소스 구현 | Cyclone DDS, Fast DDS, OpenDDS, RustDDS 등 | open62541 (C, MPL-2.0), Eclipse Milo (Java, EPL-2.0), UA-.NETStandard (OPC Foundation, C#), node-opcua (TypeScript, MIT) [확인, R15] |
| Rust | RustDDS, dust_dds | locka99/opcua (crate `opcua`, MPL-2.0, 마지막 릴리스 0.12.0), 후속으로 보이는 `async-opcua` [추정, R16] |

## 3. data-centric 관점에서 본 차이
- 둘 다 "주소가 아니라 데이터에 이름을 붙이고, 미들웨어가 타입과 대상을 알고, 지금 값을 준다"는 점에서 data-centric이다.
- OPC UA는 이 일을 **서버 한 곳**이 한다. 클라이언트는 서버에 접속해 주소 공간을 탐색하고 필요한 노드를 구독한다. 정보 모델 (어떤 장비에 어떤 값이 어떤 구조로 있는가)이 풍부하고, 탐색 (browse)으로 처음 보는 장비를 이해할 수 있다는 것이 강점이다.
- DDS는 **global data space를 모두가 나눠 든다**. 서버가 없으니 단일 장애점이 없고, 대신 디스커버리 비용을 낸다 (chapter 0 3절). 정보 모델은 토픽과 타입 수준이고, 장비의 구조를 탐색하는 기능은 없다.
- 그래서 흔한 구도는 장비와 공장 계층의 정보 모델은 OPC UA, 실시간으로 많은 노드가 데이터를 나누는 부분은 DDS다. 2016년 OPC Foundation과 OMG의 공동 position paper가 이런 역할 나눔과 gateway를 제안했다 [추정, 본문 미확인, R17].

## 4. PubSub (Part 14)
- Publisher와 Subscriber는 느슨하게 묶이고 서로를 모를 수도 있다 [확인, R11].
- 내보내는 단위는 PublishedDataSet이고, 그 구조와 뜻은 DataSetMetaData가 설명한다 [확인, R11]. WriterGroup, DataSetWriter, DataSetReader로 구성한다 [추정].
- 전송은 브로커 없는 UDP (UADP, 멀티캐스트)와 Ethernet, 브로커를 쓰는 MQTT, AMQP (informative) [확인, R11].
- Subscriber가 메타데이터와 통신 설정 (DataSetWriterId 등)을 미리 알지 못할 때를 위한 discovery 메시지 (announce, probe)가 있다 [추정, 검색 요약]. Part 14 5.1에 Publisher가 PublishedDataSet을 광고하는 "PubSub directory"는 이후 버전 계획이라는 문장이 있었다 [확인, 현행 판에서 유효한지는 미확인]. 즉 DDS의 SEDP처럼 모든 writer와 reader를 자동으로 짝짓는 메커니즘은 아니다 [추정].

## 5. OMG DDS-OPCUA gateway
- 이름 "OPC-UA/DDS Gateway", 약어 DDS-OPCUA, 1.0, formal, 2020-01, formal/20-01-01 [확인, R18].
- 다루는 것: OPC UA built-in 타입, service set, subscription을 DDS로 매핑, gateway 모델, 구성용 XML schema와 예제 [확인, 문서 목록 기준, R18].
- 이 저장소에서는 다루지 않는다. 비교와 연동의 근거로만 둔다.

## 6. 확인하지 못한 것
- Variable의 Value, StatusCode, SourceTimestamp / ServerTimestamp 원문 (Part 3, Part 4)
- PubSub의 WriterGroup / DataSetWriter / DataSetReader 정의, UADP 인코딩, discovery 메시지의 필드 수준 동작, PubSub directory 문구가 1.05에서도 유효한지
- 클라이언트-서버 Subscription의 재전송 (Republish)으로 얻는 신뢰성의 정확한 범위
- Part 15 Safety의 목표 SIL (SIL 3 / SIL 4가 자료마다 다름)
- FX의 C2D / D2D 상태
- 2016 position paper 본문, 그 뒤 OPC Foundation의 DDS 관련 공식 입장
- open62541의 PubSub 지원 범위, Rust 구현 (locka99/opcua, async-opcua)의 유지보수 상태와 관계

## 7. 출처
- R1 OPC Foundation, OPC UA 소개 (분야, companion spec 수, 기능 목록): https://opcfoundation.org/about/opc-technologies/opc-ua/
- R2 Part 1 Overview and Concepts, 1.05.06, 2025-10-22: https://reference.opcfoundation.org/Core/Part1/v105/docs/
- R3 Part 3 Address Space Model 4.4.1: https://reference.opcfoundation.org/Core/Part3/v105/docs/4.4.1
- R4 Part 4 Services, 1.05.07, 2026-04-15: https://reference.opcfoundation.org/Core/Part4/v105/docs/
- R5 Part 4 5.13.1 MonitoredItem: https://reference.opcfoundation.org/Core/Part4/v105/docs/5.13.1
- R6 Part 4 5.14.1 Subscription (keep-alive, lifetime count): https://reference.opcfoundation.org/Core/Part4/v105/docs/5.14.1
- R7 Part 12 Discovery and Global Services, 1.05.07, 2026-04-15, 4.3 클라이언트가 서버를 찾는 과정 (LDS, LDS-ME, mDNS): https://reference.opcfoundation.org/Core/Part12/v105/docs/4.3
- R8 Part 4 6.6.2 Server redundancy: https://reference.opcfoundation.org/Core/Part4/v105/docs/6.6.2
- R9 Part 11 Historical Access, 1.05.04, 2024-10-15: https://reference.opcfoundation.org/Core/Part11/v105/docs/
- R10 Part 6 Mappings 7.1 (전송과 인코딩): https://reference.opcfoundation.org/Core/Part6/v105/docs/7.1
- R11 Part 14 PubSub 5.1: https://reference.opcfoundation.org/Core/Part14/v105/docs/5.1
- R12 Part 2 Security 4.1: https://reference.opcfoundation.org/Core/Part2/v105/docs/4.1
- R13 Part 14 8.1 Security Key Service: https://reference.opcfoundation.org/Core/Part14/v105/docs/8.1
- R14 OPC UA FX Part 80: https://reference.opcfoundation.org/UAFX/Part80/v100/docs
- R15 구현: https://github.com/open62541/open62541 , https://github.com/eclipse-milo/milo , https://github.com/OPCFoundation/UA-.NETStandard , https://github.com/node-opcua/node-opcua
- R16 Rust: https://github.com/locka99/opcua , https://lib.rs/crates/async-opcua
- R17 OPC Foundation / OMG position paper (2016): https://newwebsite.opcfoundation.org/wp-content/uploads/2016/04/OPCF-OPCUA-OMG-DDS-Positionspaper-final-v1.pdf
- R18 OMG DDS-OPCUA 1.0: https://www.omg.org/spec/DDS-OPCUA/
