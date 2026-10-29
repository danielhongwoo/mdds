# Phase 00 - 준비 (구현 없음, 관찰 단계)

상태: 대기

[roadmap](../roadmap.md) | 다음: [Phase 01](phase-01-dcps-concepts.md)

## 목표
규격을 읽기 전에 실제 DDS 트래픽을 눈으로 본다. Cyclone DDS와 Fast DDS가 Docker 컨테이너 두 개에서 서로 통신하게 만들고, 그 패킷을 캡처해 이후 단계의 테스트 데이터(정답 바이트)로 쓴다. 프로토콜 코드는 쓰지 않는다.

## 범위
하는 것
- Docker 기반 실행 환경 구성 (`docker/`)
- 두 구현 사이의 통신 확인과 시나리오별 패킷 캡처 (`tests/captures/`)
- Cargo 워크스페이스 뼈대와 프로젝트 규약 확정

하지 않는 것
- RTPS / CDR 코드 작성, 캡처 파일을 읽는 테스트 코드 (Phase 02, 03에서)
- 규격 정독 (Phase 01부터)
- ROS 2 환경 (Phase 08에서)
- `ddsperf` 등 벤더 고유 도구. 모든 시나리오는 OMG `shape_main` 하나로 만든다 (큰 샘플도 `--additional-payload-size`로 가능).

## 산출물
| 산출물 | 위치 |
|---|---|
| Docker 이미지 정의, compose 파일, 시나리오 실행 스크립트, 사용법 | `docker/` |
| 시나리오별 캡처 파일, 양쪽 stdout 로그, 설명 | `tests/captures/phase00/<이름>.pcap`, `<이름>.pub.log`, `<이름>.sub.log`, `README.md` |
| 빈 crate 세 개가 있는 Cargo 워크스페이스 | `Cargo.toml`, `crates/` |
| 확정된 프로젝트 규약 | 이 문서의 "프로젝트 규약" 절, `CLAUDE.md` |
| 캡처 분석 메모 (어느 프레임에 무엇이 보이는지) | `tests/captures/phase00/README.md`, 이 문서의 "진행 기록" |

## 작업 순서

### Step 1. 실행 호스트 준비
**Mac (Apple Silicon, arm64)의 Docker Desktop에서 빌드, 실행, 캡처를 모두 하는 것을 기본**으로 한다 (2026-10-28 결정). shape_main과 두 벤더의 DDS 라이브러리를 이미지 안에서 소스로 빌드하므로 아키텍처 제약이 없고, arm64로 네이티브 빌드된다 (에뮬레이션 없음). 상세는 [environment/docker-network.md](../environment/docker-network.md#실행-호스트-mac-docker-desktop).

- [ ] Docker Desktop 설치, `docker compose version`이 동작한다.
- [ ] Docker Desktop 자원: 메모리 8 GB 이상 (Fast DDS C++ 빌드용, 16 GB 권장), 디스크 여유 10 GB 이상 (빌드 중간 산출물 포함).
- [ ] Wireshark (`brew install --cask wireshark`). 캡처 파일을 여는 용도로만 쓴다.
- [ ] 확인: `docker run --rm ubuntu:24.04 uname -m` 이 `aarch64`를 출력한다.

### Step 2. 저장소 뼈대
- [x] `rust-toolchain.toml` (1.99.1 고정), 루트 `Cargo.toml` (workspace), `crates/mdds-cdr`, `crates/mdds-rtps`, `crates/mdds` (각각 `lib.rs`만 있는 빈 crate)
- [x] `tools/mcast-check`: 멀티캐스트 전달 확인 도구 (Step 4에서 쓴다)
- [x] `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` 통과
- [x] `docker/`, `tests/captures/phase00/` 디렉터리

### Step 3. Docker 이미지와 compose
- [ ] `docker/cyclone/Dockerfile`, `docker/fastdds/Dockerfile`: 벤더마다 shape_main을 **Docker 안에서 소스로 빌드**한다. 설계는 [environment/vendor-build.md](../environment/vendor-build.md).
  - [x] 첫 빌드 (`docker compose build`)가 끝나고, 빌드 시간과 이미지 크기를 vendor-build.md "빌드 결과"에 적는다.
  - [x] 각 컨테이너에서 `ldd /usr/local/bin/<벤더>_shape` 로 `not found`가 없는지 확인한다.
  - [x] `cyclone_shape -h`, `fastdds_shape -h` 로 옵션이 README와 같은지 확인한다.
  - [x] 버전 확인: Fast DDS 쪽 원래 실행 파일 이름에 라이브러리 버전이 들어 있다 (`eprosima_fastdds-3.6.1.0_shape_main_linux`). Cyclone 쪽은 직접 컴파일하므로 `docker/README.md`의 버전 표와 이미지 라벨로 확인한다.
- [ ] `docker/compose.yaml`: [docker-network.md](../environment/docker-network.md#네트워크-구성)의 설계대로.
- [x] `docker/phase00-capture.ts <시나리오>`: 아래 "시나리오 실행 흐름"대로.
- [ ] `docker compose up -d --build` 로 두 컨테이너가 뜬다.
- [ ] `docker/README.md`: 사용법 (Mac의 Docker Desktop에서 빌드와 실행), 네트워크 요약과 docker-network.md 링크, 빌드에 쓴 소스 태그와 커밋 해시.

### Step 4. 네트워크 검증 (DDS를 띄우기 전에)
- [x] 컨테이너 두 개가 서로 ping 된다 (`docker compose exec cyclone ping -c1 fastdds`). 이후 명령은 모두 `docker/`에서 친다.
- [x] 멀티캐스트가 컨테이너 사이를 넘어간다. `mcast-check`로 239.255.0.1:7400에 보내고 받는다 ([검증 명령](../environment/docker-network.md#검증-명령)). 받는 쪽에서 `ip maddr show eth0`에 239.255.0.1이 보이는 것도 확인한다.
- [x] 캡처가 된다. `tcpdump`가 위 mcast-check 패킷과 IGMP 가입 패킷을 잡고, pcap 파일이 Mac에서 열린다 (`tests/captures/phase00/mcast-check.*.pcap`).
- 여기서 실패하면 [docker-network.md "문제와 대응"](../environment/docker-network.md#문제와-대응)을 본다. DDS를 띄우기 전에 해결한다.

### Step 5. 첫 통신
- [x] `phase00-capture.ts rel-cyclone-pub`: Cyclone 발행 -> Fast DDS 구독. `rel-cyclone-pub.sub.log`에 샘플이 찍힌다.
- [x] `phase00-capture.ts rel-fastdds-pub`: 반대 방향
- [x] 캡처에서 SPDP (7400 멀티캐스트), SEDP, DATA, HEARTBEAT, ACKNACK를 Wireshark `rtps` 필터로 식별한다. 프레임 번호와 관찰 내용은 [tests/captures/phase00/README.md](../../tests/captures/phase00/README.md).

### Step 6. 시나리오 캡처
- 아래 시나리오 표의 각 항목을 `phase00-capture.ts`로 실행해 `tests/captures/phase00/`에 저장한다.
- [x] be-cyclone-pub, be-fastdds-pub
- [x] rel-cyclone-pub, rel-fastdds-pub (Step 5에서 캡처)
- [x] dispose-on-exit
- [x] lease-expiry
- [x] large-sample
- [x] xcdr2

### Step 7. 캡처 분석 메모
- [x] `tests/captures/phase00/README.md`에 시나리오마다: 실행 명령, 벤더와 버전, 캡처한 컨테이너, 흐름을 Mermaid 시퀀스 다이어그램 (`mermaid` 코드 블록)으로 적는다. 반복은 `loop`로 줄이고 메시지마다 프레임 번호와 시각을 붙인다 (첫 SPDP, SEDP 교환, HEARTBEAT / ACKNACK 왕복, dispose, 첫 DATA_FRAG). 다이어그램 아래에 "질문"을 적는다. 형식은 rel-cyclone-pub 절을 따른다.
  - 왜: 프레임 번호 표만으로는 무엇이 무엇에 대한 응답인지 보이지 않는다. 캡처 한 개에 프레임이 수백 개이고 대부분 같은 일의 반복이다. Mermaid는 GitHub과 VS Code에서 그림으로 그려지고, 원문도 메시지 한 줄씩이라 터미널에서 읽힌다. 문법이 ASCII라 전역 문자 규칙도 지킨다.
  - 버린 대안: ASCII 그림. 한글이 두 칸을 차지해 오른쪽 세로선을 표시 폭으로 맞춰야 하고, 한 줄만 고쳐도 다시 맞춰야 한다.
  - **결정: 캡처 메모는 관찰과 질문까지만 적는다.** 다이어그램에는 메시지 이름, 방향, 필드 값 (예: `HB 12-11`, `ACKNACK 13/0`)만 쓰고 "매칭", "요청", "보낼 것 없음" 같은 해석은 쓰지 않는다. 해석이 필요한 것은 "질문"에 적고 다루는 단계를 붙인다. 이미 세운 가설은 그 단계 파일의 "확인할 것"에 [추정]으로 둔다.
    - 왜: Phase 00은 규격을 읽기 전 단계라 해석이 맞는지 확인할 수 없다. 해석을 여기 쓰면 뒤 단계 문서, book의 장과 같은 설명이 두 곳에 생긴다. 질문 목록은 그 단계에서 풀 숙제가 된다.
    - 버린 대안: 메커니즘 설명까지 적기. 캡처를 막 본 시점이라 설명이 쉽게 나오지만, 규격으로 확인하지 않은 추정이 사실처럼 남는다.
- [x] 두 벤더의 차이를 메모한다: 벤더 ID, 프로토콜 버전, 벤더 전용 PID, 데이터 표현 ID, SPDP의 lease 기간 값, 유니캐스트 포트 (Fast DDS 7410 / 7411, Cyclone은 임의 포트), Fast DDS SPDP에 실리는 공유 메모리 로케이터 (벤더 전용 kind).

### Step 8. 규약 확정
- [ ] 아래 "프로젝트 규약" 절의 "제안"을 "확정"으로 바꾸고 `CLAUDE.md`와 맞춘다.

## 실행 환경
- 이미지 빌드 (소스 버전, Dockerfile 구성, 빌드 결과): [environment/vendor-build.md](../environment/vendor-build.md)
- 실행 호스트 (Mac Docker Desktop), 네트워크 구성, 멀티캐스트 원리, 검증 명령, 네트워크 문제 대응: [environment/docker-network.md](../environment/docker-network.md)
- 명령어 위주 사용법: [docker/README.md](../../docker/README.md)

요약: 컨테이너 `cyclone` (172.28.0.10)과 `fastdds` (172.28.0.11)가 사용자 정의 bridge `dds-net`에 붙어 있다. 캡처는 구독자 컨테이너 안의 tcpdump로 하고, 바인드 마운트 (`tests/captures` -> `/captures`)로 `tests/captures/phase00/`에 바로 생긴다.

## 시나리오 실행 흐름 (`phase00-capture.ts`)
스크립트는 [docker/phase00-capture.ts](../../docker/phase00-capture.ts)다. `./phase00-capture.ts <이름>`으로 실행하고, `--list`로 이름을 본다. 결과 파일이 이미 있으면 멈추며 `--force`로 덮어쓴다 (캡처는 커밋되는 정답 데이터라 실수로 지우지 않게). 시작 전에 두 컨테이너가 떠 있는지, 이전 실행의 shape_main이나 tcpdump가 남아 있는지 검사한다. 아래 흐름과 시나리오 표를 바꾸면 스크립트의 `SCENARIOS`도 함께 고친다.

**결정: 파일 이름에 단계를 붙인다 (`phase00-capture.ts`).**
- 왜: 시나리오 표와 출력 경로 (`tests/captures/phase00/`)가 Phase 00에 묶여 있다. `docker/`는 이후 단계에서도 계속 쓰므로, 일반적인 이름 (`run-scenario.ts`)이면 다른 단계의 시나리오도 이 파일로 돌리는 것처럼 읽힌다. 단계 번호를 앞에 두면 파일이 단계 순서로 정렬된다.
- 버린 대안: 시나리오 표를 단계별 파일로 빼고 실행기 하나를 같이 쓰기. 다른 단계에 캡처 시나리오가 생길 때 다시 본다. 지금은 쓰는 곳이 하나뿐이다.

**결정: TypeScript + Deno로 쓴다.**
- 왜: 시나리오를 객체로 두면 기본값과 차이가 분리되어 표처럼 읽힌다. 인자를 배열로 넘기므로 셸의 단어 분리와 이중 따옴표 문제가 사라진다. Deno는 실행 파일 하나이고, 외부 모듈 없이 표준 API (`Deno.Command`)만 쓴다. 권한은 `docker` 실행과 파일 읽기/쓰기로 제한한다. `docker/deno.json`에서 lint 규칙 `explicit-function-return-type`을 켜 모든 함수에 반환 타입을 적는다 (함수 머리만 보고 무엇을 돌려주는지 알 수 있게. Rust와 같은 습관). `docker/`에서 `deno fmt --check`, `deno check phase00-capture.ts`, `deno lint`가 통과해야 한다.
- 비용: 호스트에 Deno 2가 필요하다 (`brew install deno`). 컨테이너 안에서 리다이렉트하는 부분은 여전히 `sh -c` 한 번을 거친다.
- 버린 대안: POSIX sh (처음 버전. 시나리오가 `case` 한 줄에 몰리고 옵션을 문자열로 이어 붙여 읽기 어려웠다), Rust로 `tools/`에 두기 (프로젝트 언어와 같지만 프로세스 호출 코드가 길어져 흐름이 오히려 안 보인다).

시나리오마다 아래 변수를 정한다. 값은 시나리오 표에 있다.
```
NAME              시나리오 이름 (파일 이름에 쓴다)
OUT               결과 디렉터리 (컨테이너 안 경로). Phase 00은 /captures/phase00
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
mkdir -p ../tests/captures/phase00   # 호스트에서 만든다 (컨테이너가 만들면 Linux에서 root 소유)
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
옵션은 OMG dds-rtps README (2026-10-28 확인)의 shape_main 옵션이다. Step 3에서 `-h`로 실행 파일과 같은지 재확인한다.

- COMMON 기본값: `-d 0 -t Square -x 1`에 표의 신뢰성 옵션을 더한다. xcdr2 시나리오만 `-x 2`. 데이터 표현을 명시하는 이유는 실행 파일의 기본값을 모르기 때문이다.
- PUB_ONLY 기본값: `-c BLUE --write-period 100 --num-iterations 100` (10 Hz로 약 10초 보내고 스스로 끝난다). 표의 값은 여기에 더하거나 대체한다.
- **`-c`는 발행 전용이다.** 구독자에게 주면 내용 필터 (ContentFilteredTopic)가 되어 토픽 이름이 `Square_filtered`로 바뀌고 SEDP에 `PID_CONTENT_FILTER_PROPERTY` (DDSSQL `color = %0`)가 실린다. 색이 하나뿐이라 거르는 것은 없고 기본 캡처만 복잡해지므로 뺐다. 내용 필터 자체를 볼 캡처가 필요해지면 별도 시나리오로 추가한다.
- **구독 로그의 샘플 수는 실제 전달 수보다 적을 수 있다.** 구독자는 기본 history (KEEP_LAST)로 100ms마다 읽는데 발행도 100ms 주기라, 한 주기에 둘이 들어오면 앞의 것이 덮어써진다. 전달 여부는 캡처의 DATA seq와 ACKNACK로 판단한다. 또 발행자는 구독자를 발견하기 전부터 쓰므로 (VOLATILE) 앞쪽 seq 몇 개는 전달 대상이 아니다.
- SUB_ONLY: 모든 시나리오에서 없음.

| 이름 | 발행 (PUB) | 구독 (SUB) | COMMON 추가 | PUB_ONLY 변경 | END | AFTER | 관찰 목표 |
|---|---|---|---|---|---|---|---|
| be-cyclone-pub | cyclone | fastdds | `-b` | 기본값 | wait | 5 | best-effort DATA, HEARTBEAT 없음 |
| be-fastdds-pub | fastdds | cyclone | `-b` | 기본값 | wait | 5 | 벤더별 차이 비교 |
| rel-cyclone-pub | cyclone | fastdds | `-r` | 기본값 | wait | 5 | HEARTBEAT / ACKNACK 왕복, GAP |
| rel-fastdds-pub | fastdds | cyclone | `-r` | 기본값 | wait | 5 | 위와 같음 |
| dispose-on-exit | cyclone | fastdds | `-r` | 기본값 + `--final-instance-state d` | wait | 10 | DATA에 STATUS_INFO dispose 비트, 이어서 unregister, SEDP의 writer 제거. 구독 로그의 instance state 변화 |
| lease-expiry | cyclone | fastdds | `-r` | `--write-period 100` 만 (`--num-iterations` 없음) | kill9 | 25 (Cyclone lease 10초 + 여유) | dispose 없이 사라진 참가자를 구독자가 lease 만료로 정리. 증거는 캡처의 마지막 SPDP 시각과 그 뒤 구독자의 동작 (ACKNACK 중단 등). 구독 로그에 liveliness 변화가 찍히는지는 미확인이므로 Step 3에서 `-h`와 소스로 확인한다 |
| large-sample | cyclone | fastdds | `-r` | 기본값 + `--additional-payload-size 100000`, `--num-iterations 30` | wait | 5 | DATA_FRAG, HEARTBEAT_FRAG, NACK_FRAG (두 벤더 사이). IP 단편화도 섞이므로 Wireshark에서 IP 재조립 설정을 켜고 본다 |
| xcdr2 | cyclone | fastdds | `-r -x 2` (`-x 1` 대체) | 기본값 | wait | 5 | 캡슐화 식별자 0x0007 (CDR2_LE) 등 XCDR2 페이로드 |

- lease 기본값: Cyclone `Discovery/LeaseDuration` 10초, Fast DDS 20초 (공식 문서로 확인). 구독자는 상대가 SPDP로 광고한 lease 값을 쓰므로, Cyclone 발행자를 죽이면 Fast DDS 구독자는 약 10초 뒤에 정리한다. 캡처의 PID_PARTICIPANT_LEASE_DURATION 값으로 재확인한다.
- 각 시나리오는 한 번에 참가자 둘만 띄운다. 실행 흐름 10번의 `pgrep`이 비어 있는지 확인하고 시작한다.
- 선택: dispose-on-exit의 `d`를 `u`로 바꾼 unregister-only 캡처, 발행 `-r` + 구독 `-b` 같은 QoS 불일치 캡처 (REQUESTED_INCOMPATIBLE_QOS 관찰)는 시간이 남으면 추가한다.


## 예상 문제와 대응
빌드 문제는 [vendor-build.md](../environment/vendor-build.md#빌드-문제와-대응), 네트워크와 캡처 문제는 [docker-network.md](../environment/docker-network.md#문제와-대응)를 본다. 시나리오 실행에 관한 것만 적는다.

| 증상 | 원인 후보 | 대응 |
|---|---|---|
| 로그 파일이 비어 있음 | 블록 버퍼링 | `stdbuf -oL`을 확인한다 |
| `pkill`이 프로세스를 못 찾음 | 15자 넘는 실행 파일 이름 | 짧은 이름으로 복사했는지 확인한다. `-x`는 이름 전체 일치다 |
| 스크립트에서 "input device is not a TTY" | `docker compose exec`에 `-T` 누락 | `-T`를 붙인다 |
| SPDP는 보이는데 매칭이 안 됨 | QoS 불일치, 데이터 표현 불일치 (`-x`) | 양쪽 옵션을 같게 맞춘다. 이것 자체도 좋은 캡처다 |

## 프로젝트 규약 (제안, Step 8에서 확정)
- 라이선스: `MIT OR Apache-2.0` 이중 라이선스 (2026-10-28 결정). 파일: `LICENSE-MIT`, `LICENSE-APACHE`
  - Rust 생태계 표준 관례 (Rust 자체, tokio, serde 등). 사용자가 둘 중 하나를 고른다.
  - Apache-2.0은 특허 허락 조항이 있고, MIT는 GPLv2 프로젝트와도 섞어 쓸 수 있다.
  - 참고 구현 중 Cyclone DDS는 EPL-2.0이다. 읽고 배우는 것은 괜찮지만 코드를 그대로 옮겨 오지 않는다.
- Rust: edition 2024. 시작 시점의 stable 버전을 `rust-toolchain.toml`에 고정하고 그 버전을 MSRV로 삼는다.
- 형식과 정적 검사: `rustfmt` 기본 설정, `cargo clippy -- -D warnings` 통과
- 테스트 배치: 단위 테스트는 모듈 안 `#[cfg(test)]`, pcap 기반 골든 테스트와 상호운용 테스트는 `tests/`
- 캡처 파일: `tests/captures/phase00/<시나리오>.pcap`과 `.pub.log`, `.sub.log`. README에 실행 명령, 벤더 버전, 빌드 소스 커밋 해시를 적는다. 파일 하나가 수 MB를 넘으면 (large-sample) 필요한 프레임만 `editcap`으로 잘라 저장한다.
  - 디렉터리 이름을 `captures`로 한 이유: 내용이 전부 캡처와 그 로그이고, 컨테이너 안 마운트 경로 `/captures`와 이름이 같다. 버린 대안 `fixtures`: 테스트의 준비 코드 (pytest fixture, rstest `#[fixture]`)와 이름이 겹쳐 헷갈린다.
- 코드 주석에 규격 근거를 적는다. 형식: `// RTPS 2.5 Sec. 9.4.5.3`
- 문서 / 커밋 / 주석은 ASCII와 한글만 쓴다 (전역 지침).
- CI: 원격 저장소를 연결하면 GitHub Actions로 fmt, clippy, test를 돌린다.
- 프로젝트 `CLAUDE.md`: 위 규약과 문서 갱신 규칙을 적는다.

## 참고 코드
- Cyclone DDS: RTPS 상태 머신, 디스커버리 로직 읽기 좋음
- RustDDS: Rust식 구조
- dust_dds: XTypes, 워크스페이스 구성

## 완료 기준
- Mac의 Docker Desktop (또는 어떤 Docker 호스트)에서 `docker compose up -d` 와 `phase00-capture.ts <이름>` 만으로 (Deno 2 필요) 누구나 같은 캡처를 재현할 수 있다.
- 시나리오 표의 캡처 8개와 로그가 저장소에 있고, README에 주목할 프레임이 적혀 있다.
- 캡처에서 SPDP / SEDP / DATA / HEARTBEAT / ACKNACK / STATUS_INFO(dispose) / DATA_FRAG를 Wireshark로 식별했고, 두 벤더의 차이를 메모했다.
- Cargo 워크스페이스가 빌드되고 clippy, fmt를 통과한다.
- 프로젝트 규약이 확정되어 이 문서와 `CLAUDE.md`에 반영되어 있다.

## 진행 기록
결정과 막힌 점만 한 줄씩 적고 근거 문서로 링크한다. 세부 경위는 커밋 메시지에 둔다.
- 2026-10-28: 벤더 DDS는 OMG 배포 실행 파일 대신 이미지 안에서 소스로 빌드하고, 이미지를 벤더별로 나눔 -> [vendor-build.md](../environment/vendor-build.md)
- 2026-10-28: 실행 호스트를 Mac (arm64) Docker Desktop으로 정함. x86_64 Linux 머신은 선택 -> [docker-network.md](../environment/docker-network.md#실행-호스트-mac-docker-desktop)
- 2026-10-28: Cyclone을 태그 11.0.1이 아닌 master `f2aaccb4`로 고정 (11.0.1로는 C판 shape_main 링크 실패) -> [vendor-build.md](../environment/vendor-build.md#cyclone을-master-커밋으로-고정하는-이유)
- 2026-10-28: Step 3 이미지 빌드 완료 (Mac에서 약 3분) -> [vendor-build.md "빌드 결과"](../environment/vendor-build.md#빌드-결과)
- 2026-10-28: 실행 환경 문서를 Phase 00에서 `docs/environment/`로 분리 (이후 단계에서도 계속 쓰므로). 검토 기록 절을 없애고 진행 기록을 한 줄 색인으로 줄임
- 2026-10-28: Step 4 완료. 이 환경의 bridge는 querier가 없어 IGMP 가입과 무관하게 멀티캐스트를 모든 포트로 흘려보냄 (전달 여부는 받는 쪽 커널의 가입이 결정) -> [docker-network.md](../environment/docker-network.md#멀티캐스트가-컨테이너-사이를-넘어가는-원리)
- 2026-10-28: 캡처 볼륨을 `tests/captures` 전체로 바꿈 (단계에 묶지 않음). 시나리오는 `/captures/phase00/`에 쓴다 -> [docker-network.md "compose 설계"](../environment/docker-network.md#compose-설계)
- 2026-10-28: 멀티캐스트 확인 도구를 socat 대신 Rust로 만듦 (`tools/mcast-check`, 이미지 `mdds/tools`, 벤더 컨테이너의 네트워크 네임스페이스를 빌려 실행). 이를 위해 Step 2 워크스페이스 뼈대를 함께 만듦 -> [docker-network.md "검증 명령"](../environment/docker-network.md#검증-명령)
- 2026-10-28: Docker Desktop (Mac arm64) VM 안 bridge가 컨테이너 사이 멀티캐스트를 전달함을 확인 (fastdds 가입, cyclone 송신 3건 모두 수신)
- 2026-10-28: Step 5 완료. 시나리오 스크립트 작성, rel 두 시나리오 캡처. shape_main `-c`를 발행 전용으로 바꿈 (구독자에게는 내용 필터가 됨) -> [시나리오 표](#시나리오-표), [캡처 README](../../tests/captures/phase00/README.md)
- 2026-10-29: 시나리오 스크립트를 POSIX sh에서 TypeScript + Deno로 바꿈 (가독성) -> [시나리오 실행 흐름](#시나리오-실행-흐름-phase00-capturets)
- 2026-10-29: Step 6 완료. 나머지 6개 시나리오 캡처 (large-sample은 2.9MB라 자르지 않음). 시나리오별 다이어그램은 Step 7 -> [캡처 README](../../tests/captures/phase00/README.md#파일)
- 2026-10-29: 캡처 메모는 관찰과 질문까지만 적고, 해석과 가설은 해당 단계의 "확인할 것"으로 옮김 (Phase 03, 05, 06, 07, 09) -> [Step 7](#step-7-캡처-분석-메모)
- 2026-10-29: Step 7 완료. 캡처 8개 모두 다이어그램과 질문을 적음 -> [캡처 README](../../tests/captures/phase00/README.md)
