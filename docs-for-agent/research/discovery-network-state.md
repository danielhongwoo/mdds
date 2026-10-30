# 조사 노트: DDS 트래픽이 네트워크 장비에 남기는 상태

[roadmap](../roadmap.md) | 함께 보기: [discovery-scaling.md](discovery-scaling.md), [docker-network.md](../../docs/environment/docker-network.md#멀티캐스트가-컨테이너-사이를-넘어가는-원리) | 사용처: 미정 (book에 넣을지는 나중에 결정)

확인일: 2026-10-30. 표기는 [middleware-comparison.md](middleware-comparison.md)와 같다: [확인] / [추정] / [벤더 주장] / [3rd] / (재확인). [추론]은 규격과 관찰에서 저자가 이끌어 낸 것이다.

질문: DDS 디스커버리 (SPDP 멀티캐스트, SEDP와 사용자 데이터 유니캐스트)가 스위치, 라우터, Wi-Fi AP, NAT / 방화벽, 호스트 커널에 어떤 상태를 만들고, 참가자가 정리 없이 사라지면 그 상태가 얼마나 남는가.

---

## 1. 요약
- **장비마다 상태를 만든다.** 스위치는 MAC 표와 IGMP snooping 표, 라우터와 호스트는 neighbor (ARP) 표, NAT / 방화벽은 UDP conntrack 항목, Wi-Fi AP는 단말별 가입 정보와 DTIM 버퍼를 둔다. hub (repeater)는 물리 계층 장치라 상태가 없다. [추정, hub]
- **DDS의 수명과 네트워크 상태의 수명이 다르다.** DDS는 상대가 사라졌음을 lease (Cyclone 10초, Fast DDS 20초)로 알지만, 스위치의 IGMP 가입 정보는 IGMP 기본값으로 260초까지 남을 수 있다. 서로 상대의 정리 시점을 모른다. [추론]
- **프로세스가 죽는 것과 호스트가 사라지는 것은 다르다.** 이 저장소의 lease-expiry 캡처에서 Cyclone 프로세스를 SIGKILL로 죽였는데도 IGMP 탈퇴 보고가 나갔다. 커널이 소켓을 닫으면서 그룹을 떠나는 것으로 보인다 [추정]. 그래서 오래 남는 상태는 주로 호스트 전원이 꺼지거나 링크가 끊길 때 생긴다. [추론]
- **멀티캐스트 TTL이 벤더마다 다르다.** 이 저장소의 캡처에서 SPDP 멀티캐스트의 IP TTL은 Cyclone DDS가 32, Fast DDS가 1이었다. 문서상 기본값도 같다 (Cyclone `General/MulticastTimeToLive` 32, RTI 1). 보통의 LAN에서는 라우터가 멀티캐스트를 라우팅하지 않으므로 어느 쪽이든 서브넷을 벗어나지 않는다. [확인, 캡처와 문서]

## 2. 이 저장소의 캡처에서 본 것
`captures/cyclone-fastdds/` (구독자 컨테이너 eth0에서 캡처, IGMP 포함)

- **IGMP 가입과 탈퇴**: 모든 캡처에 IGMPv3 보고가 12개씩 있다. 시작할 때 각 참가자가 239.255.0.1에 "Change To Exclude" (가입) 보고를 두 번 보내고, 끝날 때 "Change To Include" (탈퇴) 보고를 두 번 보낸다. 두 번인 것은 RFC 3376의 Robustness Variable 기본값 2 때문이다 ([docker-network.md](../../docs/environment/docker-network.md#멀티캐스트가-컨테이너-사이를-넘어가는-원리)). [확인]
- **SIGKILL 뒤에도 탈퇴가 나간다**: lease-expiry에서 Cyclone 발행자를 SIGKILL로 죽인 뒤 (마지막 RTPS 패킷 14.26초) 14.30초와 15.09초에 Cyclone 쪽 탈퇴 보고가 나갔다 (프레임 272, 273). 프로세스는 정리 코드를 돌리지 못했지만 커널이 소켓을 닫으면서 그룹 가입을 해제하고 보고를 보낸 것으로 보인다. [확인, 관찰] 원인 설명은 [추정]이다. 캡처 README에 "누가 보내나? -> chapter 8" 질문으로 남아 있다.
- **SPDP 멀티캐스트의 TTL**: rel-cyclone-pub에서 172.28.0.10 (Cyclone)이 보낸 239.255.0.1 패킷은 TTL 32, 172.28.0.11 (Fast DDS)은 TTL 1이다. lease-expiry도 같다. [확인]
- **유니캐스트 흐름**: 참가자 2개인 rel-cyclone-pub에서 UDP 흐름 (보내는 주소:포트 -> 받는 주소:포트)은 3개였다. Cyclone -> Fast DDS 7410 (메타 트래픽), Cyclone -> Fast DDS 7411 (사용자 데이터), Fast DDS -> Cyclone의 임의 포트. 방화벽이나 NAT가 있으면 흐름마다 conntrack 항목이 하나씩 생긴다. [확인, 흐름 수] [추론, conntrack]
  - 7410, 7411은 RTPS 2.5 Sec. 9.6.2.1의 기본 포트 공식 (PB 7400 + DG 250 x domainId + d1 10 + PG 2 x participantId, 사용자 데이터는 d3 11)에서 domain 0, participantId 0일 때의 값이다. (재확인)

## 3. 장비별 상태

### hub (repeater)
- 받은 신호를 모든 포트로 다시 내보내는 물리 계층 장치다. MAC 표, IGMP 표, 연결 상태가 없다. 그래서 멀티캐스트든 유니캐스트든 모든 호스트가 받고, 각 NIC가 목적지 MAC으로 걸러 버린다. [추정, IEEE 802.3 원문 미확인]

### 스위치
- **MAC 주소 표 (FDB)**: 프레임의 보낸 쪽 MAC과 들어온 포트를 기록한다. SPDP를 포함한 모든 프레임이 갱신한다. Linux bridge 기본 ageing time은 300초다. https://www.kernel.org/doc/html/latest/networking/bridge.html [확인, 조사 결과]. 상용 스위치 기본값은 확인하지 않았다.
- **IGMP snooping 표**: 그룹 (239.255.0.1)마다 가입한 포트를 기록해 그 포트로만 멀티캐스트를 보낸다 (RFC 4541 Sec. 2.1.2). 239.255.0.1은 항상 모든 포트로 보내는 224.0.0.x 예외 대역이 아니므로 snooping 대상이다. [확인, 조사 결과]
  - RFC 4541은 스위치가 탈퇴 메시지에만 기대지 말고 membership timeout을 구현하라고 하고, 그 값을 설정할 수 있게 하라고 한다 (Sec. 2.1.1). (재확인) https://www.rfc-editor.org/rfc/rfc4541.txt
  - 그 timeout의 기준이 되는 IGMP 기본값: Robustness Variable 2, Query Interval 125초, Query Response Interval 10초. Group Membership Interval은 Robustness x Query Interval + Query Response Interval이므로 260초다 (RFC 3376 Sec. 8.1-8.4). (재확인) https://www.rfc-editor.org/rfc/rfc3376.txt
  - 탈퇴 보고를 받으면 querier가 그룹 질의를 짧은 간격 (Last Member Query Interval 1초 x 2회)으로 보내 남은 가입자를 확인하므로 몇 초 안에 정리된다. 탈퇴 없이 사라지면 최대 약 260초 동안 그 포트로 SPDP가 계속 나간다. [확인, RFC 2236 Sec. 8, 조사 결과] [추론, DDS 영향]
- **querier가 없을 때**: snooping 표는 querier의 질의에 대한 보고로 갱신된다. querier가 없으면 가입할 때의 보고 말고는 갱신이 없다. 이때 항목이 만료되어 멀티캐스트가 끊기는 구현과, 반대로 모든 포트로 흘려보내는 구현이 있어 장비마다 다르다. [벤더 주장, 상충]
  - Linux bridge 기본값은 `multicast_snooping` 1, `multicast_querier` 0이다. [확인, 조사 결과] 이 저장소의 Docker Desktop bridge도 같은 설정이었고, 아무도 가입하지 않은 그룹의 패킷을 모든 포트로 흘려보냈다 ([docker-network.md](../../docs/environment/docker-network.md#멀티캐스트가-컨테이너-사이를-넘어가는-원리)). [확인, 관찰]
  - DDS 영향 [추론]: 만료 후 끊기는 구현이면, 처음에는 서로 찾다가 수 분 뒤 SPDP가 끊기는 이상한 증상이 생길 수 있다. 흘려보내는 구현이면 snooping을 끈 것과 같다.

### 라우터
- **멀티캐스트 라우팅 상태** ((*,G), (S,G)): PIM 같은 멀티캐스트 라우팅을 켰을 때만 생긴다. Linux의 `mc_forwarding`은 기본 꺼짐이다. 보통의 LAN에서는 SPDP가 서브넷을 벗어나지 않으므로 라우터에 멀티캐스트 상태가 생기지 않는다. [확인, Linux 문서, 조사 결과] [추정, 일반 라우터]
- **TTL**: TTL은 라우터를 몇 개 지날 수 있는지다. Fast DDS와 RTI의 1은 라우터를 넘지 못한다는 뜻이고, Cyclone의 32는 TTL로는 막지 않는다는 뜻이다. 실제로 넘어가려면 라우터가 멀티캐스트를 라우팅해야 한다. [확인, 캡처와 Cyclone, RTI 문서] [추론, 의미]
  - Cyclone: https://cyclonedds.io/docs/cyclonedds/latest/config/config_file_reference.html , RTI: https://community.rti.com/kb/how-do-i-change-multicast-time-live-ttl
- **neighbor (ARP) 표**: 유니캐스트 상대마다 항목이 생기고, 상대가 사라지면 stale이 되었다가 정리된다. Linux 기본 `gc_stale_time` 60초, 항목 수 상한 `gc_thresh3` 1024. 참가자가 많은 서브넷에서는 이 상한에 닿을 수 있다 [추론]. https://www.kernel.org/doc/html/latest/networking/ip-sysctl.html [확인, 조사 결과]

### NAT / 상태 기반 방화벽
- UDP 흐름마다 conntrack 항목이 생긴다. Linux 기본 timeout은 한 방향만 본 흐름 30초 (`nf_conntrack_udp_timeout`), 양방향 흐름 120초 (`nf_conntrack_udp_timeout_stream`). https://www.kernel.org/doc/html/latest/networking/nf_conntrack-sysctl.html [확인, 조사 결과]
- DDS는 참가자 쌍마다 방향별로 메타 트래픽 흐름과 사용자 데이터 흐름을 만들므로 흐름 수가 대략 N x (N - 1)에 비례한다 [추론]. 참가자가 사라지면 항목은 timeout으로 저절로 사라진다.
- 더 큰 문제는 구조다. SPDP와 SEDP 데이터 안에 "나에게는 이 주소:포트로 보내라"는 locator가 들어 있어, NAT 뒤 참가자의 사설 주소를 상대가 그대로 쓰다가 실패한다. 멀티캐스트도 NAT를 넘지 못한다. [추론, 원문 인용 없음]
- 같은 Docker user-defined bridge 안의 컨테이너끼리는 L2로 전달된다. bridge-nf-call-iptables가 켜져 있으면 iptables (conntrack)를 거친다. [추정]

### Wi-Fi AP
- 멀티캐스트와 브로드캐스트는 가장 낮은 기본 속도로 보내고, ACK와 재전송이 없으며, 절전 중인 단말을 위해 AP가 모아 두었다가 DTIM beacon 뒤에 보낸다. [벤더 주장, IEEE 802.11 원문 미확인]
- 그래서 주기적인 SPDP가 무선 airtime을 많이 차지하고, 받는 단말이 없어도 그 비용을 낸다. [추론]
- 일부 AP는 IGMP snooping으로 단말별 가입을 기억해 멀티캐스트를 단말별 유니캐스트로 바꿔 보낸다 (Aruba Dynamic Multicast Optimization, Linux bridge 포트의 `multicast_to_unicast`). 유니캐스트라 재전송이 되지만, 가입 단말 수만큼 복제된다. [벤더 주장] https://arubanetworking.hpe.com/techdocs/aos/aos10/design/bc-mc/dyn-mc-optimisation/
- 현장 보고 (ROS Discourse, 정량 근거 아님)
  - "Unconfigured DDS considered harmful to Networks" (2022-05-20): 설정하지 않은 ROS 2가 유선과 무선 네트워크를 멀티캐스트로 느리게 만든다는 보고와 토론. 작성자 스스로 나중에 "destructive"는 과장이었다고 했고, 멀티캐스트를 내던 기기를 뽑자 복구되었다고 했다. https://discourse.openrobotics.org/t/unconfigured-dds-considered-harmful-to-networks/25689 [확인, 조사 결과]
  - "ROS2 WIFI Multicast Multi Robot and IGMP Snooping" (2022-11-25): 드론 여러 대를 Wi-Fi로 붙였을 때 세 번째 드론부터 데이터가 끊겼고, IGMP snooping을 켠 공유기로 바꾸자 대부분 해결되었다는 보고. https://discourse.openrobotics.org/t/ros2-wifi-multicast-multi-robot-and-igmp-snooping/28516 [확인, 조사 결과]

### 호스트 커널
- 소켓이 그룹에 가입하면 인터페이스의 그룹 목록 (`ip maddr`)에 올라가고 IGMP 보고를 보낸다. 소켓이 닫히면 가입이 해제되고, 그 그룹을 쓰는 다른 소켓이 없으면 탈퇴 보고를 보낸다. [추정, man page 미확인] 2절의 SIGKILL 관찰과 맞는다.
- 소켓 하나가 가입할 수 있는 그룹 수 상한 `igmp_max_memberships` 기본 20. [확인, 조사 결과]

## 4. 요약 표

| 장비 | 남는 상태 | 만드는 DDS 트래픽 | 수명 (기본값) | 참가자가 정리 없이 사라지면 |
|---|---|---|---|---|
| hub | 없음 | - | - | 영향 없음 |
| 스위치 FDB | MAC -> 포트 | 모든 프레임 | 300초 (Linux bridge) | 항목이 남다가 만료. 영향 작음 |
| 스위치 IGMP snooping | 그룹 -> 가입 포트 | 가입할 때의 IGMP 보고 | 260초 (RFC 3376 기본값으로 계산), 탈퇴 시 몇 초 | 호스트가 사라지면 최대 약 260초 그 포트로 SPDP가 나감. querier가 없으면 장비마다 동작이 다름 |
| 라우터 | 멀티캐스트 라우팅 (켰을 때만), neighbor | 유니캐스트 상대 | neighbor stale 60초 (Linux) | 보통 LAN에서는 SPDP가 서브넷을 넘지 않음 |
| NAT / 방화벽 | UDP conntrack | 참가자 쌍마다 흐름 | 30초 / 120초 (Linux) | timeout으로 사라짐. locator 안의 사설 주소는 구조적 문제 |
| Wi-Fi AP | 단말 가입 (기능이 있을 때), DTIM 버퍼 | SPDP 멀티캐스트 | 장비마다 다름 | 멀티캐스트 airtime 소모가 계속됨 |
| 호스트 커널 | 그룹 가입, neighbor, 소켓 버퍼 | 소켓 가입 | 소켓을 닫으면 해제 | 프로세스가 죽으면 커널이 정리. 호스트가 꺼지면 탈퇴가 안 나감 |

비교: DDS 자신의 정리 시점은 상대 참가자의 lease다 (Cyclone 10초, Fast DDS 20초, [chapter 01-04 노트의 시나리오 표](../notes/01-04-environment-and-captures.md#시나리오-표)).

## 5. book에 넣는다면 (결정은 나중에)
- chapter 02 (멀티캐스트)의 lab: IGMP 가입과 탈퇴 보고를 캡처에서 찾고, SIGKILL 뒤에도 탈퇴가 나가는 것을 확인한다. querier가 없는 Docker bridge가 모든 포트로 흘려보내는 것을 `mcast-check.nojoin.pcap`으로 본다.
- chapter 04 (같은 규격 다른 구현): SPDP 멀티캐스트 TTL이 벤더마다 다르다 (32 / 1).
- chapter 09 (SPDP): lease와 네트워크 장비 timeout의 수명 차이.

## 6. 확인하지 못한 것
- 프로세스가 죽을 때 커널이 탈퇴 보고를 보내는 경로 (Linux 소스나 man page로 확인).
- IEEE 802.11의 group addressed frame과 DTIM 규칙 원문.
- querier가 없을 때 상용 스위치들의 동작과, 벤더가 querier를 두라고 권하는 공식 문구.
- Fast DDS 문서상의 멀티캐스트 TTL 기본값 (캡처로는 1을 관찰).
