# 저자 노트: UDP 멀티캐스트 밖으로, TCP와 WAN 전송 (chapter20)

상태: 대기

[roadmap](../roadmap.md) | chapter: 20 ([목차](../../book/README.md#목차))

## 목표
RTPS 규격이 정한 전송은 UDP/IP뿐이다 (RTPS 2.5 Sec. 9). 멀티캐스트가 막힌 네트워크, NAT와 방화벽 너머, 인터넷 구간에서 벤더들이 TCP나 자체 WAN 전송으로 이를 어떻게 넘는지 본다. 그 대가로 벤더 사이 상호운용이 사라진다는 점을 확인한다.

## 범위
하는 것
- 규격이 정한 것: UDP PSM, 로케이터 kind, 포트 공식. TCP 로케이터가 규격에 있는지
- Fast DDS TCP transport (TCPv4 / TCPv6, listening port, 연결 방향, TLS)
- Cyclone DDS TCP 설정 (있다면 그 수준)
- RTI Real-Time WAN Transport (UDP 기반, NAT 통과, 비교)
- NAT와 conntrack이 디스커버리에 주는 영향. [research/discovery-network-state.md](../research/discovery-network-state.md)의 conntrack 내용을 여기서 쓴다

하지 않는 것
- 중앙 디스커버리 서버 (chapter19). 다만 WAN 구성에서 함께 쓰는 경우는 그 chapter를 링크한다
- 다른 네트워크를 중계로 잇는 방식 (chapter21)
- 공유 메모리 전송 (chapter22 후보)
- DDS-Security와 TLS의 관계 상세 (chapter18)

## 읽을 규격과 문서
- RTPS 2.5 Sec. 9.6 (UDP/IP PSM), Sec. 9.3 Locator_t의 kind 값. TCP kind가 예약되어 있는지
- Fast DDS 문서: TCP transport, WAN 예제
- Cyclone DDS 설정 레퍼런스: `General/Transport`
- RTI Real-Time WAN Transport 문서

## 다룰 기능
| 기능 | 벤더 | 무엇을 바꾸나 | 실행 가능 | lab |
|---|---|---|---|---|
| TCP transport | Fast DDS | RTPS 메시지를 TCP 위에 싣는다 (벤더 자체 framing) | 예 | 예 |
| TCP | Cyclone | (확인할 것) | 확인 후 | 확인 후 |
| Real-Time WAN Transport | RTI Connext | UDP 유지, NAT 통과와 공개 주소 발견 | 아니오 (상용) | 아니오, 문서로 비교 |

## 구현 후보 (결정 전)
- 없음이 기본이다. 벤더 TCP framing은 벤더마다 달라 mdds가 맞출 대상이 하나로 정해지지 않는다.
- I/O-free core 덕분에 전송을 바꿔 끼울 수 있다는 점을 보이는 정도로, mdds 드라이버에 TCP를 붙이는 것은 후보로 둔다 (상호운용 대상이 없으면 mdds끼리만 통신).

## 실습 환경 (계획)
- WAN을 흉내 내는 Docker 구성: 두 bridge 네트워크와 그 사이 NAT 역할의 컨테이너. 멀티캐스트는 넘어가지 않는다. 지연과 손실은 `tc netem`으로 준다 (컨테이너 권한 확인 필요)
- 기존 `docker/compose.yaml`은 바꾸지 않고 별도 compose 파일로 둔다

## lab (계획)
| lab | 제목 | 하는 일 -> 기대 결과 |
|---|---|---|
| Lab 20-1 | NAT 너머에서는 | 두 네트워크 사이 NAT, 표준 UDP 디스커버리 -> 서로 찾지 못하거나 한 방향만 됨. SPDP에 실린 로케이터가 NAT 안쪽 주소임을 캡처로 확인 |
| Lab 20-2 | TCP로 넘기 | Fast DDS TCP transport, 한쪽만 listening -> 통신이 됨. 캡처에서 RTPS가 TCP 스트림 안에 있음 |
| Lab 20-3 | TCP를 모르는 상대 | 같은 구성에 Cyclone (UDP) -> 통신 안 됨. 벤더 전송은 상호운용 범위 밖 |
| Lab 20-4 | 나쁜 링크 | `tc netem`으로 지연, 손실 -> UDP reliable과 TCP의 재전송이 겹칠 때의 지연 |

## 확인할 것
- RTPS 2.5에 TCP 로케이터 kind가 있는지, 없다면 Fast DDS가 쓰는 kind 값 (벤더 영역인지)
- Fast DDS TCP의 framing (RTCP 헤더라고 알려져 있음, 원문 확인), 연결을 누가 여는지, keep alive
- Cyclone DDS TCP 지원 수준과 현재 버전에서의 상태
- RTI Real-Time WAN Transport가 NAT를 넘는 방식 (공개 주소 발견, UDP hole punching 여부)
- Docker Desktop (Mac) 컨테이너에서 `tc netem`을 쓸 수 있는지 (`NET_ADMIN`)

## 완료 기준
- Lab 20-1 ~ 20-4의 캡처가 있고, NAT에서 표준 디스커버리가 막히는 이유를 SPDP 로케이터로 설명할 수 있다.
- 벤더 TCP / WAN 전송이 서로 호환되지 않음을 캡처로 보였다.
- 구현 후보에 대한 결정과 이유를 적었다.

## 진행 기록
- 2026-10-30: chapter를 만듦 -> [conventions/book.md](../conventions/book.md#벤더-확장-chapter)
