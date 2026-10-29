# Phase 00 캡처

[Phase 00](../../../docs/phases/phase-00-setup.md) | [시나리오 표](../../../docs/phases/phase-00-setup.md#시나리오-표) | [docker-network.md](../../../docs/environment/docker-network.md)

Cyclone DDS와 Fast DDS의 shape_main을 통신시켜 잡은 패킷과 로그다. 이후 단계에서 파서와 상태 머신을 검증하는 정답 데이터로 쓴다.

## 공통 조건
- 벤더와 빌드 소스: Cyclone DDS master `f2aaccb4` (11.0.1), Fast DDS 3.6.1, shape_main 소스 omg-dds/dds-rtps `526dc56`. 자세한 버전 표는 [docker/README.md](../../../docker/README.md#빌드-소스와-버전).
- 주소: cyclone 172.28.0.10, fastdds 172.28.0.11 (bridge `dds-net`).
- 캡처는 구독자 컨테이너의 `eth0`에서 `tcpdump "udp or igmp"`로 한다. 자기가 보낸 멀티캐스트는 loopback 복사본 때문에 두 번씩 잡힌다 ([docker-network.md](../../../docs/environment/docker-network.md#멀티캐스트가-컨테이너-사이를-넘어가는-원리)).
- 다시 만들기: `docker/`에서 `./phase00-capture.ts <이름>` (기존 파일이 있으면 `--force`). 다시 만들면 프레임 번호와 임의 포트가 바뀌므로 아래 표도 고친다.
- Wireshark 4.6.9 기준. 필터 `rtps`.

## 파일
| 파일 | 내용 |
|---|---|
| `mcast-check.{fastdds,cyclone,nojoin}.pcap` | Step 4 네트워크 검증. DDS 없이 멀티캐스트만. [docker-network.md "검증 명령"](../../../docs/environment/docker-network.md#검증-명령) |
| `rel-cyclone-pub.*` | Cyclone 발행 -> Fast DDS 구독, RELIABLE. 캡처는 fastdds |
| `rel-fastdds-pub.*` | Fast DDS 발행 -> Cyclone 구독, RELIABLE. 캡처는 cyclone |

## rel-cyclone-pub
```
fastdds: fastdds_shape -S -d 0 -t Square -x 1 -r
cyclone: cyclone_shape -P -d 0 -t Square -x 1 -r -c BLUE --write-period 100 --num-iterations 100
```
| 항목 | 프레임 / 값 |
|---|---|
| 첫 SPDP (멀티캐스트) | 1 (fastdds) |
| 첫 SEDP | 20 (fastdds publications writer의 HEARTBEAT), 21 (subscriptions writer) |
| 첫 사용자 ACKNACK / HEARTBEAT / DATA | 33 (fastdds) / 48 (cyclone) / 50 (cyclone) |
| 사용자 DATA seq | 12-100 (89건 모두 전달). 1-11은 구독자 발견 전에 쓴 것이라 VOLATILE로 전달 대상이 아니다 |
| 구독 로그 샘플 수 | 84. 나머지는 구독자 history (KEEP_LAST)에서 덮어써진 것이다 |
| GAP | 없음 |
| 종료 | 구독 로그 끝에 `NOT_ALIVE_NO_WRITERS_INSTANCE_STATE`. 마지막은 Fast DDS의 SPDP `DATA(p[UD])` (참가자 제거) |

## rel-fastdds-pub
```
cyclone: cyclone_shape -S -d 0 -t Square -x 1 -r
fastdds: fastdds_shape -P -d 0 -t Square -x 1 -r -c BLUE --write-period 100 --num-iterations 100
```
| 항목 | 프레임 / 값 |
|---|---|
| 첫 SPDP (멀티캐스트) | 1 (cyclone) |
| 첫 SEDP | 13 (fastdds publications writer), 15 (subscriptions writer) |
| 첫 사용자 ACKNACK / HEARTBEAT / DATA | 20 (cyclone) / 24 (fastdds) / 32 (fastdds) |
| 사용자 DATA seq | 2-100 (99건 모두 전달) |
| 구독 로그 샘플 수 | 99 |
| GAP | 없음 |

## 두 벤더의 차이 (지금까지 확인한 것)
| 항목 | Cyclone DDS | Fast DDS |
|---|---|---|
| vendorId | 0x0110 (Eclipse) | 0x010f (eProsima) |
| RTPS 헤더 프로토콜 버전 | 2.5 | 2.2 |
| SPDP lease duration | 10초 | 20초 |
| SPDP 멀티캐스트 주기 | 시작 때 2번 (0.1초 간격), 이후 8초 | 시작 때 6번 (0.1초 간격), 이후 3초 |
| 유니캐스트 포트 | 임의 (캡처마다 다름. 33690, 44686, 41822, 48749). 메타트래픽과 사용자 데이터가 같은 포트 | 7410 (메타트래픽), 7411 (사용자 데이터). 규격 기본 공식대로 |
| SPDP의 멀티캐스트 로케이터 | 메타트래픽 239.255.0.1:7400, 사용자 데이터 239.255.0.1:7401 | 광고하지 않음 |
| SPDP의 그 밖의 로케이터 | 없음 | kind 0x13 (벤더 전용, 공유 메모리로 추정)의 사용자 데이터 로케이터 |
| 벤더 전용 PID (SPDP) | 0x8007: 길이 52, 문자열 `cyclone/11.0.1/Linux/Linux`를 포함. 0x8019: 길이 4 | 0x8000: 제품 버전 `03 06 01 00` (3.6.1.0). 0x8007: 길이 4, 값 1 |
| 벤더 전용 서브메시지 | 없음 | 모든 메시지 끝에 0x80 (길이 56). 통계 모듈 (`StatisticsSubmessageData`)이 붙인다 |
| SEDP의 PID_TYPE_INFORMATION | 있음 | 없음 |
| SPDP를 상대 유니캐스트 주소로도 보냄 | 예 | 예 |

- **같은 벤더 전용 PID 번호가 벤더마다 뜻이 다르다.** 0x8007이 Cyclone에서는 버전 문자열, Fast DDS에서는 4바이트 값이다. 벤더 전용 PID와 서브메시지는 메시지 헤더의 vendorId를 보고 해석하고, 모르면 건너뛴다 (RTPS 2.5 Sec. 8.3.4.1 규칙 3, Sec. 9.6.2.2.1). mdds 파서가 지켜야 할 규칙이고, 이 캡처가 그 시험 데이터다.
- Fast DDS의 0x80 서브메시지 근거: Fast DDS 실행 파일에 `StatisticsSubmessageData`, `add_statistics_sent_submessage` 기호가 있다. 내용 배치는 바이트로 본 것이다: 목적지 로케이터 (24), 시각 (8), 메시지 순번 (8), 누적 바이트 수 (8), 0 (8). [필드 이름은 추정]
- **Wireshark가 Cyclone의 SEDP DATA를 Malformed로 표시한다** (rel-cyclone-pub 22, rel-fastdds-pub 23). `PID_TYPE_INFORMATION` (XCDR2로 인코딩된 XTypes TypeInformation)의 해석에서 깨진다. Fast DDS는 이 SEDP로 정상 매칭했으므로 Wireshark 해석기 쪽 문제로 본다 [추정]. Phase 12 (XTypes)에서 직접 해석해 확인한다.
- Fast DDS의 RTPS 헤더 버전 2.2는 SPDP의 `PID_PROTOCOL_VERSION`도 2.2다. Cyclone 2.5와 문제없이 통신한다.
