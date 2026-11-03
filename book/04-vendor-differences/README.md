# 4. 같은 규격, 다른 구현

[목차](../README.md) | 이전: [3 첫 DDS 통신 들여다보기](../03-first-communication/README.md) | 다음: [5 DCPS 개념 모델](../05-dcps-concepts/README.md)

chapter 3에서는 두 벤더에 공통인 흐름을 보았다. 이 chapter에서는 같은 규격을 따르는 두 구현이 와이어에서 어떻게 다른지 본다. 그리고 뒤 chapter에서 풀 네 가지 문제, 곧 인스턴스를 거두고 떠나는 법, 말없이 사라진 참가자를 아는 법, 큰 샘플을 나르는 법, 바이트 표현을 바꾸는 법이 와이어에서 어떻게 보이는지 미리 본다. 원리와 구현은 뒤 chapter의 몫이다.

## 1. 왜 필요한가

두 벤더가 통신에 성공하는 데 필요한 것은 규격이 "해야 한다"로 정한 최소 요구뿐이다. 그 밖의 것, 예를 들어 자기를 알리는 주기, 받을 포트, 메시지에 덧붙이는 정보, HEARTBEAT를 얼마나 자주 보낼지는 구현이 고른다. 그래서 chapter 3의 두 캡처는 같은 흐름이지만 자세히 보면 곳곳이 다르다.

mdds를 만들 때 이 구분이 중요하다. 캡처에서 본 동작을 그대로 따라 하면 "Cyclone처럼" 또는 "Fast DDS처럼" 동작하는 구현이 된다. 물어야 할 것은 "어느 쪽을 따라야 하나"가 아니라 "이것은 규격의 어느 쪽인가"다. 이 chapter는 규격의 문장을 네 가지로 나누어 본다.
- **값을 정한 것**: 그대로 따라야 한다. 예: 멀티캐스트 주소.
- **기본값만 준 것**: 기본값으로 시작하되 바꿀 수 있다. 예: 공지 주기.
- **비워 둔 영역**: 벤더가 채우라고 번호 구간을 남겨 둔 것. 예: 벤더 전용 PID.
- **규격에 없는 것**: 아무 말이 없어 구현이 고른 것. 예: IP TTL.

Lab 4-1에서 두 벤더의 차이를 모으고, 3.1절에서 규격 원문으로 네 가지 가운데 어디인지 가른다. 나머지 세 lab은 미리 보기다. 발행자가 떠나는 두 가지 방식 (Lab 4-2), 전송 한 번에 담기지 않는 샘플 (Lab 4-3), 다른 직렬화 표현 (Lab 4-4)을 캡처 한 개씩으로 본다. 각 현상의 규칙은 chapter 13, 9, 15, 16에서 구현하면서 배운다.

이 chapter의 캡처는 모두 chapter 3과 같은 방법으로 만든 참고 캡처다 ([캡처 README](../../captures/cyclone-fastdds/README.md)). 직접 만들어 보는 것은 Lab 4-2의 SIGKILL 하나다.

## 2. 캡처에서 보기

### 2.1 Lab 4-1: 두 벤더의 차이

#### 2.1.1 목표

Cyclone DDS가 발행한 `reliable-cyclone-pub`과 Fast DDS가 발행한 `reliable-fastdds-pub`을 나란히 열어, 두 구현이 와이어에서 다르게 하는 것을 모은다.

#### 2.1.2 준비

[`reliable-cyclone-pub.pcap`](../../captures/cyclone-fastdds/README.md#reliable-cyclone-pub)과 [`reliable-fastdds-pub.pcap`](../../captures/cyclone-fastdds/README.md#reliable-fastdds-pub)을 Wireshark 창 두 개에 하나씩 연다 (macOS에서는 `docker/`에서 `open -n -a Wireshark ../captures/cyclone-fastdds/reliable-cyclone-pub.pcap`처럼 두 번, Ubuntu에서는 `wireshark <파일> &`를 두 번. 한 창에서 File > Open으로 번갈아 열어도 된다). 화면 표기는 Wireshark 4.6.9 기준이다.

두 파일은 캡처한 쪽이 다르다. `reliable-fastdds-pub`은 구독자인 cyclone 쪽에서 캡처했으므로 cyclone이 보낸 멀티캐스트가 두 개씩 보이고 Fast DDS가 보낸 것은 한 번씩 보인다. 사용자 writer의 entityId도 다르다. Cyclone의 Square writer는 `0x00000202`, Fast DDS의 Square writer는 `0x00000102`다.

#### 2.1.3 절차

표의 프레임을 펼쳐 두 벤더의 값을 적는다. 괄호는 Wireshark 상세 창의 라벨이다.

| 볼 것 | reliable-cyclone-pub | reliable-fastdds-pub | 어디서 |
|---|---|---|---|
| 참가자 정보 `DATA(p)`의 필드 | 17 (Cyclone), 18 (Fast DDS) | 1 (Cyclone), 9 (Fast DDS) | RTPS 헤더의 `Protocol version`, `vendorId`. 그 아래 `PID_PARTICIPANT_LEASE_DURATION` > `lease_duration`, `PID_METATRAFFIC_UNICAST_LOCATOR`, `PID_DEFAULT_UNICAST_LOCATOR`, `PID_DEFAULT_MULTICAST_LOCATOR`, `Unknown (0x8000)`, `Unknown (0x8007)`, `Unknown (0x8019)`의 `parameterLength`와 `parameterData`. Internet Protocol 아래 `Time to Live` (17과 1-16은 멀티캐스트) |
| SPDP 멀티캐스트 주기 | Fast DDS 1-16 (0.1초 간격 6번), 그 뒤 3초마다. Cyclone 17, 35 (0.1초 뒤), 204 (8초 뒤) | Cyclone 1-2, 5-6 (0.1초 간격 2번), 그 뒤 8초마다. Fast DDS 9부터 0.1초 간격 6번, 그 뒤 3초마다 | 필터 `rtps.sm.wrEntityId == 0x000100c2 && ip.dst == 239.255.0.1 && frame.time_relative < 12`. 캡처한 쪽 (reliable-cyclone-pub은 fastdds, reliable-fastdds-pub은 cyclone)이 보낸 것은 같은 시각에 두 개씩 보이므로 한 번으로 센다 |
| 사용자 DATA에 붙는 HEARTBEAT와 INFO_DST | 50부터 `INFO_TS, DATA, HEARTBEAT` | 32 `INFO_DST, INFO_TS, DATA -> Square, Unknown[80]`, 33 `INFO_DST, INFO_TS, DATA -> Square, HEARTBEAT -> Square, Unknown[80]` | 필터 `rtps.sm.wrEntityId == 0x00000202` (Cyclone) 또는 `0x00000102` (Fast DDS) |
| 벤더 전용 서브메시지와 Wireshark의 Malformed | Fast DDS의 모든 메시지 끝 `Unknown[80]`. 22 Cyclone의 writer 정보 `DATA[Malformed Packet]` | 같음. Cyclone의 reader 정보 23이 `DATA[Malformed Packet]` | Info 열. `Unknown[80]`을 펼치면 `octetsToNextHeader: 56` |

표를 다 채웠으면 한 행을 골라 (예: lease 기간이나 vendorId) 규격 PDF의 해당 절 (3.1절에 적힌 절 번호)에서 규격이 무엇을 정했는지 직접 읽어 본다.

#### 2.1.4 기대 결과: 두 벤더의 차이

| 항목 | Cyclone DDS 11.0.1 | Fast DDS 3.6.1 | 규격은 (근거는 3.1절) |
|---|---|---|---|
| vendorId | `01.16` (바이트 `01 10`, Eclipse Foundation - Cyclone DDS) | `01.15` (바이트 `01 0f`, eProsima - Fast-RTPS) | 값을 정함. OMG가 배정한다 |
| 프로토콜 버전 | 2.5 | 2.2 | 받는 쪽이 지원하는 major (2)보다 크지 않으면 받는다 |
| lease 기간 | 10초 | 20초 | 값도 기본값도 없음. 공지에 값이 없으면 100초로 보고, 공지 주기보다 길게 잡도록 권한다 |
| SPDP 멀티캐스트 주기 | 시작 때 2번 (0.1초 간격), 그 뒤 8초 | 시작 때 6번 (0.1초 간격), 그 뒤 3초 | 기본값만 줌 (30초). 조정할 수 있어야 한다 |
| 유니캐스트 SPDP | 상대를 찾은 직후 약 1초 간격으로 몇 번, 그 뒤 멀티캐스트 주기에 맞춰 | 상대가 있는 동안 매 주기 멀티캐스트와 함께 | 선택 동작으로만 언급 (처음 본 상대에게 한 번 더, 찾은 상대의 로케이터를 공지 목록에 추가) -> chapter 9 |
| 유니캐스트 포트와 로케이터 | 임의 포트 하나 (캡처마다 다름)를 메타트래픽과 사용자 데이터에 공용. 멀티캐스트 로케이터 239.255.0.1:7400과 7401도 알림 | 7410 (메타트래픽), 7411 (사용자 데이터). 벤더 전용 종류 0x13 (Fast DDS의 공유 메모리 전송)의 로케이터 하나 더. 멀티캐스트 로케이터는 알리지 않음 | 기본 포트를 써야 설정 없이 서로 찾는다. 바꾼 포트는 로케이터로 알린다. 멀티캐스트 239.255.0.1 (도메인 0이면 7400)로 보내고 듣는 것은 값을 정함 |
| 벤더 전용 PID | 0x8007 (52바이트, 문자열 `cyclone/11.0.1/Linux/Linux` 포함), 0x8019 (4바이트) | 0x8000 (`03 06 01 00`), 0x8007 (`01 00 00 00`) | 비워 둔 영역. vendorId에 따라 해석하고, 모르면 건너뛴다 |
| 벤더 전용 서브메시지 | 없음 | 모든 메시지 끝에 0x80 (헤더 4 + 내용 56 = 60바이트) | 비워 둔 영역. 모르는 vendorId의 것은 건너뛴다 |
| SEDP 정보의 선택 PID | PID_TYPE_INFORMATION 있음. PID_DATA_REPRESENTATION은 기본값 (XCDR)이면 싣지 않음 | PID_TYPE_INFORMATION 없음. PID_DATA_REPRESENTATION 늘 있음 | 둘 다 선택 (DDS-XTypes) -> chapter 16 |
| 사용자 DATA에 HEARTBEAT를 붙이는 빈도 | DATA마다 | DATA 두 번에 한 번 | 주기적 HEARTBEAT는 의무, 빈도와 붙이는 방식은 구현 -> chapter 10 |
| 사용자 DATA 앞의 INFO_DST | 없음 | 있음 | 해석에 필요할 때 두는 것은 의무이고, 그 밖에는 구현이 고른다 -> chapter 7 |
| SPDP 멀티캐스트의 IP TTL | 32 | 1 | 규격에 없음 |

같은 PID 번호 0x8007이 Cyclone에서는 52바이트 문자열, Fast DDS에서는 4바이트 값이다. 벤더 전용 영역은 vendorId와 함께 읽어야 뜻이 정해진다. 표의 값 가운데 몇 가지는 두 벤더의 설정 기본값이다 (소스와 설정 문서로 확인). Cyclone은 멀티캐스트 디스커버리를 쓸 때 참가자 번호를 쓰지 않는 설정 (`Discovery/ParticipantIndex` = `none`)이 기본이라 유니캐스트 포트를 임의로 고르고, 멀티캐스트 TTL 기본값이 32이며, SPDP 주기 기본값이 lease의 약 80%다. Fast DDS의 0x80 서브메시지는 통계 모듈이 모든 메시지에 붙이는 것이다. Cyclone의 writer / reader 정보가 `Malformed`로 보이는 것은 chapter 3에서 본 것과 같은 Wireshark 4.6.9의 한계이고, 그 뒤의 PID는 화면에 나오지 않는다. 그래서 위 표의 "PID_DATA_REPRESENTATION을 싣지 않음"은 화면이 아니라 바이트로 확인한 것이다.

#### 2.1.5 확인 질문

- 두 벤더가 알리는 lease 기간이 다르다. Cyclone 구독자는 Fast DDS 발행자가 사라졌다고 판단할 때 어느 값을 쓰겠는가?

#### 2.1.6 이 chapter에서 답하지 않는 것

- 유니캐스트 SPDP를 언제 보내는지, 상대가 없을 때 Cyclone이 종료하면서 `DATA(p[UD])`를 보내지 않는 이유 -> chapter 9
- 두 캡처에서 SEDP 교환에 걸린 시간이 1.1초와 10ms로 다른 이유 -> chapter 11
- HEARTBEAT에 적힌 숫자 (`reliable-fastdds-pub`의 첫 HEARTBEAT는 2-1이다)와 붙이는 빈도의 기준 -> chapter 10
- INFO_DST가 받는 쪽의 해석을 어떻게 바꾸는지 -> chapter 7
- 포트 공식과 participantId, 그리고 Cyclone처럼 임의 포트를 쓰면 유니캐스트만으로 찾을 수 없게 되는 이유 -> chapter 8

### 2.2 Lab 4-2: 사라지는 두 방법

#### 2.2.1 목표

발행자가 사라지는 두 경우를 본다. 하나는 인스턴스를 dispose하고 정리하며 끝나는 경우 (`dispose-on-exit`), 다른 하나는 SIGKILL로 아무 말 없이 죽는 경우 (`lease-expiry`)다. 뒤의 것은 직접 해 본다.

#### 2.2.2 준비

[`dispose-on-exit.pcap`](../../captures/cyclone-fastdds/README.md#dispose-on-exit)과 [`lease-expiry.pcap`](../../captures/cyclone-fastdds/README.md#lease-expiry), 그리고 같은 디렉터리의 `dispose-on-exit.sub.log`, `lease-expiry.sub.log`를 쓴다. 직접 하는 부분은 [chapter 3의 2.1.2절](../03-first-communication/README.md#212-준비) 점검과 터미널 A, B, C에 터미널 D를 하나 더한다. 모두 `docker/`에서 연다.

알아 둘 용어가 셋 있다.
- **dispose와 unregister**는 writer가 인스턴스 (여기서는 색이 BLUE인 Square 하나)에 대해 하는 선언이다. dispose는 그 인스턴스가 더는 유효하지 않다는 뜻, unregister는 이 writer가 더는 그 인스턴스를 갱신하지 않는다는 뜻이다 ([용어 정리](../glossary.md#프로토콜)). chapter 3에서 본 `[UD]`가 둘 다 선 것이고, `[_D]`는 dispose만 선 것이다.
- **inline QoS**는 DATA 서브메시지 안에 함께 실리는 ParameterId 목록이다. 이 선언은 그 안의 `PID_STATUS_INFO`에 실린다.
- **Serialized Key**는 DATA의 플래그로, 페이로드에 값 전체가 아니라 key (여기서는 색)만 들어 있다는 표시다.

#### 2.2.3 절차

먼저 `dispose-on-exit`의 끝부분을 읽는다.

| 프레임 | 시각 (초) | 표지와 보이는 것 |
|---|---|---|
| 266 | 13.30 | 마지막 보통 샘플 `INFO_TS, DATA, HEARTBEAT` |
| 268 | 13.41 | `INFO_TS, DATA([_D]), HEARTBEAT`. 상세 창에서 `Flags`에 `Serialized Key`와 `Inline QoS`, inlineQos의 `PID_STATUS_INFO` > `Flags: 0x00000001, Disposed`, `serializedKey`에 `BLUE`만 있다. writerSeqNumber는 101 |
| 270 | 13.41 | `DATA(w[UD])`. writer가 사라진다 |
| 271-272 | 13.42 | `DATA(p[UD])`. 참가자가 사라진다 (chapter 3의 종료와 같다) |
| 273-274 | 13.42, 14.04 | IGMP Leave |

`dispose-on-exit.sub.log`의 끝은 `matched writers 0`, `NOT_ALIVE_DISPOSED_INSTANCE_STATE`, `Done.`이다. chapter 3의 `reliable-cyclone-pub.sub.log`는 같은 자리가 `NOT_ALIVE_NO_WRITERS_INSTANCE_STATE`였다.

다음은 SIGKILL을 직접 한다.

| 터미널 | 하는 일 |
|---|---|
| A | fastdds 컨테이너의 `eth0` 캡처 |
| B | fastdds 컨테이너에서 구독 |
| C | cyclone 컨테이너에서 발행 (끝나지 않는다) |
| D | 발행자를 SIGKILL로 끝내고 시각을 적는다 |

1. 터미널 A에서 캡처를 시작하고 `listening on eth0` 뒤 1초 기다린다. 같은 이름의 파일이 있으면 말없이 덮어쓴다.
   ```bash
   docker compose exec fastdds tcpdump -i eth0 -U -Z root -w /captures/cyclone-fastdds/my-lease-expiry.pcap 'udp or igmp'
   ```
2. 터미널 B에서 구독자를 시작하고 3초 기다린다.
   ```bash
   docker compose exec fastdds fastdds_shape -S -d 0 -t Square -x 1 -r
   ```
3. 터미널 C에서 발행자를 시작한다. `--num-iterations`가 없으므로 스스로 끝나지 않는다. 터미널 B에 `matched writers 1`과 샘플 줄이 흐른다.
   ```bash
   docker compose exec cyclone cyclone_shape -P -d 0 -t Square -x 1 -r -c BLUE --write-period 100
   ```
4. 샘플 줄이 10초쯤 흐른 뒤 터미널 D에서 다음 한 줄을 실행한다. 시각을 찍고 발행자를 SIGKILL로 끝낸다. 터미널 C에는 프롬프트가 돌아오고 (터미널 C의 `docker compose exec`가 종료 코드 137로 끝난다), 터미널 B의 샘플 줄이 멈춘다.
   ```bash
   date +%T; docker compose exec cyclone pkill -9 -x cyclone_shape
   ```
5. 터미널 B를 보다가 `matched writers 0`이 찍히는 순간 터미널 D에서 `date +%T`를 실행한다. 두 시각의 차가 lease 만료까지 걸린 시간이다. 초 단위 시계와 손의 반응이 섞이므로 1-2초의 오차가 있다.
6. `NOT_ALIVE_NO_WRITERS_INSTANCE_STATE`가 찍힌 뒤 5초 더 기다린다. Fast DDS의 SPDP 주기 (3초)보다 길게 기다려야 "더 보내지 않음"이 캡처에 남는다.
7. 터미널 B에서 Ctrl+C (`Done.`), 2초 뒤 터미널 A에서 Ctrl+C. chapter 3의 2.1.2절 점검 명령으로 남은 프로세스가 없는지 확인한다.
8. `my-lease-expiry.pcap`을 열어 같은 것을 캡처 시각으로 본다. 발행자의 마지막 패킷은 필터 `ip.src == 172.28.0.10 && rtps`의 마지막 프레임이다. 구독자가 발행자에게 보낸 유니캐스트 SPDP는 필터 `ip.src == 172.28.0.11 && ip.dst == 172.28.0.10 && rtps.sm.wrEntityId == 0x000100c2`로 보이는데, 발행자의 마지막 패킷 뒤 10초 안쪽에서 끊기고 그 뒤 3초 간격의 프레임이 없다. 벽시계 시각과 캡처 시각은 직접 비교할 수 없으므로 "끊긴 시점이 마지막 패킷 뒤 10초 안쪽인가"만 본다.

#### 2.2.4 기대 결과

- 직접 한 실행에서 `matched writers 0`은 SIGKILL 뒤 약 10초에 찍힌다 (저자의 실행에서 10.1초). Cyclone이 SPDP로 알린 lease가 10초다. 기준점은 Cyclone의 마지막 SPDP (죽기 약 2-3초 전)가 아니라 마지막 패킷이다. Fast DDS 3.6.1은 상대 참가자의 어떤 RTPS 메시지든 받을 때마다 lease 시계를 되돌린다 (소스로 확인. 3.2절).
- 참고 캡처 `lease-expiry`의 끝은 다음과 같다.

  | 프레임 | 시각 (초) | 표지와 보이는 것 |
  |---|---|---|
  | 205-206 | 11.22 | cyclone의 마지막 SPDP `DATA(p)` |
  | 270 | 14.26 | cyclone의 마지막 패킷 (`DATA, HEARTBEAT`). 이 직후 SIGKILL |
  | 272-273 | 14.30, 15.09 | cyclone 쪽 IGMP Leave. `DATA(w[UD])`도 `DATA(p[UD])`도 없다 |
  | 274, 277, 280 | 15.52, 18.52, 21.53 | fastdds가 cyclone에게 보낸 유니캐스트 `DATA(p)` |
  | 283부터 | 24.53 | fastdds의 `DATA(p)`는 멀티캐스트로만 나간다 |
  | 293-294 | 39.46 | fastdds의 `DATA(p[UD])` (구독자 Ctrl+C) |

  fastdds는 21.53초에는 cyclone에게 유니캐스트로 보냈고 24.53초에는 보내지 않았다. 그 사이에 cyclone을 지운 것으로 보인다. `lease-expiry.sub.log`의 끝은 `matched writers 0`, `NOT_ALIVE_NO_WRITERS_INSTANCE_STATE`, `Done.`으로, 정상 종료 때와 같다.
- 두 경우의 차이는 발행자가 보낸 마지막 메시지에 있다. dispose는 `DATA([_D])`, `DATA(w[UD])`, `DATA(p[UD])`로 알리고 사라진다. SIGKILL은 아무것도 알리지 않고, 구독자가 lease로 판단한다. 구독 로그는 두 경우를 `NOT_ALIVE_DISPOSED`와 `NOT_ALIVE_NO_WRITERS`로 구분한다 (3.2절).

선택 실습: Lab 3-1의 발행 명령 끝에 `--final-instance-state d`를 더하면 `dispose-on-exit`과 같은 캡처가 된다.

#### 2.2.5 확인 질문

- SIGKILL 뒤에도 IGMP Leave는 나갔다. 그런데 왜 구독자는 그것으로 발행자가 사라졌음을 알지 못하는가?

#### 2.2.6 실패하면

- 터미널 B에 `matched writers 1`만 찍히고 샘플 줄이 없거나, SIGKILL 뒤 25초가 지나도 `matched writers 0`이 찍히지 않으면 이전 실습의 발행자가 남아 있는 것이다. 터미널 B를 Ctrl+C로 끝내고 chapter 3의 2.1.2절 점검부터 다시 한다. 남은 발행자는 `docker compose exec cyclone pkill -INT -x cyclone_shape`로 끝낸다.
- 터미널 D의 `pkill`이 아무것도 찾지 못하면 (종료 코드 1) 발행자가 이미 끝난 것이다. 터미널 C를 본다.
- 측정값이 5초 미만이거나 15초를 넘으면 시각을 잘못 찍은 것이다. 캡처 (8단계)로 다시 본다.
- 캡처가 비어 있거나 7400 문제가 있으면 [chapter 3의 2.1.5절](../03-first-communication/README.md#215-실패하면)을 본다. Ubuntu에서 `my-lease-expiry.pcap`은 root 소유로 생긴다.

#### 2.2.7 이 chapter에서 답하지 않는 것

- dispose가 `[_D]`인데 그 뒤의 writer 정리가 `[UD]`인 이유와 key만 직렬화하는 규칙 -> chapter 13
- 규격의 문장은 공지 (SPDP)로 갱신한다고 하는데 Fast DDS는 모든 메시지로 갱신한다. mdds는 어느 쪽을 따를지 -> chapter 9
- SIGKILL로 죽은 프로세스의 IGMP Leave는 누가 보내는가 -> chapter 8
- `dispose-on-exit`의 구독 로그 샘플 수 (92)와 값을 실은 사용자 DATA 수 (98)의 차이 -> chapter 13

### 2.3 Lab 4-3: 큰 샘플

#### 2.3.1 목표

전송 한 번에 담기지 않는 샘플 (약 100KB)이 와이어에서 어떻게 나뉘어 가는지 본다.

#### 2.3.2 준비

[`large-sample.pcap`](../../captures/cyclone-fastdds/README.md#large-sample) (2.9MB)을 연다. 발행 옵션에 `--additional-payload-size 100000`을 더해 샘플 하나를 100,032바이트로 만들고 30번 쓴 캡처다 (직접 만들지 않는다). 파일을 열면 Info 열이 거의 다 `Fragmented IP protocol`이라 깨진 것처럼 보인다. 먼저 필터 `rtps`를 건다. Wireshark는 기본 설정에서 IP 조각을 다시 합친 뒤 해석한다. 합친 결과는 마지막 조각 프레임에 나타난다.

#### 2.3.3 절차

| 프레임 | 표지와 보이는 것 |
|---|---|
| 41-49 | `Fragmented IP protocol`. RTPS가 아니라 IP 조각이다 |
| 50 | `INFO_TS, DATA_FRAG, HEARTBEAT_FRAG`. Internet Protocol 아래 `[10 IPv4 Fragments (13544 bytes): #41(1480), ... #50(224)]`. DATA_FRAG 아래 `fragmentStartingNum: 1`, `fragmentsInSubmessage: 10`, `fragmentSize: 1344`, `sampleSize: 100032` |
| 60, 70, ..., 110 | 같은 샘플의 다음 조각 묶음. `fragmentStartingNum`이 11, 21, ..., 61 |
| 115 | 마지막 묶음 `DATA_FRAG, HEARTBEAT`. `fragmentStartingNum: 71`, `fragmentsInSubmessage: 5` |
| 137 | `HEARTBEAT`만. 다음 샘플의 조각을 보내는 도중이다 |

1. 필터 `rtps`로 310개가 남는 것을 확인한다. 전체 2,198개 가운데 1,876개가 IP 조각이고 12개가 IGMP다.
2. 50을 펼쳐 표의 값을 확인한다. 샘플 100,032바이트를 1,344바이트 조각 75개로 나누고 RTPS 메시지 하나에 10개씩 실었다 (마지막 메시지는 5개). 그 메시지는 약 13.5KB라, 이더넷 프레임 하나에 들어가는 크기 (MTU 1,500바이트)를 넘어 IP 층에서 1,480바이트 9개와 224바이트 1개로 다시 나뉘었다.
3. 필터 `rtps.sm.id == 0x16` (DATA_FRAG)과 `rtps.sm.id == 0x13` (HEARTBEAT_FRAG)으로 둘 다 있는 것을, `rtps.sm.id == 0x12` (NACK_FRAG)로 빠진 조각을 요청한 적이 없는 것을 확인한다.

#### 2.3.4 기대 결과

- 조각내기가 두 층에서 겹쳐 보인다. RTPS는 샘플을 DATA_FRAG 조각 75개로 나누어 메시지 8개에 실었고 ([용어 정리](../glossary.md#프로토콜)), IP는 그 메시지 하나를 다시 10개 (마지막 메시지는 5개)로 나누었다. 샘플 하나가 이더넷 프레임 75개로 갔다.
- IP 조각은 받는 커널이 합친 뒤에 RTPS에 올라오므로 RTPS 메시지 안에는 드러나지 않는다. RTPS 메시지를 얼마나 크게 만들지는 구현이 정한다.
- 조각 크기 1,344와 메시지당 10개는 Cyclone의 설정 기본값에서 나온다 (`General/FragmentSize` 1344 B, 그리고 UDP 페이로드 상한 `General/MaxMessageSize` 14720 B에 조각 10개까지 들어간다). 규격은 조각 크기를 writer가 정해 고정하고, 모든 전송이 조각 하나 이상을 담을 수 있어야 하며, 65,536바이트 이하이고, 필요할 때만 조각내라고 요구한다. 메시지 하나에 조각을 최대한 많이 담으라는 것은 권고다 (RTPS 2.5 Sec. 8.4.14.1).

#### 2.3.5 확인 질문

- 필터 `rtps`를 걸기 전에 프레임 목록의 Info 열이 대부분 `Fragmented IP protocol`인 이유는 무엇인가? RTPS 메시지 하나는 몇 개의 프레임에 걸쳐 있는가?

#### 2.3.6 이 chapter에서 답하지 않는 것

- 조각 크기와 메시지당 조각 수를 정하는 기준, IP 단편화를 피하는 방법, 조각이 빠졌을 때의 NACK_FRAG, 샘플 중간에 나가는 HEARTBEAT -> chapter 15
- 30번 썼는데 캡처의 샘플은 28개 (seq 3-30)다 -> chapter 10, 11

### 2.4 Lab 4-4: XCDR2

#### 2.4.1 목표

같은 샘플을 `-x 2` (XCDR2)로 보냈을 때 와이어에서 무엇이 달라지는지 본다.

#### 2.4.2 준비

[`xcdr2.pcap`](../../captures/cyclone-fastdds/README.md#xcdr2)과 chapter 3의 `reliable-cyclone-pub.pcap`을 연다. 흐름은 같고 바이트 안의 표현만 다르다. ShapeType은 멤버 다섯 개 (`color`, `x`, `y`, `shapesize`, 그리고 이 캡처에서는 길이 0인 `additional_payload_size`)의 appendable 타입 (뒤에 멤버를 더할 수 있는 타입. chapter 16)이다 ([용어 정리](../glossary.md#도구)의 shape_main).

#### 2.4.3 절차

| 파일 | 프레임 | 표지와 보이는 것 |
|---|---|---|
| reliable-cyclone-pub | 50 | 사용자 DATA의 `serializedData` > `encapsulation kind: CDR_LE (0x0001)` ([용어 정리](../glossary.md#프로토콜)의 RepresentationIdentifier) |
| xcdr2 | 41 | `encapsulation kind: D_CDR2_LE (0x0009)` |
| reliable-cyclone-pub | 47 | Fast DDS의 reader 정보 `DATA(r)` > `PID_DATA_REPRESENTATION` > `XCDR_DATA_REPRESENTATION (0x0)` |
| xcdr2 | 38 | 같은 자리 `XCDR2_DATA_REPRESENTATION (0x2)` |
| xcdr2 | 29 | Cyclone의 writer 정보. `Malformed`로 끊기기 전에 `PID_DATA_REPRESENTATION` > `XCDR2_DATA_REPRESENTATION (0x2)`가 보인다. `reliable-cyclone-pub`의 22에는 이 PID가 없다 |

1. 두 사용자 DATA의 `serializedData`를 펼쳐 encapsulation kind와 그 아래 바이트 (`serializedData` 리프)를 비교한다. 아래에 맞춰 적었다. 위가 `reliable-cyclone-pub` 50, 아래가 `xcdr2` 41이다. x, y 값은 캡처마다 다르다.
   ```
               05 00 00 00 42 4c 55 45 00 00 00 00 a1 00 00 00 9c 00 00 00 14 00 00 00 00 00 00 00
   1c 00 00 00 05 00 00 00 42 4c 55 45 00 00 00 00 0f 00 00 00 0c 00 00 00 14 00 00 00 00 00 00 00
   ```
   xcdr2 쪽에만 맨 앞에 `1c 00 00 00` (28)이 더 있다. 뒤따르는 내용의 길이를 적은 DHEADER다. 그 뒤는 두 캡처가 같은 배치다 (길이 5의 `BLUE`, x, y, 크기 20, 길이 0의 sequence).
2. reader 정보와 writer 정보에서 PID_DATA_REPRESENTATION의 값을 비교한다.

#### 2.4.4 기대 결과

- `-x 2`로 바뀐 것은 세 가지다. encapsulation kind (`D_CDR2_LE`), 페이로드 앞의 DHEADER 4바이트 (DDS-XTypes 1.3 Sec. 7.4.3.4.1), SEDP에 알리는 표현 (XCDR2). 나머지 바이트 배치는 이 타입에서는 같다.
- 어느 표현을 쓸지는 SEDP의 PID_DATA_REPRESENTATION으로 알린다 ([용어 정리](../glossary.md#프로토콜)). 값이 없으면 XCDR로 본다 (DDS-XTypes 1.3 Sec. 7.6.3.1.1). Cyclone은 기본값일 때 이 PID를 싣지 않고 `-x 2`일 때만 싣는다. 양쪽에 `-x 2`를 주었으므로 writer와 reader가 모두 XCDR2를 알렸고 매칭됐다.

#### 2.4.5 확인 질문

- chapter 3의 Lab 3-2에서 쓴 필터와 표지 표는 이 캡처에서도 그대로 통한다. 표현이 바뀌어도 바뀌지 않는 것은 무엇인가?

#### 2.4.6 이 chapter에서 답하지 않는 것

- DHEADER가 왜 appendable 타입에 붙는지, XCDR1과 XCDR2의 정렬 규칙, 한쪽만 `-x 2`일 때 매칭되는지 -> chapter 16
- 구독 로그의 샘플 수 (97)와 캡처의 사용자 DATA 수 (98)의 차이 -> chapter 13

## 3. 규격 읽기

### 3.1 규격이 정하는 것과 구현에 맡기는 것

RTPS가 상호운용을 위해 요구하는 것은 서로 다른 벤더의 구현 사이에 오가는 메시지에 관한 최소 요구로 한정된다 (RTPS 2.5 Sec. 8.4.2). 규격은 구현마다 메모리, 대역폭, 규모의 trade-off가 다를 수 있다고 적고 (Sec. 8.4.1), 상호운용에 필요한 것만 "해야 한다"로 정한다. Lab 4-1 표의 마지막 열은 그 문장들을 1절의 네 가지로 나눈 것이다.

- **값을 정한 것.** vendorId는 OMG가 배정하고 (Sec. 8.3.3.1. 목록은 참고 자료), 규격 적합 구현은 배정받은 값을 써야 한다 (Sec. 9.4.4). 모든 참가자는 멀티캐스트 239.255.0.1 (포트는 도메인 번호와 기본 매개변수로 정해지며 도메인 0이면 7400)로 공지를 보내고 들어야 한다 (Sec. 9.6.2.4). 내장 엔드포인트의 entityId와 사용자 entityId의 마지막 바이트도 규격이 정한다 (chapter 3).
- **기본값만 준 것.** 포트 공식의 매개변수 (PB 7400 등)는 기본값을 써야 별도 설정 없이 서로 찾을 수 있고, 미리 설정한 공지 대상도 이 기본 포트를 쓴다 (Sec. 9.6.2.3, 8.5.3.4). 구현은 이 매개변수를 사용자가 바꿀 수 있게 해야 하고, 기본과 다른 유니캐스트 포트는 로케이터로 알린다 (Sec. 9.6.2.1, 9.6.2.2). 그래서 Cyclone처럼 임의 포트만 쓰면 멀티캐스트 없이 상대의 기본 포트로 찾는 방식으로는 찾을 수 없다. SPDP 공지 주기의 기본값은 30초이고 (Sec. 9.6.2.4), 주기적인 동작과 늦춰도 되는 응답의 시간은 사용자가 조정할 수 있어야 한다 (Sec. 8.4.2.1). lease 기간은 정한 값도 기본값도 없다. 공지에 실어 알리고, 공지 주기를 lease보다 짧게 잡도록 권하며 (Sec. 8.5.3.1, should), 공지에 값이 없으면 받는 쪽이 100초로 본다 (표 9.19).
- **비워 둔 영역.** 벤더는 규격이 마련한 확장 방법으로 메시지를 확장할 수 있고, 그것은 상호운용을 해치지 않는다 (Sec. 8.4.2.1, 8.6). 서브메시지 ID 0x80-0xff와 ParameterId 0x8000-0xffff, 로케이터 종류 0x03-0x01ffffff가 그 자리다 (Sec. 9.4.5.1.1, 9.4.2.11, 9.3.2.4). 해석은 보낸 쪽 vendorId에 따른다. 모르는 vendorId의 서브메시지는 건너뛰고 (Sec. 8.3.4.1), 모르는 ParameterId는 must-understand 비트 (0x4000)가 꺼져 있으면 건너뛰며, 켜져 있으면 그 서브메시지를 버린다 (표 9.6). 이 캡처의 0x8000, 0x8007, 0x8019는 모두 그 비트가 꺼져 있다.
- **규격에 없는 것.** HEARTBEAT를 DATA에 붙일지는 안내일 뿐이고 (Sec. 8.4.15.4), 주기적 HEARTBEAT 자체만 의무다 (Sec. 8.4.2.2). INFO_DST는 뒤의 서브메시지를 해석하는 데 필요하면 앞에 두어야 하고 (Sec. 8.4.2.1), 없으면 목적지는 받는 참가자 자신이다 (Sec. 8.3.4 표 8.17). 그 밖에 붙일지는 구현이 고른다. IP TTL은 규격에 언급이 없다.

프로토콜 버전은 따로 볼 만하다. 받는 쪽은 버전에 관해서는 헤더의 major가 자기가 지원하는 것보다 클 때만 메시지를 무효로 본다 (Sec. 8.3.6.3). 그래서 2.2를 쓰는 Fast DDS와 2.5를 쓰는 Cyclone이 통신한다. minor 차이로 생기는 모르는 서브메시지, 플래그, ParameterId는 건너뛰는 규칙이 흡수한다 (Sec. 8.3.4.1, 표 9.6). 다만 2.4 미만을 알리는 상대의 메시지에는 받는 쪽이 vendorId와 버전을 보고 달리 해석해야 하는 항목이 있다 (Sec. 9.3.2.3). chapter 7에서 본다.

mdds에 적용하면 이렇다. 값을 정한 것은 그대로 따른다. 기본값만 준 것은 기본값으로 시작하되 설정으로 바꿀 수 있게 만든다. 비워 둔 영역과 규격에 없는 것은 어느 벤더도 흉내 내지 않아도 되지만, 받는 쪽은 두 벤더가 보내는 모든 형태를 처리해야 한다. HEARTBEAT가 붙은 DATA와 따로 오는 HEARTBEAT, INFO_DST가 있는 메시지와 없는 메시지, 모르는 서브메시지와 PID가 그것이다.

### 3.2 사라지는 두 방법

**알리고 사라지기.** writer는 인스턴스의 수명 변화를 DATA 서브메시지로 알린다 (Sec. 8.3.8.2). inline QoS의 PID_STATUS_INFO에 D (dispose), U (unregister) 비트를 세우고 (Sec. 9.6.4.9), 값 대신 key만 실을 때는 Serialized Key 플래그를 세운다. Lab 4-2의 `DATA([_D])`가 그것이다. 내장 엔드포인트도 같은 방법을 쓴다. writer 정보와 참가자 정보의 `[UD]`는 그 writer와 참가자가 사라진다는 알림이다. reader 쪽에서 인스턴스는 writer가 dispose했으면 NOT_ALIVE_DISPOSED, 살아 있는 writer가 하나도 없음을 reader가 감지했으면 NOT_ALIVE_NO_WRITERS가 된다 (DDS 1.4 Sec. 2.2.2.5.1.3). 구독 로그의 마지막 줄이 두 경우를 가르는 것이 이 규칙이다.

**말없이 사라지기.** 참가자는 자기 lease 기간을 SPDP 공지에 실어 알린다. 상대는 그 참가자의 항목이 lease 기간 넘게 갱신되지 않으면 지우고, 그 참가자의 엔드포인트와 통신하던 자기 엔드포인트를 정리한다 (Sec. 8.5.3.3, 8.5.5.2). 판단에 쓰는 값은 사라진 쪽이 알린 lease다. Lab 4-2에서 Fast DDS가 Cyclone을 정리하기까지 걸린 시간이 Cyclone의 lease 10초와 일치한 것과 맞는다. 규격대로라면 반대로 Fast DDS가 사라질 때 Cyclone은 Fast DDS의 lease 20초를 기다린다 (이 책의 캡처로는 확인하지 않았다). 규격의 문장은 "공지"가 갱신한다고 적지만, Fast DDS 3.6.1은 상대 참가자의 어떤 RTPS 메시지든 받으면 마지막 수신 시각을 갱신하고 그 시각 + lease가 지나면 참가자를 지운다 (소스 `MessageReceiver.cpp`, `PDP.cpp`로 확인). Lab 4-2의 시점이 마지막 공지가 아니라 마지막 패킷 기준인 이유다. mdds가 어느 쪽을 따를지는 chapter 9에서 정한다. IGMP Leave는 DDS 메시지가 아니고 구독자의 DDS는 그것을 보지 않는다. DDS가 아는 것은 공지가 끊겼다는 사실뿐이다.

## 4. 스스로 확인하기

명령이나 수치를 외웠는지가 아니라 동작의 원리를 이해했는지 확인하는 질문이다. 먼저 스스로 답해 본 뒤 "해설"을 펼쳐 비교해 보라.

1. mdds가 Fast DDS처럼 모든 메시지 끝에 자기만의 서브메시지 (ID 0x80)를 붙여 보내면 Cyclone과 통신이 되는가?

   <details>
   <summary>해설</summary>

   된다. 0x80-0xff는 규격이 벤더 확장용으로 비워 둔 서브메시지 ID이고, 받는 쪽은 자기가 해석하지 않는 벤더 전용 서브메시지를 건너뛰고 나머지를 해석한다 (RTPS 2.5 Sec. 8.3.4.1, 9.4.5.1.1). Lab 4-1에서 Cyclone이 Fast DDS의 0x80이 붙은 메시지를 받고 정상 동작한 것이 그 예다. 다만 벤더 전용 영역은 헤더의 vendorId에 따라 해석되므로 전제가 하나 있다. mdds는 OMG가 배정한 vendorId가 없고, 규격은 vendorId를 규격을 지키기로 한 구현자에게만 배정한다 (Sec. 8.3.3.1). 학습용 구현이 어떤 vendorId를 쓸지는 chapter 7에서 정한다.

   </details>

2. 발행자를 (a) 정상 종료, (b) dispose 후 종료, (c) SIGKILL로 끝냈을 때 구독자가 받는 마지막 메시지와 구독 로그의 마지막 상태는 각각 무엇인가? 구독자는 세 경우를 모두 구별하는가?

   <details>
   <summary>해설</summary>

   (a) `DATA(w[UD])`와 `DATA(p[UD])`를 받고 `NOT_ALIVE_NO_WRITERS`. (b) `DATA([_D])`, `DATA(w[UD])`, `DATA(p[UD])`를 받고 `NOT_ALIVE_DISPOSED`. (c) 아무 메시지도 받지 못하고, lease 기간이 지난 뒤 `NOT_ALIVE_NO_WRITERS`. 로그의 마지막 상태만으로는 (a)와 (c)를 구별할 수 없다. 구별할 수 있는 것은 "알림을 받았는가"와 "기다린 시간"이다. (a)는 발행자가 끝나는 즉시, (c)는 lease 기간 뒤에 정리된다.

   </details>

3. mdds의 lease 기간과 SPDP 주기를 Cyclone (10초, 8초)이나 Fast DDS (20초, 3초)에 맞춰야 하는가? 두 참가자가 서로 다른 lease를 알리면 누가 어느 값으로 상대를 지우는가?

   <details>
   <summary>해설</summary>

   맞출 필요가 없다. 주기는 규격이 기본값 (30초)만 준 설정값이고 (Sec. 9.6.2.4), lease는 규격이 값을 정하지 않으며 공지에 없을 때 100초로 볼 뿐이다 (표 9.19). 지킬 것은 공지 주기를 자기 lease보다 짧게 잡으라는 권고뿐이다 (Sec. 8.5.3.1). 상대를 지울 때 쓰는 값은 상대가 알린 lease다 (Sec. 8.5.3.3). 그래서 Fast DDS는 Cyclone을 10초 뒤에 지운다. mdds가 lease를 길게 잡으면 상대가 mdds의 부재를 늦게 알고, 주기를 짧게 잡으면 공지가 자주 나간다. SPDP는 best-effort라 공지 하나가 사라질 수도 있다. 그 trade-off는 chapter 9에서 정한다.

   </details>

## 참고 자료

- [DDSI-RTPS 2.5 규격 (PDF)](https://www.omg.org/spec/DDSI-RTPS/2.5/PDF), Sec. 8.4.2 Behavior Required for Interoperability: 상호운용에 필요한 최소 동작, 조정 가능한 시간, 벤더 확장의 허용 (3.1절)
- 같은 규격, Sec. 8.3.4.1 (모르는 서브메시지의 처리), 8.3.6.3 (헤더 버전 검사), 9.4.2.11 (ParameterId 범위와 must-understand), 9.6.2 (포트와 SPDP 기본 설정) (3.1절)
- 같은 규격, Sec. 9.6.4.9 (StatusInfo_t), 8.5.3.3과 8.5.5.2 (lease 만료와 참가자 제거), 8.4.14.1 (단편화 요구) (3.2절, Lab 4-3)
- [DDS 1.4 규격 (PDF)](https://www.omg.org/spec/DDS/1.4/PDF), Sec. 2.2.2.5.1.3: 인스턴스 상태 ALIVE, NOT_ALIVE_DISPOSED, NOT_ALIVE_NO_WRITERS (3.2절)
- [DDS-XTypes 1.3 규격 (PDF)](https://www.omg.org/spec/DDS-XTypes/1.3/PDF), Sec. 7.4.3.4.1 (DHEADER), 7.6.3.1 (데이터 표현의 식별자와 PID_DATA_REPRESENTATION) (Lab 4-4)
- [OMG DDS-RTPS vendor and product IDs](https://portals.omg.org/dds/dds-rtps-vendor-and-product-ids/): 배정된 vendorId 목록 (Lab 4-1)
