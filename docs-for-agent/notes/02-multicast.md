# 저자 노트: 멀티캐스트 (chapter 02, 구현 없음)

상태: 완료

[roadmap](../roadmap.md) | chapter: 02 ([목차](../../book/README.md#목차)) | 이전: [01-environment](01-environment.md) | 다음: [03-first-communication](03-first-communication.md)

## 목표
DDS를 실행하기 전에, 컨테이너 사이에서 멀티캐스트가 오가는지와 그 원리 (그룹 가입, IGMP, bridge의 동작)를 확인한다. DDS의 참가자 디스커버리 (SPDP)가 멀티캐스트에 기대므로, 여기서 막히면 DDS도 서로를 찾지 못한다.

## 범위
하는 것
- 컨테이너 사이 멀티캐스트 송수신 확인 (`tools/mcast-check`)
- IGMP 가입과 탈퇴, bridge의 전달 방식 (snooping, querier, flooding) 관찰과 캡처

하지 않는 것
- 컨테이너 사이 유니캐스트 연결 (Lab 1-1의 ping이 맡는다)
- DDS가 멀티캐스트를 쓰는 방식 (SPDP)의 상세 (chapter 03에서 관찰, chapter 09에서 구현)
- 벤더별 SPDP TTL (chapter 04), RTPS 포트 공식과 여러 참가자의 포트 공유 (`SO_REUSEADDR`, chapter 08)
- `IP_MULTICAST_ALL`, IGMPv3 source filtering, querier 선출, 멀티캐스트 라우팅: 이 chapter의 목표 (네트워크와 DDS를 나눠 확인)에 필요 없다
- 네트워크 장비에 남는 상태의 상세 ([research/discovery-network-state.md](../research/discovery-network-state.md))

## 산출물
| 산출물 | 위치 |
|---|---|
| 멀티캐스트 확인 도구 | `tools/mcast-check`, 실행 이미지 `docker/tools/Dockerfile`, `docker/mcast-check.sh` |
| 참고 캡처 스크립트 | `docker/capture-mcast-check.ts` (시나리오 `join`, `nojoin`) |
| 참고 캡처와 설명 | `captures/mcast-check/{join.fastdds,join.cyclone,nojoin.fastdds}.pcap`, [README](../../captures/mcast-check/README.md) |
| 이 환경의 사실과 검증 명령 | [docker-network.md](../../docs/environment/docker-network.md#멀티캐스트가-컨테이너-사이를-넘어가는-원리) |
| 교재 chapter | [book/02-multicast](../../book/02-multicast/README.md) |

## 틀
관찰 chapter의 틀 ([book.md](../conventions/book.md#chapter))에서 4, 5, 6을 뺐다. 벤더 DDS가 나오지 않는다.
1. 왜 필요한가: 상대 주소를 모를 때 찾는 세 방법, RTPS 기본 주소 (Sec. 9.6.2.4, 9.6.2.3), 멀티캐스트가 막혔을 때의 증상, UDP라 전달 보장 없음 (-> chapter 10), lab 전에 알아 둘 용어, mcast-check의 소켓 호출 순서 (C 함수 이름 표, Rust 코드는 싣지 않음)
2. 캡처에서 보기: Lab 2-1, Lab 2-2
3. 규격 읽기: 3.1 주소와 MAC (RFC 1112, 2365, IP 그룹 32개가 MAC 하나를 나눠 쓰므로 커널이 IP 단계에서 거름), 3.2 IGMPv3 (RFC 3376: 가입과 탈퇴 모두 Report, 재전송 규칙), 3.3 snooping, querier, flooding (RFC 4541은 Informational, Linux bridge는 querier가 없으면 flooding, querier가 있어도 가입자 없는 그룹은 flooding), 3.4 네트워크를 먼저 의심할 때
4. 스스로 확인하기: 질문 네 개 (보내는 쪽도 가입해야 하는가, 가입 없이 eth0에 도착했으면 가입은 필요 없는가, 서브넷이 다른 두 노트북, chapter 03에서 서로 못 찾을 때의 순서)

**결정: lab을 원리보다 먼저 둔다 (관찰 틀 그대로).** 1절 끝에 lab에 필요한 최소 용어만 두고, 규칙은 3절에서 lab 결과를 근거로 설명한다.
- 왜: 원리를 먼저 쓰면 lab 전에 RFC 네 개의 용어가 한꺼번에 나온다. 책 전체의 방식 (본다 -> 규격으로 설명한다)과도 맞다.
- 버린 대안: 원리 -> lab (계획 초안). 독자 검토에서 lab 전에 지루해지는 지점으로 지적되었다.

**결정: "연습 문제" 대신 "스스로 확인하기"를 둔다.** 연습 문제의 출처인 캡처 README의 질문은 모두 뒤 chapter로 넘기는 것뿐이고, 이 chapter의 핵심 (가입이 하는 두 가지 일, 네트워크와 DDS를 나눠 확인)은 해설이 있어야 혼자 점검할 수 있다.

**결정: 기존 계획의 Lab 2-1 "컨테이너 사이 연결 (ping)"은 뺐다.** Lab 1-1이 같은 확인을 한다.

**결정: 참고 캡처는 받는 쪽과 보내는 쪽을 따로 둔다 (`join.fastdds`, `join.cyclone`).** multicast loopback 때문에 보낸 쪽 캡처에는 자기 패킷이 두 개씩 보인다. 이 차이를 chapter 03에서 패킷 수를 셀 때 쓴다.

## 작업 순서

### Step 1. 멀티캐스트 확인 도구
- [x] `tools/mcast-check`: 239.255.0.1:7400에 보내고 받는 Rust 도구. 벤더 컨테이너의 네트워크 네임스페이스를 빌려 실행한다 ([검증 명령](../../docs/environment/docker-network.md#검증-명령)).

### Step 2. 멀티캐스트 검증과 참고 캡처
- [x] 멀티캐스트가 컨테이너 사이를 넘어간다. 받는 쪽 `ip maddr show eth0`에 239.255.0.1이 보이고, 받기를 끝내면 사라진다.
- [x] `docker/capture-mcast-check.ts`로 `join`, `nojoin` 캡처를 만든다.
- [x] 아무도 가입하지 않은 그룹으로 보내도 받는 쪽 `eth0`까지 온다 (`nojoin.fastdds.pcap`). bridge 설정 `multicast_snooping=1`, `multicast_querier=0`.
- [x] 같은 IGMP가 두 번씩 보이는 것이 loopback 때문인지 `tcpdump -Q out` / `-Q in`으로 확인 (out 2, in 같은 2).

## lab
규칙은 [conventions/book.md](../conventions/book.md#lab).

| lab | 제목 | 하는 일 -> 기대 결과 | 쓰는 산출물 |
|---|---|---|---|
| Lab 2-1 | 멀티캐스트 보내고 받기 | 터미널 세 개로 tcpdump, recv, send, `ip maddr` -> hello 3건 수신, 가입 중에만 239.255.0.1이 보임, 캡처 11 프레임 (Join 2회 x 2, UDP 3, Leave 2회 x 2) | `tools/mcast-check`, `docker/mcast-check.sh`, `join.fastdds.pcap` |
| Lab 2-2 | 가입하지 않으면 | recv 없이 send -> eth0 캡처에 UDP 3개, `ip maddr`에 그룹 없음 | `nojoin.fastdds.pcap` |

## 확인한 것 (원문)
- RTPS 2.5 Sec. 9.6.2.4 (DefaultMulticastLocator 239.255.0.1, "All Participants must announce and listen"), Sec. 9.6.2.3 (PB = 7400).
- RFC 3376 Sec. 4.2.12 (record type), 4.2.14 (224.0.0.22), 5.1 (재전송 [Robustness Variable] - 1번, 간격 (0, [Unsolicited Report Interval])), 8.1 (기본 2), 8.11 (기본 1초). 탈퇴는 TO_IN({}) (IGMPv2 Leave에 대응하는 표).
- RFC 1112 Sec. 6.4, RFC 2365 Sec. 6.1, RFC 4541 Sec. 2.1.2 (Informational).
- Linux bridge (net/bridge/br_input.c, master): MDB 기반 전달은 `br_multicast_querier_exists()`가 참일 때만, 그 밖에는 `br_flood`. link-local (224.0.0.x)은 MDB에 넣지 않는다. 받는 쪽 폐기는 IP 라우팅 입력 단계 (`ip_check_mc_rcu`). 커밋을 고정하지 않았으므로 본문에는 소스 경로를 쓰지 않았다.
- 소켓 기본값: `mc_ttl = 1`, `MC_LOOP` 켬 (net/ipv4/af_inet.c `inet_create`).
- 2026-11-02 (torvalds/linux master): veth에는 MAC 멀티캐스트 필터가 없다 (drivers/net/veth.c `veth_set_multicast_list`는 빈 함수). 공통 수신 경로 (include/linux/etherdevice.h `eth_skb_pkt_type`)는 그룹 MAC이면 등록 여부와 무관하게 PACKET_MULTICAST로 올린다. 가입하지 않은 그룹의 폐기는 net/ipv4/route.c의 `ip_check_mc_rcu` 호출에서다. bridge는 querier와 무관하게 Report로 MDB 항목을 만든다 (net/bridge/br_multicast.c `br_multicast_add_group`은 `br_multicast_ctx_should_use`만 본다). bridge 기본값: membership_interval 260 s, query_interval 125 s, querier_interval 255 s (`br_multicast_ctx_init`).
- RFC 3376 Sec. 4: IGMP 메시지는 TTL 1로 보낸다. Sec. 5.1의 표: 가입 = TO_EX({}), 탈퇴 = TO_IN({}).
- RTPS 2.5 Sec. 9.6.2.3: PB = 7400, DG = 250, d0 = 0.

## 확인할 것
- querier가 있는 Linux 호스트에서 Lab 2-2의 결과 (소스로는 가입자 없는 그룹은 flooding이라 같을 것으로 [추정]). 두 컨테이너 환경에서는 시험하지 않았다. 후보: IGMPv2 querier가 있는 LAN의 Linux 호스트 ([research/linux-test-host-network.md](../research/linux-test-host-network.md)).
- 비대화형 (`timeout`, 파이프)으로 `mcast-check.sh ... recv`를 끝내면 IGMP 탈퇴가 캡처되지 않은 사례 (원인 미조사). 터미널 (pty)에서 Ctrl+C로 끝내면 재현되지 않고, 스크립트는 `docker kill -s INT`로 끝내 문제가 없다.
- 보낸 직후 tcpdump를 끝내면 마지막 패킷을 놓친다 (Lab 2-2에서 3개 중 2개). 2초 기다리면 모두 잡힌다. 원인은 [추정] 신호를 받을 때 아직 읽지 않은 패킷.
- Ubuntu 호스트에서 lab 실행 (macOS에서만 확인).
- 독자 관점 검토 (2026-11-02)에서 나왔지만 반영하지 않은 구조 제안. 저자가 판단한다.
  - 1.1절이 길고 3절과 겹친다 (ARP, 224.0.0.0/4, IGMP Report, flooding). lab에 필요한 항목만 남기고 나머지는 3절에서 처음 설명하는 안.
  - 4절 질문 1, 2가 2.1.5, 2.2.5의 확인 질문과 겹친다. 상황에서 출발하는 질문으로 바꾸는 안 (예: "tcpdump에는 hello가 보이는데 프로그램이 못 받는다. 원인은?").
  - 1.1절의 `docker compose build tools`에 걸리는 시간을 측정해 적는 것.

## 완료 기준
- `mcast-check`로 컨테이너 사이 멀티캐스트 송수신을 재현할 수 있다.
- IGMP 가입과 탈퇴, 가입하지 않은 경우의 전달을 캡처로 설명할 수 있다.

## 진행 기록
결정과 막힌 점만 한 줄씩 적고 근거 문서로 링크한다. 세부 경위는 커밋 메시지에 둔다.
- 2026-10-28: 멀티캐스트 확인 도구를 socat 대신 Rust로 만듦 (`tools/mcast-check`, 이미지 `mdds/tools`) -> [docker-network.md "검증 명령"](../../docs/environment/docker-network.md#검증-명령)
- 2026-10-28: Docker Desktop (Mac arm64) VM 안 bridge가 querier 없이 멀티캐스트를 모든 포트로 flooding함을 확인 -> [docker-network.md](../../docs/environment/docker-network.md#멀티캐스트가-컨테이너-사이를-넘어가는-원리)
- 2026-10-30: chapter 01-04를 함께 맡던 노트에서 나눔
- 2026-10-31: 아키텍트, 시니어 엔지니어, DDS 전문가, 독자 네 관점의 검토와 협의로 틀을 정하고 본문 초안 작성 -> [틀](#틀). 참고 캡처를 스크립트로 다시 만들고 이름을 `join`, `nojoin`으로 바꿈. docker-network.md의 원리 설명은 chapter 02로 옮기고 틀린 문장 (querier가 있으면 가입 포트로만, Leave 표현, 재전송 시각)을 고침
- 2026-11-01: Lab 2-2에서 bridge 설정 읽기를 빼고 3.3절 querier 설명으로 옮김. lab 시점에는 snooping, querier를 설명하기 전이라 값을 해석할 수 없다. 읽는 명령은 [docker-network.md](../../docs/environment/docker-network.md#멀티캐스트가-컨테이너-사이를-넘어가는-원리)
- 2026-11-01: 네트워크 지식이 적은 독자를 기준으로 고쳐 씀. 1.1절에 lab 네트워크 그림과 기본 개념 (호스트, 스위치와 포트, IP/MAC 주소, 커널과 소켓, 서브넷)을 넣고, 3.1절에 그룹 MAC 주소를 ARP로 묻지 않고 계산하는 이유를 넣음. 주소 범위 (Local Scope 등)는 DDS 이해에 필요하지 않아 RFC 2365 링크만 남김
- 2026-11-02: 사실 확인 (RFC, 커널 원문)과 독자 관점 검토를 거쳐 고침. Lab 2-2는 querier 유무를 가르지 못한다 (가입자 없는 그룹은 어느 경우든 flooding)는 점을 2.2.4와 3.3.3에 밝힘. veth에는 MAC 필터가 없어 가입하지 않은 eth0에서도 그룹 패킷이 보이는 이유를 3.1.2에 넣음. 3.4의 순서를 "DDS 없이 전달 확인 -> DDS 실행 후 가입 확인"으로 바꿈. 질문 3에서 TTL과 chapter 21을 뺌 (3.1.1만으로 답할 수 있다). 참고 캡처를 다시 만들어 본문의 시각을 맞춤. 끝내지 않은 `mcast-check recv` 도구 컨테이너가 7400을 잡고 있던 사례를 2.1.6과 docker-network.md에 적음. 3.3.2의 Query 반복 그림과 수치 (125초, 0-10초, 260초)는 이 환경에서 관찰할 수 없어 한 문단으로 줄임
