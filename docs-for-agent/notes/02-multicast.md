# 저자 노트: 멀티캐스트 (chapter 02, 구현 없음)

상태: 완료

[roadmap](../roadmap.md) | chapter: 02 ([목차](../../book/README.md#목차)) | 이전: [01-environment](01-environment.md) | 다음: [03-first-communication](03-first-communication.md)

## 목표
DDS를 띄우기 전에, 컨테이너 사이에서 멀티캐스트가 오가는지와 그 원리 (그룹 가입, IGMP, bridge의 동작)를 확인한다. DDS의 참가자 디스커버리 (SPDP)가 멀티캐스트에 기대므로, 여기서 막히면 DDS도 서로를 찾지 못한다.

## 범위
하는 것
- 컨테이너 사이 연결과 멀티캐스트 송수신 확인 (`tools/mcast-check`)
- IGMP 가입과 bridge의 전달 방식 (snooping, querier, flooding) 관찰과 캡처

하지 않는 것
- DDS가 멀티캐스트를 쓰는 방식 (SPDP)의 상세 (chapter 03에서 관찰, chapter 09에서 구현)
- RTPS 포트 공식 (chapter 08)
- 네트워크 장비에 남는 상태의 상세 ([research/discovery-network-state.md](../research/discovery-network-state.md), book 반영은 미정)

## 산출물
| 산출물 | 위치 |
|---|---|
| 멀티캐스트 확인 도구 | `tools/mcast-check`, 실행 이미지 `docker/tools/Dockerfile`, `docker/mcast-check.sh` |
| 멀티캐스트 캡처 | `captures/mcast-check/mcast-check.{cyclone,fastdds,nojoin}.pcap` |
| 원리와 검증 명령 | [docker-network.md](../../docs/environment/docker-network.md#멀티캐스트가-컨테이너-사이를-넘어가는-원리) |
| 교재 chapter | [book/](../../book/README.md) 02 |

## 작업 순서

### Step 1. 연결
- [x] 컨테이너 두 개가 서로 ping 된다 (`docker compose exec cyclone ping -c1 fastdds`). 이후 명령은 모두 `docker/`에서 친다.

### Step 2. 멀티캐스트 확인 도구
- [x] `tools/mcast-check`: 239.255.0.1:7400에 보내고 받는 Rust 도구. 벤더 컨테이너의 네트워크 네임스페이스를 빌려 실행한다 ([검증 명령](../../docs/environment/docker-network.md#검증-명령)).

### Step 3. 멀티캐스트 검증
- [x] 멀티캐스트가 컨테이너 사이를 넘어간다. 받는 쪽에서 `ip maddr show eth0`에 239.255.0.1이 보이는 것도 확인한다.
- [x] 캡처가 된다. `tcpdump`가 mcast-check 패킷과 IGMP join 패킷을 잡고, pcap 파일이 Mac에서 열린다 (`captures/mcast-check/mcast-check.*.pcap`).
- [x] 아무도 가입하지 않은 그룹으로 보내도 받는 쪽 `eth0`까지 오는지 본다 (`mcast-check.nojoin.pcap`).
- 여기서 실패하면 [docker-network.md "문제와 대응"](../../docs/environment/docker-network.md#문제와-대응)을 본다. DDS를 띄우기 전에 해결한다.

## lab
규칙은 [conventions/book.md](../conventions/book.md#lab).

| lab | 제목 | 하는 일 -> 기대 결과 | 쓰는 산출물 |
|---|---|---|---|
| Lab 2-1 | 컨테이너 사이 연결 | 주소 확인과 ping -> cyclone 172.28.0.10, fastdds 172.28.0.11이 서로 닿음 | [docker-network.md](../../docs/environment/docker-network.md#네트워크-구성) |
| Lab 2-2 | 멀티캐스트 보내고 받기 | `mcast-check`로 239.255.0.1:7400 송수신, `ip maddr show eth0` -> 3건 모두 수신, 가입 주소가 보임, tcpdump에 IGMP join이 잡힘 | `tools/mcast-check`, `captures/mcast-check/` |
| Lab 2-3 | 가입하지 않으면 | 아무도 가입하지 않은 채 송신 -> 패킷이 받는 쪽 `eth0`까지 옴. bridge는 거르지 않고, 받을지는 받는 쪽 커널의 가입이 정함 | `mcast-check.nojoin.pcap` |

## 완료 기준
- `mcast-check`로 컨테이너 사이 멀티캐스트 송수신을 재현할 수 있다.
- IGMP join과 leave, 가입하지 않은 경우의 전달을 캡처로 설명할 수 있다.

## 진행 기록
결정과 막힌 점만 한 줄씩 적고 근거 문서로 링크한다. 세부 경위는 커밋 메시지에 둔다.
- 2026-10-28: 멀티캐스트 확인 도구를 socat 대신 Rust로 만듦 (`tools/mcast-check`, 이미지 `mdds/tools`, 벤더 컨테이너의 네트워크 네임스페이스를 빌려 실행). 이를 위해 워크스페이스 뼈대를 함께 만듦 -> [docker-network.md "검증 명령"](../../docs/environment/docker-network.md#검증-명령)
- 2026-10-28: Docker Desktop (Mac arm64) VM 안 bridge가 컨테이너 사이 멀티캐스트를 전달함을 확인 (fastdds 가입, cyclone 송신 3건 모두 수신)
- 2026-10-28: 이 환경의 bridge는 querier가 없어 IGMP 가입과 무관하게 멀티캐스트를 모든 포트로 flooding함 (전달 여부는 받는 쪽 커널의 가입이 결정) -> [docker-network.md](../../docs/environment/docker-network.md#멀티캐스트가-컨테이너-사이를-넘어가는-원리)
- 2026-10-30: chapter 01-04를 함께 맡던 노트에서 나눔
