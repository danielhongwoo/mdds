# 조사 노트: DDS와 비슷한 미들웨어 비교

[roadmap](../roadmap.md) | 사용처: [chapter 00 노트](../notes/00-introduction.md#비교-대상과-기준)

확인일: 2026-10-30. 대상과 기준을 고른 이유는 [chapter 00 노트 "비교 대상과 기준"](../notes/00-introduction.md#비교-대상과-기준)에 있다.

표기
- [확인]: 원문 (규격 PDF, 공식 문서, 공식 저장소)을 읽고 확인했다.
- [추정]: 원문에서 찾지 못했거나 구조에서 추론했다. 교재에 쓰기 전에 다시 확인한다.
- [벤더 주장]: 만든 회사나 재단의 주장이다. 교재에서는 주장으로 쓴다.
- [3rd]: 제3자 출처다.
- (재확인): 조사 결과를 저자가 원문으로 한 번 더 확인한 항목이다. chapter 00에서 근거로 쓸 것은 모두 재확인한다.

---

## 1. 기준이 되는 DDS
비교의 기준이므로 규격 원문 (`omg-specs/`)으로만 적는다.

- **data-centric**: DDS 1.4 Sec. 1.2는 DCPS를 "global data space" 모델로 설명한다. Publisher가 이 공간에 데이터를 넣으면 미들웨어가 관심 있는 Subscriber에게 전파한다. 데이터 모델의 기본 단위는 topic과 type이다. [확인]
- **적용 분야**: 같은 절이 C4I, 산업 자동화, 분산 제어와 시뮬레이션, 통신 장비 제어, 센서 네트워크, 네트워크 관리를 든다. [확인]
- **표준과 상호운용**: DDS 1.4 Sec. 1.2는 API 이식성을 정하고, 벤더 사이 와이어 상호운용은 별도 규격 (DDSI-RTPS)이 맡는다고 적는다. "at least ten compliant implementations"를 언급한다 (2015년 판 기준). [확인]
- **내장 디스커버리, 단일 장애점 없음**: RTPS 2.5 Sec. 7.2는 DDS가 이름 서버 없이 publisher와 subscriber를 계속 발견하는 내장 디스커버리를 요구하고, 단일 장애점을 두지 말아야 하므로 중앙 이름 서버나 브로커에 의존하면 안 된다고 적는다. [확인]
- **QoS 계약**: RTPS 2.5 Sec. 7.2는 DataWriter와 DataReader 사이의 QoS 계약이 정보 흐름을 조절한다고 적는다. [확인]
- **디스커버리의 한계**: RTPS 2.5 Sec. 8.5.1은 디스커버리를 참가자 (PDP)와 엔드포인트 (EDP) 두 단계로 나누고, 상호운용을 위해 모든 구현이 SPDP와 SEDP를 지원해야 한다고 한다. 그리고 이 둘이 "small to medium scale networks"에 충분하다고 적는다. 큰 네트워크용 프로토콜은 이후 판에 추가될 수 있다고 한다. [확인]
- **이 저장소의 캡처**: rel-cyclone-pub은 프레임 268개 중 디스커버리가 72개다 ([캡처 README](../../captures/cyclone-fastdds/README.md)). [확인]

## 2. 공통 기준

| 기준 | DDS (RTPS) | ZeroMQ | Zenoh | MQTT | SOME/IP | DDS-XRCE |
|---|---|---|---|---|---|---|
| 브로커 | 없음 | 없음 (필요하면 proxy를 직접 구성) | 모드에 따라 다름. peer 모드: 없음 (peer끼리 직접). client 모드: 있음 (router가 broker 역할, 모든 통신이 router 경유). router 모드: router끼리 mesh로 이어 client를 중계 | 필수 (모든 메시지가 broker 경유) | 없음 | Agent 경유 필수 |
| 디스커버리 | 내장 SPDP + SEDP, 모든 엔드포인트 정보 교환 | core에 없음 (endpoint 직접 지정). 별도 규격 ZRE | scouting (UDP multicast) + gossip, key expression 단위 interest | 없음 (broker 주소를 미리 앎). MQTT-SN에는 gateway 발견 | SOME/IP-SD (multicast Offer / Find, TTL, 주기 offer) | Client가 Agent를 찾고, DDS 디스커버리는 Agent가 대신 함 |
| 데이터 모델 | data-centric. topic + type + key (인스턴스), 상태 (history, durability) | message-centric. 불투명한 byte frame, 직렬화 없음 | key expression + bytes. storage / queryable로 최신 값 조회. 타입 시스템 없음 | topic 문자열 + 불투명 payload. retained (마지막 값), session | service 지향 (method, event, field). 직렬화 규칙은 규격에, 타입은 ARXML 모델 | DDS와 같음 (topic, type, key). 엔티티를 이름 참조나 XML로 생성 |
| QoS | 정책 집합 (reliability, durability, history, deadline, liveliness 등) | socket 옵션 (HWM, conflate 등). deadline, durability 없음 | priority 8단계, congestion control, express. deadline류 없음 [추정] | QoS 0/1/2 (구간별), message expiry, will, keep alive | 전송 선택 (TCP / UDP)과 SD 타이머. E2E 보호 | stream 단위 best-effort / reliable. DDS QoS는 Agent 쪽 설정으로 |
| 전송 | UDP (규격 PSM). 구현마다 공유 메모리, TCP | tcp, ipc, inproc, pgm (multicast) 등 | tcp, udp, tls, quic, serial, shared memory 등 | TCP, TLS, WebSocket. MQTT-SN은 UDP와 센서망 | UDP, TCP. SD는 UDP | UDP, TCP, serial (구현은 CAN FD도) |
| 표준화 | OMG (DDS 1.4, DDSI-RTPS 2.5) | 커뮤니티 규격 (rfc.zeromq.org, ZMTP 3.0) | 공개 명세 (Zenoh Protocol Spec 1.0.0). 표준 기구 아님 | OASIS (5.0, 2019), ISO/IEC 20922 (3.1.1) | AUTOSAR (R25-11, 2025-11-27) | OMG (DDS-XRCE 1.0, 2020-02) |
| 독립 구현 | 여럿 (Cyclone, Fast DDS 등) | 여럿 (libzmq, JeroMQ, NetMQ, zmq.rs 등) | 사실상 한 조직 (zenoh-rust, zenoh-pico) [추정] | 많음 (Mosquitto, EMQX, HiveMQ 등) | vsomeip와 상용 스택 | 사실상 하나 (eProsima Micro XRCE-DDS) [추정] |

출처는 6절의 대상별 상세에 있다.

## 3. 분야별 기준

### MCU (펌웨어 / RTOS)
Linux가 도는 edge 기기 (Jetson Orin NX 등)는 포함하지 않는다. 범위는 [chapter 00 노트](../notes/00-introduction.md#비교-대상과-기준)에 있다.


DDS 열은 규격이 아니라 구현의 성질이다. 근거는 6절 "DDS 구현 (MCU, 안전 인증)".

| 기준 | DDS (구현) | ZeroMQ | Zenoh | MQTT | SOME/IP | DDS-XRCE |
|---|---|---|---|---|---|---|
| 지원 RTOS / 플랫폼 | 구현마다 다름. RTI Connext Micro: FreeRTOS (Armv7E-M), MicroSAR (TC397) (재확인). embeddedRTPS: FreeRTOS + lwIP (STM32F7 등). Cyclone DDS, Fast DDS: 공식 MCU 지원 없음 | 공식 지원 없음. contiki-zmtp는 오래됨 | zenoh-pico: Zephyr, FreeRTOS, ESP-IDF, Arduino, RPi Pico 등 | 경량 client 많음. Zephyr, RIOT에 MQTT-SN API | AUTOSAR Classic (MCU) BSW 모듈로 구현 | FreeRTOS, Zephyr, NuttX |
| 크기 | Connext Micro의 MCU 수치는 공개 안 됨 (참조 설정 FreeRTOS heap 400 KB). Cyclone DDS를 ESP32-S3 + Zephyr에 올린 커뮤니티 실측: flash 약 0.9 MB, heap 피크 약 100 kB [3rd] | 수치 없음 | RPi Pico W 기본 구성 flash 약 80 KB, RAM 약 12 KB [벤더 주장] | 공식 수치 없음 | 공식 수치 없음 | 규격 목표 코드 100 KB 미만. 구현 문서: RAM 2 kB 미만 (단순 pub/sub) [벤더 주장] |
| 동적 할당 | Connext Micro: 초기값 = 최대값으로 설정하면 런타임 할당 없음. Cyclone DDS: heap 사용 | 근거 없음 | 확인 못 함 | 구현마다 다름 (coreMQTT는 없음 [추정]) | 정적 설정 중심 [추정] | Client는 동적 / 정적 할당을 쓰지 않음 [벤더 주장] |
| 비IP 전송 | 없음 (규격 PSM이 UDP/IP. MCU 구현도 lwIP UDP) | 없음 | serial | MQTT-SN (ZigBee 등 센서망) | 직접 지원 없음 (gateway) | serial, CAN FD (Agent) |
| 끊기는 링크 | reliable 프로토콜 (HEARTBEAT / ACKNACK)과 lease로 참가자 정리. MCU에서는 endpoint를 정적으로 선언해 SEDP를 생략하기도 함 (Connext Micro DPSE) | TCP 재연결, heartbeat. 재전송 없음 | link lease, keep alive | TCP 의존. MQTT-SN sleeping client | UDP / TCP 선택, SD TTL로 상태 갱신 | session과 reliable stream. sleep / 간헐 연결을 전제로 설계 (규격 8.1) |

### 미션크리티컬

| 기준 | DDS | ZeroMQ | Zenoh | MQTT | SOME/IP | DDS-XRCE |
|---|---|---|---|---|---|---|
| 단일 장애점 | 프로토콜이 중앙 서버나 브로커에 기대지 않도록 규격이 요구 (RTPS 2.5 Sec. 7.2). 데이터를 내는 쪽은 OWNERSHIP으로 이중화 (아래 설명) | 기본은 없음. proxy를 쓰면 생김 | peer는 없음. client 모드는 router | broker | 없음 (provider 자신은 예외) | Agent |
| 시간 제약 표현 | deadline, latency budget, liveliness QoS | 없음 (응용이 처리) | 없음 [추정] | 없음 | SD 타이머, request timeout 정도 | 없음 |
| 보안 | DDS-Security (OMG 규격) | CURVE (인증 + 암호화), ZAP | TLS / mTLS, 인증, 접근 제어 설정 | TLS, 5.0 enhanced auth | 프로토콜에 없음. IPsec, MACsec, TLS, DTLS, SecOC에 맡김 | 규격 범위 밖, 전송 계층 (DTLS / TLS)에 맡김 |
| 안전 인증 | 상용 안전 에디션만. RTI Connext Cert: ISO 26262 ASIL D, TUV SUD, SEooC (재확인). eProsima Safe DDS: ASIL D [벤더 주장]. 오픈소스 Cyclone DDS, Fast DDS는 없음 | 찾지 못함 | Zenoh 자체는 없음. ASIL-D 제품은 계획 단계 [벤더 주장] | 찾지 못함 | 벤더 스택 단위 (예: ASIL-D 개발 프로세스 주장) [벤더 주장] | 없음. 안전 표준으로 개발하지 않았다고 명시 |
| 공급자 | 여러 벤더 | 단일 커뮤니티. libzmq 마지막 릴리스 4.3.5 (2023-10-09) | ZettaScale 주도 | 여러 벤더 | 여러 벤더 | eProsima |

#### 단일 장애점 (single point of failure)
- **뜻**: 그 하나가 멈추면 시스템 전체나 그것에 기대는 모든 통신이 멈추는 구성 요소다. 미션크리티컬 시스템이 가장 먼저 없애려는 대상이고, 보통 같은 일을 하는 것을 둘 이상 두어 하나가 죽으면 다른 것이 이어받게 하는 이중화 (redundancy)와 장애 조치 (failover)로 없앤다.
- **통신 미들웨어에서 생기는 자리**
  1. 모든 메시지가 지나가는 broker: 멈추면 모든 통신이 멈춘다 (MQTT).
  2. 서로를 찾을 때 묻는 중앙 이름 서버나 디렉터리: 멈추면 새 참가자가 연결되지 못한다. 이미 연결된 통신은 이어질 수 있다.
  3. 기기를 대신해 네트워크에 참여하는 agent나 gateway: 멈추면 그 뒤의 기기가 모두 끊긴다 (DDS-XRCE Agent, MQTT-SN gateway, Zenoh client 모드의 router).
  4. 데이터를 내는 쪽 자체: 센서 값을 내는 프로세스가 하나뿐이면 그것이 단일 장애점이다. 미들웨어 구조와 상관없이 생긴다.
- **DDS 규격이 말하는 것**
  - RTPS 2.5 Sec. 7.2: DDS 규격은 구현이 단일 장애점을 두지 않도록 정하고 ("should not introduce"), 따라서 프로토콜은 중앙 이름 서버나 중앙 정보 브로커에 의존하면 안 된다 ("must not rely")고 적는다. Sec. 7.3은 RTPS의 특징으로 단일 장애점 없는 네트워크를 만들 수 있는 fault tolerance를 든다. [확인]
  - RTPS가 이를 위해 택한 설계: 참가자마다 SPDP로 자기를 알리고 (8.5.3.1), SEDP로 서로 직접 endpoint 정보를 주고받으며 (8.5.4.1), 데이터도 writer에서 reader로 바로 간다. 그래서 한 노드가 죽으면 그 노드와의 통신만 끊기고, 남은 참가자는 lease가 끝나면 죽은 참가자를 지운다 (8.5.3.1의 leaseDuration). 위 1, 2, 3번 자리가 프로토콜에 없다.
  - 데이터를 내는 쪽 (4번)은 QoS로 이중화한다. OWNERSHIP을 EXCLUSIVE로 두면 같은 인스턴스를 쓰는 DataWriter 여럿 가운데 살아 있고 (LIVELINESS) DEADLINE을 지키는 것 중 OWNERSHIP_STRENGTH가 가장 큰 하나의 값만 reader에게 보인다. 그 writer가 죽거나 deadline을 어기면 다음 writer로 넘어간다 (DDS 1.4 Sec. 2.2.3.9.2). 장애 조치를 응용 코드가 아니라 표준 QoS로 표현한다는 점이 다른 대상과 다르다. [확인]
- **주의**
  - 규격의 요구는 프로토콜 설계에 대한 것이다. 배포된 시스템에 단일 장애점이 없다는 보장은 아니다. 디스커버리를 중앙 서버로 바꾸는 벤더 기능 (Fast DDS Discovery Server 등)은 확장성을 위해 이 설계를 일부 되돌리는 선택이고, 그래서 서버를 여럿 두는 방법을 함께 제공한다.
  - 이 문장은 RTPS 규격이 DDS 규격을 인용한 것인데, DDS 1.4 원문에서는 "single point" 표현을 찾지 못했다. [확인, 원문 검색]
  - 네트워크 장비 (스위치, 링크)의 이중화는 미들웨어 밖의 일이다.
- **다른 대상**: ZeroMQ는 기본 구조에 없지만 proxy를 두면 생긴다. Zenoh는 peer 모드에는 없고 client 모드에서는 router가 그 자리다 (rmw_zenoh 기본 구성은 router가 있어야 서로를 찾는다). MQTT는 broker가 단일 장애점이고 clustering은 규격 밖이다. SOME/IP는 중앙 broker가 없고 SD도 peer끼리 주고받는다. DDS-XRCE는 Agent가 그 자리다.

### 로보틱스

| 기준 | DDS | ZeroMQ | Zenoh | MQTT | SOME/IP | DDS-XRCE |
|---|---|---|---|---|---|---|
| ROS 2 | 기본 RMW (Fast DDS), Cyclone DDS, Connext | rmw 없음 | rmw_zenoh. Kilted (2025-05)부터 Tier 1 (재확인) | rmw 없음. bridge (mqtt_client) | 직접 관계 없음. bridge 연구와 PoC | micro-ROS (rmw_microxrcedds)로 MCU를 ROS 2에 붙임 |
| 타입 | IDL, XTypes (타입 진화) | 없음 | 없음 (rmw_zenoh는 CDR과 type hash 사용) | 없음 | ARXML 모델 | IDL (DDS와 같음) |
| 큰 데이터 | 단편화 (DATA_FRAG), 구현별 공유 메모리 | 공유 메모리 없음 | 공유 메모리 (rmw_zenoh 기본 임계값 512 bytes) | 전용 기능 없음 | UDP 1400 bytes 권장, TCP 또는 SOME/IP-TP | 제약 기기용이라 해당 없음 |
| 다수 로봇 / 무선 | 디스커버리 트래픽이 문제로 지적됨 (4절) | 해당 기능 없음 | 디스커버리 트래픽 감소를 설계 목표로 함 (4절) | broker 부하 | 차량 안 수십 ECU 가정 [추정] | Agent가 DDS 쪽을 맡음 |

## 4. 디스커버리 관점 (chapter 00의 핵심 근거)
- DDS는 내장 디스커버리를 규격으로 요구한다. 브로커와 이름 서버 없이, 새 참가자가 언제든 들어오고 나간다 (RTPS 2.5 Sec. 7.2). 다른 대상과 비교하면 이것이 DDS의 특징이다. ZeroMQ와 MQTT에는 디스커버리가 없고, SOME/IP-SD는 서비스 단위의 단순한 offer / find이며, XRCE는 Agent에게 맡긴다.
- 대가: SPDP / SEDP는 모든 참가자가 모든 엔드포인트 정보를 교환한다. 규격도 이것이 소~중규모용이라고 적는다 (RTPS 2.5 Sec. 8.5.1).
- Zenoh 블로그 (2021-03-23)는 DDS 디스커버리 트래픽이 n*(n-1)*(r+w)로 는다고 쓴다 (n: 참가자 수, r / w: 참가자당 reader / writer 수). (재확인) [벤더 주장]
- 같은 글의 측정: ROS 2 Foxy, TurtleBot Burger SLAM + 노트북 RViz2, WiFi에서 DDS 디스커버리 686 packets / 251,576 bytes. zenoh-plugin-dds를 거치면 31 packets / 6,617 bytes (97.37% 감소), 두 기법을 함께 쓰면 99.97% 감소. 단일 시나리오이고 벤더 측정이다. (재확인) [벤더 주장]
  - 출처: https://zenoh.io/blog/2021-03-23-discovery/
- ROS 2 core team은 2023-09-27 "ROS 2 alternative middleware report"에서 DDS의 문제가 배포마다 해결은 되지만 전문가의 응용별 DDS 설정이 필요한 경우가 많다고 하고, Zenoh가 요구 사항을 가장 잘 만족하며 사용자 추천도 가장 많았다고 결론 냈다. 구체적 요구 사항과 점수는 링크된 PDF 보고서에 있다 (읽지 못함). (재확인)
  - 출처: https://discourse.openrobotics.org/t/ros-2-alternative-middleware-report/33771
- ROS 2 Kilted Kaiju (2025-05-23)가 Zenoh를 Tier 1 미들웨어로 지원한 첫 릴리스다. (재확인) Lyrical의 기본 RMW는 여전히 Fast DDS다 [확인, 조사 결과].
  - 출처: https://www.openrobotics.org/blog/2025/5/23/ros-2-kilted-kaiju-released
- rmw_zenoh는 multicast scouting을 기본으로 끄고 router로 디스커버리를 한다. 잘못 설정된 네트워크 문제와 같은 LAN 안 로봇끼리의 통제되지 않은 통신을 막기 위해서라고 설계 문서가 적는다. [확인]
  - 출처: https://github.com/ros2/rmw_zenoh/blob/rolling/docs/design.md
- 자원이 적거나 안전 인증이 필요한 곳에서는 동적 엔드포인트 디스커버리를 빼는 선택이 정식 기능으로 있다. RTI Connext Micro의 DPSE (Dynamic Participant, Static Endpoint)는 참가자만 SPDP로 찾고, 상대 endpoint와 그 QoS는 응용이 미리 선언한다. 이때 DataWriter와 DataReader의 QoS 일치를 런타임에 검사하지 않는다고 문서가 경고한다. (재확인) 안전 에디션들도 정적 디스커버리를 지원한다 (6절). 디스커버리가 DDS의 장점이자 비용이라는 점을 보여 주는 예다.
  - 출처: https://community.rti.com/static/documentation/connext-micro/4.1.0/doc/html/usersmanual/discovery.html
- 무선 환경의 디스커버리 트래픽 문제는 저장소 루트의 int2DDS 논문도 제기한다 (인용 여부는 [chapter 00 노트](../notes/00-introduction.md#확인할-것)에서 결정).

## 5. 대상별 한 줄 요약 (DDS와 다른 설계 선택)
- **ZeroMQ**: 브로커 없는 메시지 전송 라이브러리. 디스커버리, 데이터 모델, QoS는 응용 몫이다.
- **Zenoh**: 브로커 없는 pub/sub에 저장과 질의를 더하고, 디스커버리를 interest 단위로 줄였다. 타입 시스템과 풍부한 QoS는 두지 않았다. 공개 명세는 있으나 표준 기구의 표준은 아니다.
- **MQTT**: broker 중심 pub/sub. 단순해서 구현이 많고 제약 기기에 맞지만, broker가 단일 장애점이고 데이터 모델이 없다.
- **SOME/IP**: 차량 안 ECU 사이의 service 지향 RPC. 디스커버리는 단순한 offer / find다. AUTOSAR Adaptive의 ara::com은 SOME/IP와 DDS를 바꿔 끼울 수 있는 network binding으로 둔다.
- **DDS-XRCE**: 제약 기기가 RTPS 디스커버리와 상태를 감당하지 않도록 Agent에게 맡긴 DDS 계열. 대가는 Agent 단일 장애점과 비표준 Client API다.

## 6. 대상별 상세와 출처

### DDS 구현 (MCU, 안전 인증)
MCU 위에서 agent 없이 RTPS 노드로 도는 구현
- RTI Connext Micro (상용): 지원 플랫폼 표에 FreeRTOS 10.0.0 (Armv7E-M), MicroSAR 4.2.2 (TC397). https://community.rti.com/static/documentation/connext-micro/4.3.0/doc/html/release/supported_platforms.html (재확인). FreeRTOS 참조 보드는 STM32F769I-DISC0, lwIP, FreeRTOS heap 400 KB. https://community.rti.com/static/documentation/connext-micro/4.1.0/doc/html/platformnotes/freertos.html [확인]. MCU용 flash / RAM 수치는 공개 문서에서 찾지 못함
- 디스커버리 플러그인 DPDE (SPDP + SEDP)와 DPSE (SPDP + 정적 endpoint) 중 선택. (재확인, 위 discovery 문서)
- 런타임 할당 없이 쓰려면 자원 한도의 초기값과 최대값을 같게 두고, participant를 disabled 상태로 만들어 endpoint를 모두 만든 뒤 enable한다. https://community.rti.com/kb/how-do-i-minimize-memory-footprint-and-avoid-dynamic-memory-allocation [확인]
- embeddedRTPS (RWTH Aachen, 학술): FreeRTOS + lwIP, STM32F767ZI, Aurix, Cortex-R5. SPDP와 SEDP 구현, QoS는 기초 수준이고 스스로 "far from a complete RTPS implementation"이라 적음. Fast DDS와 시험. https://github.com/embedded-software-laboratory/embeddedRTPS [확인]
- Dust DDS (Rust): no_std 대상과 Embassy runtime 지원을 주장. 보드 목록과 크기 수치 없음. https://github.com/s2e-systems/dust-dds [확인]
- Cyclone DDS: `ports/freertos-posix`는 FreeRTOS 시뮬레이터용이고 lwIP 통합이 아직 없다. https://github.com/eclipse-cyclonedds/cyclonedds/tree/master/ports/freertos-posix (재확인). Zephyr 포트 갱신 PR은 최소 구성에 RAM 약 1 MB가 필요하다고 적음. https://github.com/eclipse-cyclonedds/cyclonedds/pull/2461 [확인]. ESP32-S3 + Zephyr 커뮤니티 실측: flash 913,700 B, heap 피크 약 100 kB, best-effort만. https://fj-blanco.github.io/notes/ros2-zephyr/ [3rd]
- Fast DDS, OpenDDS, GurumDDS, CoreDX: MCU 직접 지원 근거를 찾지 못함 [추정]. eProsima의 MCU 답은 Micro XRCE-DDS다.

안전 인증
- RTI Connext Cert for Connext Drive: TUV SUD가 ISO 26262 ASIL D로 인증한 SEooC. 모든 메모리를 시작할 때 할당한다. https://www.rti.com/hubfs/_Collateral/Datasheets/rti-datasheet-connext-cert-for-connext-drive.pdf (재확인). 인증 대상 버전과 인증서 번호는 데이터시트에 없음
- RTI Connext DDS Cert: DO-178C Level A 인증 증거 패키지 (2015, Verocel 제작). 항공은 제품 인증이 아니라 고객이 당국에 내는 증거 방식. https://www.rti.com/news/do-178c-cert-data-package [확인]
- eProsima Safe DDS: Fast DDS와 별개로 새로 작성한 상용 제품. 포럼 공지는 ASIL D 인증을 주장하지만 인증 기관이 없고, 공식 문서는 "designed for compliance"라고만 씀. https://safe-dds.docs.eprosima.com/main/ [확인], https://discourse.openrobotics.org/t/announcement-safe-dds-3-0-is-iso-26262-asil-d-certified-ros-2-tutorial-field-deployment/50361 [벤더 주장]
- Apex.AI: 2021 발표에서 Apex.OS (ROS 2 상용 fork, 당시 DDS는 Connext Micro)가 TUV NORD 감사로 ASIL D. 현재 Apex.Grace 페이지는 ASIL B로 표기. https://www.apex.ai/apexgrace [확인]. ROS 2 (rclcpp) 자체는 인증되지 않음
- TTTech Auto MotionWise Cyclone DDS: ASIL D 인증 진행 중. 완료는 확인 못 함. https://www.trustmotion.com/knowledge-platform/motionwise-cyclone-dds-unleashed [확인]
- 오픈소스 Cyclone DDS, Fast DDS: 인증 주장 없음 [추정]
- 공통점: 안전 에디션은 DDS API의 부분 집합이고, 시작 시 메모리 할당, 정적 디스커버리 지원을 갖는다.

### ZeroMQ
- 브로커 없음이 기본, proxy / broker 패턴은 선택. https://zeromq.org/get-started/ , https://zguide.zeromq.org/docs/chapter5/ [확인]
- 자동 디스커버리 없음. endpoint를 bind / connect로 지정하고 끊기면 재연결 (ZMQ_RECONNECT_IVL 기본 100 ms). https://libzmq.readthedocs.io/en/latest/zmq_setsockopt.html [확인]
- LAN 디스커버리는 별도 규격 36/ZRE (UDP 5670 beacon). libzmq core가 아니다. https://rfc.zeromq.org/spec/36/ [확인]
- PUB은 HWM (기본 1000)을 넘으면 버린다. https://libzmq.readthedocs.io/en/latest/zmq_socket.html [확인]
- wire protocol ZMTP 3.0 (23/ZMTP)은 stable, 3.1은 draft. 커뮤니티 규격이다. https://rfc.zeromq.org/spec/23/ [확인]
- 보안: NULL, PLAIN, CURVE와 27/ZAP. https://rfc.zeromq.org/spec/25/ [확인]
- 최신: libzmq 4.3.5 (2023-10-09), 이 판에서 라이선스가 MPL-2.0으로 바뀜. https://github.com/zeromq/libzmq/releases [확인]
- ROS 2 공식 rmw 목록에 없음. https://docs.ros.org/en/lyrical/Concepts/Advanced/About-Middleware-Implementations.html [확인]

### Zenoh
- peer (기본), client, router 모드. https://zenoh.io/docs/getting-started/deployment/ [확인]
- scouting: UDP multicast 224.0.0.224:7446, SCOUT / HELLO, gossip. https://zenoh.io/docs/getting-started/deployment/ [확인]
- key expression, storage, queryable, HLC timestamp. https://zenoh.io/docs/manual/abstractions/ [확인]
- QoS: priority, congestion control, express. https://docs.rs/zenoh/latest/zenoh/qos/index.html [확인]. reliability가 링크 선택용 표시라는 설명은 검색 요약 [추정]
- 전송 목록. https://raw.githubusercontent.com/eclipse-zenoh/zenoh/main/DEFAULT_CONFIG.json5 [확인]
- 명세: Zenoh Protocol Specification 1.0.0, wire version 9. https://spec.zenoh.io/spec/1.0.0/introduction.html [확인]
- 최신: 1.10.2 (2026-10-28 [추정, 연도 표기 확인 필요]). https://github.com/eclipse-zenoh/zenoh/releases
- zenoh-pico 지원 플랫폼. https://raw.githubusercontent.com/eclipse-zenoh/zenoh-pico/main/README.md [확인]. 크기 수치는 https://zenoh.io/blog/2025-01-08-introducing-raspberry-pi-pico-support-in-zenoh-pico/ (검색 요약) [추정]
- rmw_zenoh: CDR 직렬화, key expression에 domain / topic / type / type hash, 공유 메모리 설정 (임계값 512 bytes, pool 48 MiB). https://github.com/ros2/rmw_zenoh [확인]
- adopters 페이지는 로고 목록이며 배포 규모를 증명하지 않는다. https://zenoh.io/adopters/ [벤더 주장]

### MQTT
- 5.0 규격 (OASIS, 2019-03-07): broker 역할, session expiry, will, keep alive, content type, user property, shared subscription. https://docs.oasis-open.org/mqtt/mqtt/v5.0/mqtt-v5.0.html [확인]
- 3.1.1 (OASIS 2014-10-29, ISO/IEC 20922 승인 발표 2016-07-19): 전송은 ordered, lossless, bi-directional byte stream (TCP, TLS, WebSocket). UDP는 그대로는 부적합. https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html , https://www.oasis-open.org/2016/07/19/oasis-mqtt-internet-of-things-standard-now-approved-by-iso-iec-jtc1/ [확인]
- MQTT-SN 1.2 (2013, IBM 문서): gateway 발견 (ADVERTISE, SEARCHGW, GWINFO), sleeping client, QoS -1. 2.0은 OASIS에서 작업 중. [확인] (PDF), 2.0 진행 상황은 [추정]
- HA와 clustering은 규격 밖, 구현 기능 (EMQX, Mosquitto Pro). [3rd]
- ROS 2 bridge: https://github.com/ika-rwth-aachen/mqtt_client [확인]

### SOME/IP
- 규격: AUTOSAR FO R25-11 (2025-11-27). SOME/IP PRS https://www.autosar.org/fileadmin/standards/R25-11/FO/AUTOSAR_FO_PRS_SOMEIPProtocol.pdf , SOME/IP-SD PRS https://www.autosar.org/fileadmin/standards/R25-11/FO/AUTOSAR_FO_PRS_SOMEIPServiceDiscoveryProtocol.pdf [확인]
- SD: UDP 전용, 기본 포트 30490, Initial Wait -> Repetition -> Main 단계, TTL (0 = stop offer). [확인]
- 데이터: method, event, field. 헤더는 big endian. TLV 직렬화. [확인]
- 보안은 프로토콜 범위 밖 (PRS Sec. 7.2), 하부 IPsec / MACsec / TLS / DTLS. [확인]
- AUTOSAR Adaptive Communication Management (R24-11)는 SOME/IP, signal 기반, DDS network binding을 나란히 둔다. https://www.autosar.org/fileadmin/standards/R24-11/AP/AUTOSAR_AP_SWS_CommunicationManagement.pdf [확인]
- 오픈소스 구현 vsomeip (COVESA, 최신 3.7.6, 2026-09-17). https://github.com/COVESA/vsomeip [확인]

### DDS-XRCE (micro-ROS)
- 규격: OMG DDS-XRCE 1.0 (formal/20-02-01). Sec. 8.1: Client끼리 직접 통신하지 않고 모든 통신이 Agent를 거친다. 코드 100 KB 미만 목표. Client API는 표준이 없어 벤더 사이 이식이 안 된다. https://www.omg.org/spec/DDS-XRCE/ [확인]
- Agent 발견: multicast GET_INFO (기본 239.255.0.2:7400) 또는 미리 설정한 주소. [확인]
- 구현: eProsima Micro XRCE-DDS Client / Agent 3.0.2 (2026-08 / 2026-09). 전송 UDP, TCP, serial, CAN FD (Agent, Linux), custom. https://micro-xrce-dds.docs.eprosima.com/en/latest/ [확인]
- micro-ROS: rclc + rmw_microxrcedds, 정적 메모리 풀, 지원 distro humble / jazzy / kilted / lyrical / rolling. 안전 표준으로 개발하지 않았다고 명시. https://github.com/micro-ROS/rmw_microxrcedds , https://micro.vulcanexus.org/ [확인]
- PX4가 비행 컨트롤러와 ROS 2 사이에 uXRCE-DDS를 쓴다. https://docs.px4.io/main/en/middleware/uxrce_dds.html [확인]

## 7. 확인하지 못한 것
- DDS 안전 에디션의 인증서 원문 (어느 제품도 TUV 인증서나 인증 DB 항목을 보지 못함). Safe DDS의 인증 기관, Apex.Grace의 ASIL 수준 (B / D).
- IEC 61508, IEC 62304, EN 50128 인증을 받은 DDS 제품 (찾지 못함).
- Connext Micro의 MCU용 flash / RAM 수치.
- ROS 2 alternative middleware report PDF 본문 (요구 사항과 점수).
- zenoh-pico의 동적 할당 여부와 손실 링크 동작 (소스로 확인).
- Zenoh 1.10.2 릴리스 연도 표기.
- 대상별 실제 사용 분야의 구체 사례. 지금 표의 분야 열은 규격의 목적, 공식 통합 (ROS 2, AUTOSAR, PX4)과 벤더 주장에 기댄다.
