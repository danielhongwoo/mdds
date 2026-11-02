# 저자 노트: 도메인과 네트워크를 잇는 중계와 브리지 (chapter 21)

상태: 대기

[roadmap](../roadmap.md) | chapter: 21 ([목차](../../book/README.md#목차))

## 목표
DDS 도메인 사이, 네트워크 사이, 또는 DDS와 다른 프로토콜 사이를 중계 프로세스로 잇는 방식을 본다. 중계는 한쪽에서는 reader, 다른 쪽에서는 writer인 보통의 DDS 참가자이므로, mdds가 표준만 지키면 어느 쪽에도 붙을 수 있다. 중계가 디스커버리 범위를 끊어 규모 문제를 줄이는 대신, 지연과 단일 장애점, QoS 의미 (reliability, durability, ownership)의 보존 문제를 가져온다는 점을 확인한다.

## 범위
하는 것
- DDS <-> DDS 중계: eProsima DDS Router (오픈 소스), RTI Routing Service (비교)
- DDS <-> Zenoh 브리지: zenoh-bridge-dds / zenoh-plugin-dds, ROS 2용 zenoh-bridge-ros2dds. Zenoh 쪽을 지나 다른 DDS 섬과 잇는 구성
- 중계가 보존하는 것과 잃는 것: GUID (원래 writer가 가려짐), 소스 타임스탬프, reliable의 종단 간 의미, TRANSIENT_LOCAL, OWNERSHIP
- mdds를 중계의 한쪽 참가자로 붙이기 (상호운용 확인)

하지 않는 것
- 중앙 디스커버리 서버 (chapter 19), 벤더 TCP / WAN 전송 자체 (chapter 20). 중계가 WAN 구간에 그 전송을 쓰는 경우는 링크한다
- Zenoh 프로토콜 자체의 설명. 비교는 chapter 0과 [research/middleware-comparison.md](../research/middleware-comparison.md)
- rmw_zenoh (DDS를 쓰지 않는 ROS 2 RMW). DDS 와이어가 없으므로 chapter 12의 범위 밖에 둔다

## 읽을 규격과 문서
- DDS 1.4: DURABILITY, OWNERSHIP, DESTINATION_ORDER (중계가 의미를 바꾸는 QoS). RTPS 2.5 Sec. 8.4의 reliable이 구간 단위라는 점
- eProsima DDS Router 문서
- zenoh-plugin-dds, zenoh-plugin-ros2dds README
- RTI Routing Service 문서 (비교)

## 다룰 기능
| 기능 | 만든 곳 | 잇는 것 | 실행 가능 | lab |
|---|---|---|---|---|
| DDS Router | eProsima (Fast DDS 기반) | DDS 도메인, 네트워크, WAN 구간 | 예 (오픈 소스) | 예 |
| zenoh-bridge-dds | Eclipse Zenoh (Cyclone DDS 기반) | DDS <-> Zenoh | 예 (오픈 소스) | 예 |
| zenoh-bridge-ros2dds | Eclipse Zenoh | ROS 2 (DDS) <-> Zenoh | 예 (오픈 소스) | chapter 12 이후 선택 |
| Routing Service | RTI Connext | DDS 도메인, 네트워크, 타입 변환 | 아니오 (상용) | 아니오, 문서로 비교 |

## 구현 후보 (결정 전)
- 새 프로토콜 구현은 없다. mdds는 표준 참가자로서 중계의 한쪽에 붙는다 (상호운용 시험).
- 선택: mdds로 아주 작은 도메인 중계 (도메인 0의 토픽 하나를 도메인 1로)를 만들어, 중계가 GUID와 타임스탬프를 어떻게 다뤄야 하는지 직접 겪는다.

## 실습 환경 (계획)
- DDS Router와 zenoh-bridge-dds 이미지를 `docker/`에 더한다 (이미지 빌드 방식은 [vendor-build.md](../../docs/environment/vendor-build.md) 규칙을 따른다. 공식 이미지를 쓸지 소스 빌드할지는 확인할 것)
- 두 섬: 서로 멀티캐스트가 닿지 않는 두 Docker 네트워크. chapter 20의 구성을 다시 쓸 수 있다

## lab (계획)
| lab | 제목 | 하는 일 -> 기대 결과 |
|---|---|---|
| Lab 21-1 | 도메인 잇기 | DDS Router로 도메인 0의 Cyclone 발행을 도메인 1의 Fast DDS가 받음 -> 받는 쪽이 보는 writer GUID가 Router의 것 |
| Lab 21-2 | 섬 잇기 | 두 네트워크 사이를 zenoh-bridge-dds 두 개로 -> 섬 사이에 DDS 디스커버리 트래픽이 지나가지 않음 (Zenoh 트래픽만) |
| Lab 21-3 | 중계가 바꾸는 것 | TRANSIENT_LOCAL, OWNERSHIP 시나리오를 중계 너머로 -> 늦게 들어온 reader, strength가 어떻게 보이는지 |
| Lab 21-4 | mdds를 한쪽에 | mdds 발행을 중계 너머 벤더가 받음 (chapter 11 이후) |

## 확인할 것
- DDS Router와 zenoh-bridge-dds가 원래 writer의 GUID, 소스 타임스탬프, KeyHash를 그대로 전달하는지
- zenoh-bridge-dds가 TRANSIENT_LOCAL을 어떻게 흉내 내는지 (bridge 쪽 저장)
- 두 도구가 arm64 Docker에서 소스 빌드나 공식 이미지로 돌아가는지, 버전 고정 방법
- RTI Routing Service 평가판으로 실행할 수 있는지와 그 조건
- 중계가 reliable을 종단 간이 아니라 구간마다 보장한다는 설명의 원문 근거

## 완료 기준
- Lab 21-1 ~ 21-3의 캡처가 있고, 중계 앞뒤에서 GUID, 타임스탬프, 디스커버리 트래픽이 어떻게 달라지는지 표로 정리했다.
- 중계가 보존하지 못하는 QoS 의미를 캡처 근거로 설명할 수 있다.
- Lab 21-4: mdds가 중계 너머 벤더와 통신한다 (chapter 11 이후).

## 진행 기록
- 2026-10-30: chapter를 만듦 -> [conventions/book.md](../conventions/book.md#벤더-확장-chapter)
