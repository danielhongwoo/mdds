# 조사 노트: DDS 디스커버리의 비용과 대안

[roadmap](../roadmap.md) | 사용처: [chapter0 노트](../notes/00-introduction.md), 이후 [chapter9](../notes/09-spdp.md), [chapter11](../notes/11-sedp-matching.md) | 함께 보기: [middleware-comparison.md](middleware-comparison.md)

확인일: 2026-10-30. 표기는 [middleware-comparison.md](middleware-comparison.md)와 같다: [확인] / [추정] / [벤더 주장] / [3rd] / (재확인). 여기에 [추론]을 더한다: 규격의 동작에서 저자가 이끌어 낸 것으로, 출처가 아니라 논리가 근거다.

---

## 1. 요약
- 표준 디스커버리 (SPDP + SEDP)는 브로커 없이 누구나 언제든 참여하게 해 주지만, 모든 참가자가 모든 참가자에게 자기와 자기 endpoint를 알려야 한다. 그래서 참가자 수가 늘면 트래픽과 상태가 대략 제곱으로 는다 [추론, 3절].
- 규격도 SPDP와 SEDP가 소~중규모용이라고 적는다 (RTPS 2.5 Sec. 8.5.1). [확인]
- 이 저장소의 캡처에서는 참가자 2개, 토픽 1개, 약 20초 동안에도 디스커버리 (SPDP + SEDP)가 바이트의 37~62%를 차지했다 (큰 샘플 시나리오 제외). [확인, 4절]
- 벤더들은 중앙 서버, 정적 설정, 유니캐스트 peer 목록, 범위 제한으로 이를 줄인다. 이 방법들은 규격 밖이라 서로 호환되지 않는다. OMG에는 이를 규격에 넣자는 이슈가 열려 있다. (재확인, 5절과 6절)

## 2. 규격이 정한 동작
- **SPDP** (RTPS 2.5 Sec. 8.5.3.1): 참가자마다 best-effort StatelessWriter 하나가 자기 정보 (SPDPdiscoveredParticipantData) 한 개를 미리 설정된 locator 목록 (유니캐스트, 멀티캐스트)으로 **주기적으로** 다시 보낸다. 주기는 leaseDuration보다 짧아야 한다. 주기적으로 보내기 때문에 참가자가 어떤 순서로 들어와도 서로를 찾는다. 새 참가자를 보면 바로 한 번 더 보내는 것은 선택이다. [확인]
- **SEDP** (Sec. 8.5.4.1, 8.5.4.2): 상대 참가자를 알면 미리 정해진 내장 endpoint (DCPSPublication, DCPSSubscription, DCPSTopic)와 매칭하고, 일반 데이터와 같은 RTPS 프로토콜로 자기 DataWriter와 DataReader의 존재와 QoS를 알린다. 이 내장 endpoint는 **reliable**이다. [확인]
- **한계** (Sec. 8.5.1): 모든 구현은 상호운용을 위해 SPDP와 SEDP를 지원해야 하고, 둘은 소~중규모 네트워크에 충분하다. 더 큰 네트워크용 프로토콜은 이후 판에 더할 수 있다고 적는다. [확인]

## 3. 왜 규모에 따라 커지는가 [추론]
참가자 수 N, 참가자당 endpoint 수 E로 놓는다.
- **SPDP**: 한 주기마다 참가자 N개가 한 번씩 보낸다. 멀티캐스트면 보내는 패킷은 N개지만 받는 쪽은 각자 N-1개를 처리한다. 멀티캐스트를 못 써서 유니캐스트 peer 목록으로 보내면 보내는 패킷이 N*(N-1)개다.
- **SEDP**: 참가자 쌍마다 reliable하게 endpoint 정보를 주고받으므로 전체 교환량은 N*(N-1)*E에 비례한다. 받은 정보는 모든 참가자가 보관한다. 규격은 토픽 관심사에 따라 거르는 방법을 정하지 않으므로, 내 토픽과 관계없는 endpoint 정보도 받는다.
- 같은 식을 Zenoh 블로그도 제시한다: n*(n-1)*(r+w). [벤더 주장] (재확인, [middleware-comparison.md 4절](middleware-comparison.md#4-디스커버리-관점-chapter-0의-핵심-근거))
- **무선**: 802.11에서 멀티캐스트는 가장 낮은 기본 속도로 나가고 MAC 수준 ACK가 없어 주기적 멀티캐스트의 비용이 커진다는 분석이 있다 (DGIST "Discovery Storm" 논문). [추정, 본문 미확인]
- 학술 쪽에서도 RTPS 디스커버리 메시지가 참가자 수의 제곱에 비례한다는 주장이 있다 (Park et al., Springer 2012, https://link.springer.com/chapter/10.1007/978-3-642-35600-1_5). [추정, 본문 미확인]

## 4. 이 저장소의 캡처에서 본 비중
조건: 참가자 2개 (Cyclone 1, Fast DDS 1), 토픽 1개, writer 1개와 reader 1개, 약 20초, 구독자 컨테이너에서 캡처. 자기가 보낸 멀티캐스트는 loopback 복사본 때문에 두 번 잡힌다 ([캡처 README](../../captures/cyclone-fastdds/README.md)). 그래서 SPDP 수치는 실제 전송보다 크다.

분류 기준: submessage의 writer / reader entityId. SPDP는 `0x000100c2/c7`, SEDP는 `0x000003c2/c7`, `0x000004c2/c7`, WLP (writer liveliness, 디스커버리가 아님)는 `0x000200c2/c7`, user는 entityKind가 내장 (0xc_)이 아닌 것.

| 캡처 | 전체 바이트 | SPDP | SEDP | WLP | user | SPDP + SEDP 비중 |
|---|---|---|---|---|---|---|
| reliable-cyclone-pub | 56,408 | 42 fr / 20,796 | 24 fr / 3,828 | 6 fr / 1,004 | 184 fr / 30,132 | 43.7% |
| reliable-fastdds-pub | 50,622 | 33 / 16,286 | 13 / 2,494 | 6 / 1,004 | 153 / 30,190 | 37.1% |
| best-effort-cyclone-pub | 37,462 | 42 / 20,884 | 11 / 2,186 | 6 / 1,004 | 98 / 12,740 | 61.6% |
| best-effort-fastdds-pub | 43,140 | 33 / 16,286 | 12 / 2,432 | 6 / 1,004 | 99 / 22,770 | 43.4% |
| dispose-on-exit | 61,064 | 46 / 22,996 | 12 / 2,340 | 17 / 2,170 | 201 / 32,910 | 41.5% |
| lease-expiry | 66,884 | 55 / 28,766 | 13 / 2,322 | 16 / 2,064 | 202 / 33,084 | 46.5% |
| large-sample | 2,917,692 | 31 / 14,982 | 12 / 2,152 | 7 / 998 | 260 / 58,648 | 0.6% |
| xcdr2 | 58,944 | 42 / 20,796 | 12 / 2,352 | 17 / 2,170 | 199 / 32,978 | 39.3% |

- 바이트의 대부분은 SPDP다 (SPDP 한 패킷이 462~550 bytes로 사용자 DATA보다 크다). SEDP는 endpoint가 2개뿐이라 작다. 3절에 따르면 참가자와 endpoint가 늘수록 SEDP 몫이 커진다.
- large-sample은 사용자 데이터가 커서 비중이 작다. 비중은 데이터 크기와 실행 시간에 크게 좌우되므로, chapter3에서 쓸 때도 "작은 시스템에서도 디스커버리가 눈에 띄는 양"이라는 정도로 쓴다.
- reliable-cyclone-pub의 72프레임 (SPDP 42 + SEDP 24 + WLP 6)은 캡처 README의 "디스커버리 72개"와 같다.
- 다시 세는 명령 (캡처 디렉터리에서):
  ```
  tshark -r <캡처>.pcap -T fields -e frame.len -e rtps.sm.wrEntityId -e rtps.sm.rdEntityId -e igmp.type
  ```
  출력의 entityId로 위 분류 기준을 적용한다.

## 5. 벤더와 ROS 2의 대안
| 방식 | 예 | 무엇을 바꾸나 | 대가 |
|---|---|---|---|
| 중앙 서버 (hub) | Fast DDS Discovery Server | 각 참가자가 client로 서버에 유니캐스트로 보내고 서버가 다시 나눠 준다. 문서가 드는 이유는 SPDP가 노드가 늘면 패킷이 크게 늘고, 멀티캐스트가 WiFi 같은 곳에서 믿을 만하지 않다는 것. (재확인) v2는 같은 토픽에 매칭되는 상대가 있는 client끼리만 이어 SEDP를 더 줄인다 [확인, 조사 결과] | 서버가 디스커버리의 중심이 된다. backup 서버 기능 (재확인). ROS 2 도구가 전체 그래프를 보려면 Super Client로 설정해야 한다 (재확인) |
| | RTI Cloud Discovery Service | 참가자 공지 (SPDP)만 중계하고 SEDP는 참가자끼리 직접 [확인, 조사 결과] | 목적은 멀티캐스트가 없는 환경 |
| 정적 설정 | Fast DDS STATIC EDP (XML), RTI Connext Micro DPSE, RTI LBED / LBPD | SEDP (또는 참가자 정보 일부)를 미리 쓴 설정으로 대신한다. SPDP는 계속 동작 [확인, 조사 결과]. DPSE는 (재확인) | 모든 endpoint를 미리 알아야 한다. 동적 참여를 잃는다. DPSE는 QoS 일치를 런타임에 검사하지 않는다 |
| 유니캐스트 peer, 멀티캐스트 축소 | Cyclone DDS `General/AllowMulticast=spdp`, `Discovery/Peers`; Fast DDS initial peers | `spdp`는 SPDP에만 멀티캐스트를 쓰고 endpoint 디스커버리와 사용자 데이터는 모두 유니캐스트로 한다 (재확인). peer 목록은 디스커버리를 보낼 주소를 정한다 | peer 목록을 관리해야 한다. 유니캐스트 SPDP는 N*(N-1) |
| 범위 제한 | ROS 2 `ROS_AUTOMATIC_DISCOVERY_RANGE` (SUBNET / LOCALHOST / OFF), `ROS_STATIC_PEERS` (Iron부터, `ROS_LOCALHOST_ONLY` 대체) | 디스커버리가 닿는 범위를 줄이고 필요한 peer만 더한다 [확인, 검색 요약. docs.ros.org 본문 미확인] | 범위 밖과는 통신하지 못한다 |
| 다른 프로토콜 | Zenoh (rmw_zenoh) | router 중심, interest 단위 디스커버리 ([middleware-comparison.md 4절](middleware-comparison.md#4-디스커버리-관점-chapter-0의-핵심-근거)) | DDS 와이어 상호운용을 포기한다 |

- 출처
  - Fast DDS Discovery Server: https://raw.githubusercontent.com/eProsima/Fast-DDS-docs/v2.12.2/docs/fastdds/ros2/discovery_server/ros2_discovery_server.rst (재확인). 트래픽 최대 93% 감소 주장은 https://discourse.openrobotics.org/t/new-discovery-server/17383 [벤더 주장]
  - Fast DDS STATIC EDP: https://raw.githubusercontent.com/eProsima/Fast-DDS-docs/master/docs/fastdds/discovery/static.rst [확인, 조사 결과]
  - Cyclone DDS 설정: https://raw.githubusercontent.com/eclipse-cyclonedds/cyclonedds/master/docs/manual/config/config_file_reference.rst (재확인). 규모 확장 가이드는 찾지 못함
  - RTI Cloud Discovery Service: https://community.rti.com/static/documentation/connext-dds/current/doc/manuals/addon_products/cloud_discovery_service/introduction.html [확인, 조사 결과]. LBED / LBPD: https://community.rti.com/static/documentation/connext-dds/7.1.0/doc/api/connext_dds/api_c/group__DDSDiscoveryConfigQosModule.html [확인, 조사 결과]
  - ROS 2: https://docs.ros.org/en/iron/Tutorials/Advanced/Improved-Dynamic-Discovery.html [추정, 차단되어 본문 미확인]

## 6. 표준화 상태
- RTPS 2.5 (formal/22-04-01) 이후 더 큰 네트워크용 디스커버리가 규격에 더해진 증거는 찾지 못했다. [추정]
- OMG 이슈 DDSIRTP26-37 "Specify how a discovery service could be used to bootstrap discovery": RTI가 2023-06-07에 올렸고 상태는 Open이다. 멀티캐스트가 없는 환경을 주로 겨냥하며, 디스커버리 서비스의 동작과 상호운용을 규격에 적자는 내용이다. (재확인) https://issues.omg.org/issues/DDSIRTP26-37
- 따라서 지금 5절의 대안은 벤더마다 다르고 서로 호환되지 않는다. 표준 SPDP / SEDP만이 벤더 사이의 공통분모다.

## 7. chapter0에 쓸 이야기
- 브로커 없이 서로를 찾는 것이 DDS의 장점 (단일 장애점 없음, 아무 순서로나 참여)이다.
- 그 대가로 모든 참가자가 모든 참가자에게 알려야 하고, 규격도 소~중규모용이라고 인정한다.
- 주기 공지 때문에 작은 시스템에서도 디스커버리가 트래픽의 적지 않은 몫이라는 것은 규격의 동작으로 설명한다. chapter0을 읽는 독자는 아직 실험 환경이 없어 4절의 수치를 언급할 상황이 아니다. 실험 방법은 chapter1 ~ 3에서 소개하고, 수치는 chapter3에서 독자가 직접 본다.
- 현장은 중앙 서버, 정적 설정, 범위 제한, 또는 다른 미들웨어 (Zenoh)로 대응하고, 이는 표준 밖이다. 이 책은 표준 SPDP / SEDP를 직접 구현하면서 (chapter9, 11) 그 비용이 어디서 오는지 본다.

## 8. 확인하지 못한 것
- 참가자 수를 바꿔 가며 디스커버리 트래픽을 잰 독립 측정 수치. 찾은 논문 (Park 2012, DGIST "Discovery Storm", arXiv 2608.02242 등)은 본문을 읽지 못했다. 이 저장소에서 참가자 수를 늘려 직접 재는 시나리오를 더하는 것도 방법이다.
- ROS 2 디스커버리 범위 설정의 공식 문서 본문 (docs.ros.org 차단).
- Fast DDS initial peers 문서 본문.
