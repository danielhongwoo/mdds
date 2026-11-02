# 저자 노트: 첫 DDS 통신 (chapter 03, 구현 없음)

상태: 완료

[roadmap](../roadmap.md) | chapter: 03 ([목차](../../book/README.md#목차)) | 이전: [02-multicast](02-multicast.md) | 다음: [04-vendor-differences](04-vendor-differences.md)

## 목표
Cyclone DDS와 Fast DDS가 처음 통신하는 과정을 캡처하고, Wireshark로 RTPS 메시지 (SPDP, SEDP, DATA, HEARTBEAT, ACKNACK)를 찾아 흐름을 그린다. 이 캡처가 이후 chapter의 테스트 데이터 (정답 바이트)가 된다. 메시지를 바이트 단위로 해석하지는 않는다.

## 범위
하는 것
- 시나리오 실행 스크립트 (`docker/capture-shape-scenario.ts`)와 시나리오 표
- reliable / best-effort 기본 시나리오 4개의 캡처와 흐름도 (`captures/cyclone-fastdds/`)
- 교재 chapter 03: 독자가 손으로 첫 통신을 캡처하고 (Lab 3-1), 참고 캡처 `rel-cyclone-pub`에서 다섯 구간을 찾고 (Lab 3-2), `be-cyclone-pub`과 비교한다 (Lab 3-3)

하지 않는 것
- 메시지의 바이트 해석 (chapter 07), 포트 공식 (chapter 08), SPDP의 동작 (chapter 09), reliable 동작 원리 (chapter 10), 매칭 규칙 (chapter 11), 인스턴스 상태와 dispose (chapter 13), liveliness (chapter 14)
- 벤더 차이와 나머지 시나리오 (chapter 04)

## 산출물
| 산출물 | 위치 |
|---|---|
| 시나리오 실행 스크립트 | [docker/capture-shape-scenario.ts](../../docker/capture-shape-scenario.ts) |
| 시나리오별 캡처, 양쪽 stdout 로그 | `captures/cyclone-fastdds/{rel,be}-{cyclone,fastdds}-pub.{pcap,pub.log,sub.log}` |
| 캡처 분석 메모 (흐름도, 질문) | [captures/cyclone-fastdds/README.md](../../captures/cyclone-fastdds/README.md) |
| 교재 chapter | [book/](../../book/README.md) 03 |

## 틀
관찰 chapter의 틀 ([book.md](../conventions/book.md#chapter))에서 4, 5, 6을 뺐다. 6 (두 벤더는 어떻게 하는가)은 벤더 소스를 읽는 절이고 이 chapter는 캡처만 본다. 캡처에 보이는 벤더 차이는 chapter 04가 맡는다.
1. 왜 필요한가: chapter 00 3.2절의 그림 (SPDP -> SEDP -> DATA)을 실제 패킷으로 본다는 것, 끝나면 할 수 있는 것 (캡처의 프레임이 어느 단계인지 Info 열과 필터로 가른다), 이 캡처가 뒤 chapter의 정답 데이터라는 것. 1.1 shape_main 실행과 출력 읽기 (이 chapter에서 쓰는 옵션과 로그 줄의 뜻만. 옵션 목록 자체는 chapter 01)
2. 캡처에서 보기: Lab 3-1 (손으로 첫 통신 캡처), Lab 3-2 (참고 캡처에서 다섯 구간 찾기, 내장 엔드포인트 트래픽과 사용자 트래픽 세기), Lab 3-3 (best-effort 캡처와 비교)
3. 규격 읽기: 3.1 디스커버리도 같은 프로토콜로 한다 (내장 엔드포인트, entityId와 GUID, SPDP는 best-effort이고 주기적, SEDP와 참가자 메시지 writer는 reliable, 포트와 주소는 SPDP / SEDP의 로케이터로 안다), 3.2 reliable과 best-effort (HEARTBEAT와 ACKNACK의 의무, pre-emptive ACKNACK)
4. 스스로 확인하기: 질문 네 개 (best-effort인데 HEARTBEAT / ACKNACK가 있다, 내 데이터만 어떻게 가리나, 구독자만 오래 두면 무엇이 흐르나, 참가자가 셋이면 어느 구간이 느나)

**결정: Lab 3-1은 스크립트가 아니라 손으로 한다 (터미널 세 개, Lab 2-1과 같은 방식).** 독자가 구독자 혼자일 때의 출력, 발행자가 시작되고 matched가 찍히는 순간, 발행자가 끝난 뒤의 `NOT_ALIVE_NO_WRITERS`를 터미널에서 직접 본다. 독자의 캡처는 `my-rel-cyclone-pub.pcap`이고, 참고 캡처는 스크립트가 같은 순서를 정해진 간격으로 실행해 만든 것이다. 정답 데이터는 여전히 스크립트로 만든 참고 캡처뿐이다 ([captures.md](../conventions/captures.md)).
- 버린 대안: 스크립트에 `--my` 같은 이름 접두 옵션을 넣어 독자가 스크립트로 캡처하기. 재현성은 좋지만 터미널에서 진행을 보지 못하고, 스크립트와 노트를 함께 고쳐야 한다. 필요해지면 chapter 04 이후에 넣는다.

**결정: 프레임 번호와 수치는 Lab 3-2의 구간 표와 수치 표에만 둔다.** 본문 문장은 "구간 3"처럼 표를 가리킨다. 참고 캡처 `rel-cyclone-pub`, `be-cyclone-pub`은 03 본문, 캡처 README, 09 / 10 / 11 노트가 같은 프레임 번호로 인용하므로 사실상 고정된 데이터다. 다시 만들면 고칠 곳이 이 네 곳이다.

**결정: 참고 캡처 `rel-cyclone-pub`을 바꾸지 않는다.** 이 캡처는 Cyclone이 Fast DDS의 첫 SEDP HEARTBEAT에 답하지 않아 SEDP 구간이 1.1초 늘어진 경우다 (Cyclone 발행 캡처 6개 중 2개가 이렇다. [11 노트](11-sedp-matching.md#확인할-것)). 깨끗한 캡처로 바꾸면 결과를 골라 쓰는 일이 되고, 뒤 노트가 이 번호를 이미 인용한다. 대신 본문에 "SEDP 구간의 HEARTBEAT / ACKNACK 왕복 횟수와 첫 사용자 seq는 실행마다 다르다"를 적고, 구간 경계는 `DATA(p)`, writer 정보, reader 정보, 첫 사용자 DATA, `[UD]` 같은 표지로 정한다. 저자가 손으로 돌린 캡처 (2026-11-02)는 seq 3부터, 내장 엔드포인트 트래픽 57 프레임이었다.

**결정: 캡처 README의 질문은 lab 끝의 "이 chapter에서 답하지 않는 것" 목록으로 보여 주고, 스스로 확인하기에는 이 chapter만으로 답할 수 있는 질문을 둔다.** 규칙은 [book.md](../conventions/book.md#lab).

**결정: "디스커버리가 차지하는 몫"은 비율이 아니라 묶음별 프레임 수와 바이트 수의 표로 보인다.** 비율은 실행 길이 (사용자 데이터는 약 9초, SPDP는 캡처 내내), loopback 복사본, reliable의 ACKNACK, Fast DDS의 60바이트 벤더 전용 서브메시지, frame.len에 든 42바이트 헤더에 따라 달라져서 DDS의 성질처럼 읽히면 안 된다. 표는 한 번 드는 비용 (SEDP)과 계속 드는 비용 (SPDP)을 나누어 보인다. chapter 00 3.3절의 "적지 않은 몫"을 측정으로 뒷받침하되 일반화하지 않는다.

## 작업 순서

### Step 1. 시나리오 스크립트
- [x] `docker/capture-shape-scenario.ts <시나리오>`: 아래 "시나리오 실행 흐름"대로.

### Step 2. 첫 통신
- [x] `capture-shape-scenario.ts rel-cyclone-pub`: Cyclone 발행 -> Fast DDS 구독. `rel-cyclone-pub.sub.log`에 샘플이 찍힌다.
- [x] `capture-shape-scenario.ts rel-fastdds-pub`: 반대 방향
- [x] be-cyclone-pub, be-fastdds-pub
- [x] 캡처에서 SPDP (7400 멀티캐스트), SEDP, DATA, HEARTBEAT, ACKNACK를 Wireshark `rtps` 필터로 식별한다.

### Step 3. 캡처 분석 메모
- [x] `captures/cyclone-fastdds/README.md`에 시나리오마다 실행 명령, 흐름 (Mermaid 다이어그램), 질문을 적는다. 형식과 "관찰과 질문까지만" 기준은 [captures.md](../conventions/captures.md#readme-형식).

## lab
규칙은 [conventions/book.md](../conventions/book.md#lab).

| lab | 제목 | 하는 일 -> 기대 결과 | 쓰는 산출물 |
|---|---|---|---|
| Lab 3-1 | 첫 DDS 통신 | 터미널 세 개 (fastdds tcpdump, fastdds_shape -S, cyclone_shape -P)로 손으로 실행 -> 구독 터미널에 matched, 샘플 줄, NOT_ALIVE_NO_WRITERS, Done.이 차례로 찍히고 `my-rel-cyclone-pub.pcap`이 생김 | 벤더 컨테이너의 shape_main. 참고 캡처를 만든 방법으로 `docker/capture-shape-scenario.ts`를 링크 |
| Lab 3-2 | 캡처에서 RTPS 메시지 찾기 | 참고 캡처를 Wireshark 필터 `rtps`와 entityId 필터로 봄 -> 다섯 구간 (구독자 혼자, 서로 찾기, 엔드포인트 교환, 사용자 데이터, 종료)을 프레임 번호로 찾음. 내장 엔드포인트 트래픽 (SPDP, SEDP, 참가자 메시지)과 사용자 트래픽을 프레임 수와 바이트로 셈 (chapter 00 3.3절 "대가"를 직접 확인) | `captures/cyclone-fastdds/rel-cyclone-pub.*` |
| Lab 3-3 | best-effort와 비교 | `be-cyclone-pub`을 같은 필터로 봄 -> 사용자 트래픽에는 DATA만 있고, HEARTBEAT / ACKNACK는 SEDP와 참가자 메시지 엔드포인트에만 남음 | `be-cyclone-pub.*` |

## 시나리오 실행 흐름 (`capture-shape-scenario.ts`)
스크립트는 [docker/capture-shape-scenario.ts](../../docker/capture-shape-scenario.ts)다. `./capture-shape-scenario.ts <이름>`으로 실행하고, `--list`로 이름을 본다. 결과 파일이 이미 있으면 멈추며 `--force`로 덮어쓴다 (캡처는 커밋되는 정답 데이터라 실수로 지우지 않게). 시작 전에 두 컨테이너가 떠 있는지, 이전 실행의 shape_main이나 tcpdump가 남아 있는지 검사한다. 아래 흐름과 시나리오 표를 바꾸면 스크립트의 `SCENARIOS`도 함께 고친다. chapter 04의 시나리오도 같은 스크립트와 표를 쓴다.

스크립트 언어 (TypeScript + Deno), 파일 이름 ("하는 일 + 대상"), 통과해야 하는 검사는 [typescript.md](../conventions/typescript.md)를 따른다.

시나리오마다 아래 변수를 정한다. 값은 시나리오 표에 있다.
```
NAME              시나리오 이름 (파일 이름에 쓴다)
OUT               결과 디렉터리 (컨테이너 안 경로). /captures/cyclone-fastdds
SUB, PUB          구독 / 발행 컨테이너 이름 (cyclone 또는 fastdds)
SUB_BIN, PUB_BIN  cyclone_shape 또는 fastdds_shape
COMMON            양쪽 공통 옵션 (-d, -t, -x, -r / -b)
PUB_ONLY          발행 전용 옵션 (-c, --write-period, --num-iterations, --final-instance-state, --additional-payload-size)
SUB_ONLY          구독 전용 옵션 (보통 없음)
END               발행자 종료 방식: wait (스스로 끝남) 또는 kill9 (10초 뒤 SIGKILL)
AFTER             발행자 종료 후 관찰 대기 (초)
```
```
cd "$(dirname "$0")"            # compose.yaml이 있는 docker/ 에서 실행
mkdir -p ../captures/cyclone-fastdds   # 호스트에서 만든다 (컨테이너가 만들면 Linux에서 root 소유)
X="docker compose exec -T"      # 스크립트에서는 TTY가 없으므로 -T 필수

 1. $X -d $SUB tcpdump -i eth0 -U -Z root -w $OUT/$NAME.pcap "udp or igmp"
 2. sleep 1
 3. $X -d $SUB sh -c "exec stdbuf -oL $SUB_BIN -S $COMMON $SUB_ONLY > $OUT/$NAME.sub.log 2>&1"
 4. sleep 3                      # 구독자가 먼저 SPDP를 내보내게 한다
 5. $X -d $PUB sh -c "exec stdbuf -oL $PUB_BIN -P $COMMON $PUB_ONLY > $OUT/$NAME.pub.log 2>&1"
 6. END=wait : while $X $PUB pgrep -x $PUB_BIN >/dev/null; do sleep 1; done
    END=kill9: sleep 10; $X $PUB pkill -9 -x $PUB_BIN
 7. sleep $AFTER                 # dispose 전파나 lease 만료를 관찰할 시간
 8. $X $SUB pkill -INT -x $SUB_BIN || true ; sleep 2    # 구독자 정상 종료
 9. $X $SUB pkill -TERM -x tcpdump || true ; sleep 1    # 파일이 닫히도록 SIGTERM
10. $X cyclone pgrep -x cyclone_shape; $X fastdds pgrep -x fastdds_shape   # 컨테이너마다 자기 벤더 실행 파일만 있다
    # 아무것도 안 나와야 정상. pgrep은 못 찾으면 종료 코드 1이므로 set -e 환경에서는 결과를 따로 판정한다
```
- **종료 신호는 SIGINT다.** shape_main은 SIGINT 핸들러만 설치하므로 SIGTERM을 보내면 정리 없이 죽어 dispose / unregister / SEDP 제거가 나가지 않는다. 정상 종료는 `--num-iterations N`으로 스스로 끝내거나 `pkill -INT`로 한다.
- `pkill -x` / `pgrep -x`는 프로세스 이름 전체 일치다. `-f`를 쓰면 `sh -c "... cyclone_shape ..."` 래퍼 셸까지 잡힌다. 실행 파일 이름이 15자 이하(`cyclone_shape` 13자, `fastdds_shape` 13자)라 `-x`가 된다.
- `exec -d`로 띄운 프로세스는 핸들이 없으므로 6단계처럼 `pgrep` 폴링으로 종료를 기다린다.
- **stdout은 파일로 받는다.** `exec -d`로 띄운 프로세스의 출력은 어디에도 남지 않는다. `stdbuf -oL`이 없으면 파일 리다이렉트 때 블록 버퍼링되어 SIGKILL 시 로그가 비어 있다.
- `tcpdump -U`는 패킷마다 바로 쓰기 (버퍼 손실 방지), `-Z root`는 tcpdump가 권한을 낮추면서 바인드 마운트에 쓰지 못하는 문제 방지.
- `tcpdump`는 pcap 형식으로 쓴다. Wireshark에서 바로 열린다. 저장소에는 `.pcap` 그대로 둔다.
- 캡처는 구독자 컨테이너에서 한다. 발행자를 바꾸는 시나리오에서는 캡처 컨테이너도 바뀐다.

## 시나리오 표
chapter 03과 04의 시나리오를 모두 여기에 둔다 (스크립트의 `SCENARIOS`와 일대일). 옵션은 OMG dds-rtps README (2026-10-28 확인)의 shape_main 옵션이고, `-h`로 실행 파일과 같은지 확인했다.

- COMMON 기본값: `-d 0 -t Square -x 1`에 표의 신뢰성 옵션을 더한다. xcdr2 시나리오만 `-x 2`. 데이터 표현을 명시하는 이유는 실행 파일의 기본값을 모르기 때문이다.
- PUB_ONLY 기본값: `-c BLUE --write-period 100 --num-iterations 100` (10 Hz로 약 10초 보내고 스스로 끝난다). 표의 값은 여기에 더하거나 대체한다.
- **`-c`는 발행 전용이다.** 구독자에게 주면 ContentFilteredTopic이 되어 토픽 이름이 `Square_filtered`로 바뀌고 SEDP에 `PID_CONTENT_FILTER_PROPERTY` (DDSSQL `color = %0`)가 실린다. 색이 하나뿐이라 거르는 것은 없고 기본 캡처만 복잡해지므로 뺐다. ContentFilteredTopic 자체를 볼 캡처가 필요해지면 별도 시나리오로 추가한다.
- **구독 로그의 샘플 수는 실제 전달 수보다 적을 수 있다.** 구독자는 기본 history (KEEP_LAST)로 100ms마다 읽는데 발행도 100ms 주기라, 한 주기에 둘이 들어오면 앞의 것이 덮어써진다. 전달 여부는 캡처의 DATA seq와 ACKNACK로 판단한다. 또 발행자는 구독자를 발견하기 전부터 쓰므로 (VOLATILE) 앞쪽 seq 몇 개는 전달 대상이 아니다.
- SUB_ONLY: 모든 시나리오에서 없음.

| 이름 | chapter | 발행 (PUB) | 구독 (SUB) | COMMON 추가 | PUB_ONLY 변경 | END | AFTER | 관찰 목표 |
|---|---|---|---|---|---|---|---|---|
| be-cyclone-pub | 03 | cyclone | fastdds | `-b` | 기본값 | wait | 5 | best-effort DATA, HEARTBEAT 없음 |
| be-fastdds-pub | 03, 04 | fastdds | cyclone | `-b` | 기본값 | wait | 5 | 벤더별 차이 비교 |
| rel-cyclone-pub | 03 | cyclone | fastdds | `-r` | 기본값 | wait | 5 | HEARTBEAT / ACKNACK 왕복 (이 캡처에 GAP은 없다) |
| rel-fastdds-pub | 03, 04 | fastdds | cyclone | `-r` | 기본값 | wait | 5 | 위와 같음 |
| dispose-on-exit | 04 | cyclone | fastdds | `-r` | 기본값 + `--final-instance-state d` | wait | 10 | DATA에 STATUS_INFO dispose 비트, 이어서 unregister, SEDP의 writer 제거. 구독 로그의 instance state 변화 |
| lease-expiry | 04 | cyclone | fastdds | `-r` | `--write-period 100` 만 (`--num-iterations` 없음) | kill9 | 25 (Cyclone lease 10초 + 여유) | dispose 없이 사라진 참가자를 구독자가 lease 만료로 정리. 증거는 캡처의 마지막 SPDP 시각과 그 뒤 구독자의 동작 (ACKNACK 중단 등) |
| large-sample | 04 | cyclone | fastdds | `-r` | 기본값 + `--additional-payload-size 100000`, `--num-iterations 30` | wait | 5 | DATA_FRAG, HEARTBEAT_FRAG, NACK_FRAG (두 벤더 사이). IP 단편화도 섞이므로 Wireshark에서 IP 재조립 설정을 켜고 본다 |
| xcdr2 | 04 | cyclone | fastdds | `-r -x 2` (`-x 1` 대체) | 기본값 | wait | 5 | 캡슐화 식별자 0x0007 (CDR2_LE) 등 XCDR2 페이로드 |

- lease 기본값: Cyclone `Discovery/LeaseDuration` 10초, Fast DDS 20초 (공식 문서로 확인). 구독자는 상대가 SPDP로 광고한 lease 값을 쓰므로, Cyclone 발행자를 죽이면 Fast DDS 구독자는 약 10초 뒤에 정리한다.
- 각 시나리오는 한 번에 참가자 둘만 띄운다. 실행 흐름 10번의 `pgrep`이 비어 있는지 확인하고 시작한다.
- 선택: dispose-on-exit의 `d`를 `u`로 바꾼 unregister-only 캡처, 발행 `-r` + 구독 `-b` 같은 QoS 불일치 캡처 (REQUESTED_INCOMPATIBLE_QOS 관찰)는 시간이 남으면 추가한다.

## 예상 문제와 대응
빌드 문제는 [vendor-build.md](../../docs/environment/vendor-build.md#빌드-문제와-대응), 네트워크와 캡처 문제는 [docker-network.md](../../docs/environment/docker-network.md#문제와-대응)를 본다. 시나리오 실행에 관한 것만 적는다.

| 증상 | 원인 후보 | 대응 |
|---|---|---|
| 로그 파일이 비어 있음 | 블록 버퍼링 | `stdbuf -oL`을 확인한다 |
| `pkill`이 프로세스를 못 찾음 | 15자 넘는 실행 파일 이름 | 짧은 이름으로 복사했는지 확인한다. `-x`는 이름 전체 일치다 |
| 스크립트에서 "input device is not a TTY" | `docker compose exec`에 `-T` 누락 | `-T`를 붙인다 |
| SPDP는 보이는데 매칭이 안 됨 | QoS 불일치, 데이터 표현 불일치 (`-x`) | 양쪽 옵션을 같게 맞춘다. 이것 자체도 좋은 캡처다 |

## 확인할 것
- Wireshark GUI의 Statistics > Capture File Properties의 Displayed 열 Bytes가 tshark의 `frame.len` 합 (사용자 트래픽 30,132 등)과 같은지. tshark로는 확인했고 GUI는 보지 않았다.
- Ubuntu 24.04의 apt Wireshark (4.2 계열)에서 Info 열 표기 (`DATA(p[UD])`, `Unknown[80]`, `Malformed Packet`의 위치)가 4.6.9와 같은지. 본문은 4.6.9 기준이라고 밝혔다.
- 7400을 다른 프로세스 (chapter 02의 `mcast-check recv`)가 잡고 있을 때 fastdds_shape는 오류 없이 시작하지만 7400 소켓을 열지 않는다 (2026-11-02, `ss -ulpn`으로 확인). 그 상태에서 디스커버리가 되는지 (상대의 유니캐스트 응답으로 될 수도 있다)는 시험하지 않았다. 본문에는 관찰한 사실만 적었다.
- 손으로 한 Lab 3-1에서 터미널 Ctrl+C가 pty를 거쳐 shape_main에 SIGINT로 가는 것은 Lab 2-1의 tcpdump와 같은 경로다. 저자는 `pkill -INT`로 같은 결과 (`DATA(p[UD])`, IGMP leave)를 확인했고, 터미널에서 직접 Ctrl+C를 누른 확인은 하지 않았다.

## 완료 기준
- `docker compose up -d`와 `capture-shape-scenario.ts <이름>`만으로 (Deno 2 필요) 누구나 같은 캡처를 재현할 수 있다.
- rel / be 시나리오 4개의 캡처와 로그가 저장소에 있고, README에 흐름 (프레임 번호가 붙은 다이어그램)과 질문이 적혀 있다.
- 캡처에서 SPDP / SEDP / DATA / HEARTBEAT / ACKNACK를 Wireshark로 식별했다.

## 진행 기록
결정과 막힌 점만 한 줄씩 적고 근거 문서로 링크한다. 세부 경위는 커밋 메시지에 둔다.
- 2026-10-28: 캡처 볼륨을 `captures` 전체로 바꿈 (chapter에 묶지 않음). 시나리오는 `/captures/cyclone-fastdds/`에 쓴다 -> [docker-network.md "compose 설계"](../../docs/environment/docker-network.md#compose-설계)
- 2026-10-28: 첫 통신 완료. 시나리오 스크립트 작성, rel 두 시나리오 캡처. shape_main `-c`를 발행 전용으로 바꿈 (구독자에게는 ContentFilteredTopic이 됨) -> [시나리오 표](#시나리오-표), [캡처 README](../../captures/cyclone-fastdds/README.md)
- 2026-10-29: 시나리오 스크립트를 POSIX sh에서 TypeScript + Deno로 바꿈 (가독성) -> [시나리오 실행 흐름](#시나리오-실행-흐름-capture-shape-scenariots)
- 2026-10-29: 캡처 메모는 관찰과 질문까지만 적고, 해석과 가설은 해당 chapter의 저자 노트 "확인할 것"으로 옮김 (chapter 7, 9, 10, 11, 13) -> [captures.md](../conventions/captures.md#readme-형식)
- 2026-10-29: 산출물을 서술형 이름으로 바꿈 (`capture-shape-scenario.ts`, `captures/cyclone-fastdds/`) -> [typescript.md](../conventions/typescript.md)
- 2026-10-30: 캡처를 `tests/captures/`에서 저장소 루트 `captures/`로 옮김 (테스트가 아니라 lab과 chapter가 함께 쓰는 관찰 자료) -> [captures.md](../conventions/captures.md)
- 2026-10-30: chapter 01-04를 함께 맡던 노트에서 나눔. 시나리오 표는 스크립트와 일대일이라 이 노트에만 두고 chapter 열을 더함
- 2026-11-02: 아키텍트, DDS 전문가, 시니어 엔지니어, 독자 네 관점의 검토로 틀을 정하고 본문 초안 작성 -> [틀](#틀). Lab 3-1은 손으로, 규격 읽기는 내장 엔드포인트와 reliable / best-effort 둘로 줄임 (헤더 필드와 포트 공식은 04, 07, 08의 몫). 관찰 chapter가 6절을 빼는 것과 "답하지 않는 것" 목록을 [book.md](../conventions/book.md#chapter)에 적음
