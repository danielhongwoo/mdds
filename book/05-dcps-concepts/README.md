# 5. DCPS 개념 모델

[목차](../README.md) | 이전: [4 같은 규격, 다른 구현](../04-vendor-differences/README.md) | 다음: [6 CDR 직렬화](../06-cdr/README.md)

chapter3과 4에서 writer 정보와 reader 정보가 토픽, 타입, QoS를 나르고, 양쪽 로그에 `matched`가 출력되는 것을 보았다. 응용이 쓰는 API 수준의 DDS 규격을 [DCPS](../glossary.md#모델) (Data-Centric Publish-Subscribe)라고 부르고, 이 책에서 writer와 reader는 DCPS의 DataWriter와 DataReader를 줄인 말이다. 이 chapter는 그 반대쪽, 곧 무엇이 맞지 않으면 매칭되지 않는지를 본다. DCPS 규격 (DDS 1.4)이 정한 개념 모델에서 와이어에 실리는 것과 로컬에만 있는 것을 가르고, QoS 정책 가운데 매칭 여부를 정하는 규칙 ([RxO](../glossary.md#기본-개념))을 읽은 뒤, 그 규칙을 이 책의 첫 Rust 코드로 옮긴다.

끝나면 QoS 표를 보고 어느 정책이 어디에 적용되고 와이어에 실리며 매칭 여부를 정하는지 말할 수 있고, writer와 reader의 QoS를 받아 어긋난 정책을 돌려주는 함수가 테스트와 함께 있다.

## 1. 왜 필요한가

패킷은 정상인데 데이터가 오지 않는 경우가 있다. SPDP와 SEDP가 평소처럼 오가고 writer 정보와 reader 정보가 상대에게 전달됐는데, 사용자 DATA는 한 건도 없고 양쪽 로그에 `incompatible_qos ... 11 (RELIABILITY)`만 출력된다 (11은 규격이 RELIABILITY 정책에 붙인 번호다. 3.4절). 파티션이 다른 경우에는 로그에 아무것도 출력되지 않는다. Lab 5-2에서 이 상황을 직접 만든다. chapter3의 방법으로 캡처를 열어 보아도 어디가 잘못인지 보이지 않는다. 디스커버리는 성공했고, 매칭이 되지 않았을 뿐이다.

매칭 규칙은 RTPS가 아니라 DCPS 규격에 있다. chapter0은 QoS를 네 개의 예로만 소개했다. chapter11에서 SEDP 매칭을 구현하고 chapter14에서 API를 만들려면 모델 전체가 필요하다. 이 chapter에서 세 가지를 정리한다.
- **와이어에 실리는 것과 로컬에만 있는 것.** DCPS의 엔티티 ([용어 정리](../glossary.md#모델)) 가운데 참가자와 writer / reader만 상대에게 알려진다. Publisher와 Subscriber는 와이어에 없다.
- **어느 QoS가 매칭 여부를 정하는가.** QoS 정책은 24종이다 (DDS 1.4의 22종과 DDS-XTypes 1.3의 2종. 3.4절). 그 가운데 호환 조건이 있는 것은 9개 (RxO 8개와 DATA_REPRESENTATION)이고, [PARTITION](../glossary.md#기본-개념) (writer와 reader가 같은 이름을 가져야 통신하는 문자열 목록. 3.3절)은 그와 다른 방식으로 매칭 여부를 정한다. 나머지는 QoS로서는 매칭과 무관하다.
- **어디까지 구현할 것인가.** DDS 1.4는 [적합성 프로파일](../glossary.md#기본-개념) 다섯 개로 범위를 나눈다. 이 책이 어디까지 가는지 정한다.

이 책의 첫 코드를 여기서 쓰는 이유는 RxO 규칙이 소켓도 시계도 없는 순수 함수이기 때문이다. 규격의 표를 Rust 타입과 테스트로 옮기는 일이고, 그 결과를 캡처에서 읽은 QoS 쌍에 넣어 벤더의 로그와 같은 판정이 나오는지 바로 확인할 수 있다. 이 chapter의 `check_rxo`와 `partitions_match`는 chapter11의 SEDP 매칭과 chapter14의 상태 (status) 보고가 그대로 쓴다.

### 1.1 이 책의 코드가 사는 곳

chapter1에서 clone한 프로젝트 디렉터리의 `crates/`에 crate 세 개가 있다. 역할만 적는다.

| crate | 하는 일 | 쓰기 시작하는 chapter |
|---|---|---|
| `mdds-cdr` | CDR / XCDR 직렬화 | 6 |
| `mdds-rtps` | RTPS 메시지, 디스커버리, 상태 머신. 소켓, 시계, 스레드를 쓰지 않는 I/O-free core ([용어 정리](../glossary.md#설계)) | 5 (이 chapter의 `qos` 모듈) |
| `mdds` | 사용자 API와 UDP transport. `mdds-rtps`를 쓴다 | 8 |

QoS 타입과 매칭 규칙은 `mdds-rtps`에 둔다. 디스커버리의 매칭 (chapter11)이 이 crate에 있고 `mdds`가 `mdds-rtps`에 의존하므로, QoS 타입은 아래층에 있어야 한다.

독자는 `crates/` (저자의 코드가 마지막 chapter까지 들어 있다)를 고치지 않고, 자기 작업 디렉터리 `my-code/`에서 쓴다. 여기서 만드는 것은 이 chapter의 완성 코드에서 `qos` 모듈만 뺀 연습용 사본이다. 완성 코드는 chapter마다 그 chapter의 [`code/`](code/)에 있다. 프로젝트 디렉터리에서 다음 두 명령을 실행한다.
```bash
cp -R book/05-dcps-concepts/code my-code && rm my-code/crates/mdds-rtps/src/qos.rs
sed -i.bak '/^pub mod qos;$/d' my-code/crates/mdds-rtps/src/lib.rs && rm my-code/crates/mdds-rtps/src/lib.rs.bak
```
두 번째 명령은 `my-code/crates/mdds-rtps/src/lib.rs`에서 `pub mod qos;` 줄을 지운다 (편집기로 지워도 된다). `my-code/`는 git이 무시하는 디렉터리이고, 이후 chapter에서도 여기에 이어서 쓴다 (앞 chapter를 건너뛰었으면 그 chapter의 `code/`를 새로 복사한다).

코드를 쓰고 검사하는 명령은 `my-code/`에서 실행한다 (프로젝트 디렉터리에서 `cd my-code`).
```bash
cargo test -p mdds-rtps
```
처음 실행할 때 rustup이 `rust-toolchain.toml`에 고정된 Rust 1.99.1을 내려받는다 (네트워크가 필요하다). 자기 코드와 완성본을 비교할 때는 프로젝트 디렉터리에서 두 디렉터리를 diff한다.
```bash
diff -r my-code/crates book/05-dcps-concepts/code/crates
```

## 2. 캡처에서 보기

### 2.1 Lab 5-1: 와이어에 실리는 QoS

#### 2.1.1 목표

chapter3, 4의 캡처에서 writer 정보 (`DATA(w)`)와 reader 정보 (`DATA(r)`)를 열어, QoS가 어떤 PID로 실리고 무엇이 생략되는지 본다.

#### 2.1.2 준비

[`reliable-cyclone-pub.pcap`](../../captures/cyclone-fastdds/README.md#reliable-cyclone-pub), [`best-effort-cyclone-pub.pcap`](../../captures/cyclone-fastdds/README.md#best-effort-cyclone-pub), [`reliable-fastdds-pub.pcap`](../../captures/cyclone-fastdds/README.md#reliable-fastdds-pub)을 Wireshark 4.6.9로 연다. 필터 `rtps.sm.wrEntityId == 0x000003c2 || rtps.sm.wrEntityId == 0x000004c2`로 SEDP의 DATA만 남긴다 (0x000003c2는 writer 정보를 보내는 내장 writer, 0x000004c2는 reader 정보를 보내는 내장 writer. chapter3 3.1).

#### 2.1.3 절차

표의 프레임을 상세 창에서 열고 `serializedData` 아래의 `parameterId` 목록에서 QoS에 해당하는 PID (`PID_RELIABILITY`, `PID_DURABILITY`, `PID_PARTITION`, `PID_DATA_REPRESENTATION`)를 찾아 값을 적는다. 값은 PID 아래 한 줄로 나온다 (예: `PID_RELIABILITY` > `Kind: RELIABLE_RELIABILITY_QOS (0x00000002)`. 아래 표에서는 `RELIABLE`로 줄여 적는다). `PID_TOPIC_NAME`과 `PID_TYPE_NAME`은 QoS가 아니지만 같은 목록에 있으니 함께 본다.

| 캡처 | 프레임 | 누가 보낸 무엇 | 양쪽 옵션 |
|---|---|---|---|
| reliable-cyclone-pub | 22 | Cyclone의 writer 정보 | `-r` |
| reliable-cyclone-pub | 47 | Fast DDS의 reader 정보 | `-r` |
| best-effort-cyclone-pub | 27 | Cyclone의 writer 정보 | `-b` |
| best-effort-cyclone-pub | 32 | Fast DDS의 reader 정보 | `-b` |
| reliable-fastdds-pub | 17 | Fast DDS의 writer 정보 | `-r` |
| reliable-fastdds-pub | 23 | Cyclone의 reader 정보 | `-r` |

Cyclone이 보낸 정보 (22, 27, 23)는 chapter3에서 본 대로 `Malformed`로 끊긴다. `PID_RELIABILITY`까지는 보이고, 그 뒤 `PID_RESOURCE_LIMIT`, `PID_TYPE_INFORMATION`에서 해석이 멈춘다. 그 뒤에 어떤 PID가 있는지는 화면으로 알 수 없다.

#### 2.1.4 기대 결과

| 캡처 | 프레임 | 보낸 쪽 | 양쪽 옵션 | 실린 QoS PID와 값 |
|---|---|---|---|---|
| reliable-cyclone-pub | 22 | Cyclone writer | `-r` | RELIABILITY = RELIABLE |
| reliable-cyclone-pub | 47 | Fast DDS reader | `-r` | DURABILITY = VOLATILE, RELIABILITY = RELIABLE, DATA_REPRESENTATION = [XCDR] |
| best-effort-cyclone-pub | 27 | Cyclone writer | `-b` | RELIABILITY = BEST_EFFORT |
| best-effort-cyclone-pub | 32 | Fast DDS reader | `-b` | DURABILITY = VOLATILE, DATA_REPRESENTATION = [XCDR]. RELIABILITY 없음 |
| reliable-fastdds-pub | 17 | Fast DDS writer | `-r` | DURABILITY = VOLATILE, DATA_REPRESENTATION = [XCDR]. RELIABILITY 없음 |
| reliable-fastdds-pub | 23 | Cyclone reader | `-r` | RELIABILITY = RELIABLE |

- QoS는 writer 정보와 reader 정보에 PID 하나씩으로 실린다. 어느 PID가 실릴 수 있는지는 RTPS 2.5 표 9.19가 정한다 (3.4절).
- 벤더마다 기본값인 정책을 생략하는 규칙이 다르다. Fast DDS는 `-r`인 writer (17)와 `-b`인 reader (32)에 RELIABILITY를 싣지 않지만 DURABILITY (VOLATILE)는 늘 싣는다. Cyclone은 RELIABILITY를 `-r`과 `-b` 모두에서 싣는다 (22, 27, 23).
- 없는 PID는 그 엔티티의 기본값으로 읽는다 (RTPS 2.5 Sec. 8.5.4.4. 기본값은 3.2절). writer 정보에 없으면 writer의 기본값, reader 정보에 없으면 reader의 기본값이다. 그래서 RELIABILITY를 싣지 않은 Fast DDS writer (17)를 상대는 RELIABLE로 본다.

#### 2.1.5 확인 질문

- 32의 reader 정보에는 RELIABILITY가 없다. 이 reader를 발견한 Cyclone writer는 이 reader가 무엇을 요구한다고 보는가?

#### 2.1.6 이 chapter에서 답하지 않는 것

- PID의 바이트 배치 (parameterId, length, 값)와 ParameterList 해석 -> chapter7, 9
- PID_TYPE_INFORMATION의 내용과 Wireshark가 `Malformed`로 끊는 이유 -> chapter16

### 2.2 Lab 5-2: QoS가 맞지 않으면

#### 2.2.1 목표

writer와 reader의 QoS가 호환되지 않을 때, 그리고 파티션이 다를 때 와이어와 로그에 무엇이 남는지 본다. 참고 캡처 둘을 읽고, 하나는 직접 한다.

#### 2.2.2 준비

[`qos-mismatch.pcap`](../../captures/cyclone-fastdds/README.md#qos-mismatch)과 [`partition-mismatch.pcap`](../../captures/cyclone-fastdds/README.md#partition-mismatch), 같은 디렉터리의 `.pub.log`, `.sub.log`를 쓴다. 두 캡처는 [chapter3의 Lab 3-1](../03-first-communication/README.md#21-lab-3-1-첫-dds-통신)과 같은 명령에서 옵션만 바꾼 것이다. 아래 표의 구독자와 발행자는 Lab 3-1과 같이 shape_main 프로세스를 가리킨다.

| 캡처 | 구독자 (Fast DDS) | 발행자 (Cyclone) |
|---|---|---|
| qos-mismatch | `-r` (RELIABLE을 요구) | `-b` (BEST_EFFORT만 제공) |
| partition-mismatch | `-r -p A` (파티션 `A`) | `-r` (파티션 옵션 없음) |

직접 하는 부분은 chapter3 2.1.2 점검과 터미널 A, B, C를 그대로 쓴다. 세 터미널 모두 `docker/`에서 연다.

#### 2.2.3 절차

먼저 `qos-mismatch`를 Wireshark로 읽는다.

| 프레임 | 시각 (초) | 표지와 보이는 것 |
|---|---|---|
| 3-18 | 0-3.08 | SPDP 교환 (13, 14는 IGMP). chapter3과 같다 |
| 19-27 | 3.08-3.10 | SEDP 내장 엔드포인트와 참가자 메시지 writer의 HEARTBEAT와 ACKNACK |
| 28 | 3.10 | Cyclone의 writer 정보 (목록에는 `DATA[Malformed Packet]`). `PID_RELIABILITY` > `BEST_EFFORT_RELIABILITY_QOS` |
| 31 | 3.11 | writer 정보에 대한 Fast DDS의 ACKNACK (`writerEntityId` 0x000003c2). 상대가 받았다 |
| 35 | 3.19 | Fast DDS의 reader 정보 `DATA(r)`. `PID_RELIABILITY` > `RELIABLE_RELIABILITY_QOS` |
| 40-41 | 4.09 | reader 정보에 대한 HEARTBEAT와 ACKNACK. 상대가 받았다 |
| 42-56 | 5.10-12.57 | SPDP와 참가자 메시지만. 사용자 DATA가 없다 |
| 57 | 13.69 | `DATA(w[UD])`. 발행자가 반복 횟수를 채우고 스스로 끝났다 |
| 66-67 | 19.03 | `DATA(p[UD])`. 구독자 종료 |

필터 `rtps.sm.wrEntityId == 0x00000202` (Cyclone의 Square writer. chapter4의 Lab 4-1)를 건다. 이 필터는 그 writer의 DATA와 HEARTBEAT를 모두 잡는다. 정상 통신에서는 같은 필터에 사용자 writer의 프레임이 남는다 (`reliable-cyclone-pub.pcap`에서 184 프레임). 여기서는 0이다.

로그에서 `incompatible`을 찾는다. `docker/`에서 실행한다.
```bash
grep incompatible ../captures/cyclone-fastdds/qos-mismatch.*.log
```
```
qos-mismatch.pub.log:on_offered_incompatible_qos() topic: 'Square'  type: 'ShapeType' : 11 (RELIABILITY)
qos-mismatch.sub.log:on_requested_incompatible_qos() topic: 'Square'  type: 'ShapeType' : 11 (RELIABILITY)
```

다음은 `partition-mismatch`다. 구조는 같다.

| 프레임 | 시각 (초) | 표지와 보이는 것 |
|---|---|---|
| 23 | 3.08 | Cyclone의 writer 정보 (목록에는 `DATA[Malformed Packet]`). `PID_PARTITION`이 없다 (파티션 옵션이 없는 기본값) |
| 28 | 3.09 | writer 정보에 대한 Fast DDS의 ACKNACK (`writerEntityId` 0x000003c2) |
| 33 | 3.18 | Fast DDS의 reader 정보 `DATA(r)`. `PID_PARTITION` > `name[0]: A` |
| 40-41 | 4.08 | reader 정보에 대한 HEARTBEAT와 ACKNACK |
| 42-56 | 5.09-12.56 | SPDP와 참가자 메시지만. 사용자 DATA가 없다 |

같은 필터 `rtps.sm.wrEntityId == 0x00000202`를 걸면 여기서도 0이다. 두 로그 모두 `Create topic`과 `Create writer` / `Create reader`, 그리고 종료 줄뿐이다. `matched`도 `incompatible`도 없다.

마지막으로 `qos-mismatch`를 직접 만든다. Lab 3-1의 절차에서 명령만 다르고, 세 터미널의 명령은 모두 `docker/`에서 실행한다. 발행 옵션 `--write-period 100 --num-iterations 100`은 Lab 3-1과 같다 (100 ms마다 100번 쓰고 끝난다).

1. 터미널 A에서 캡처를 시작하고 `listening on eth0` 뒤 1초 기다린다.
   ```bash
   docker compose exec fastdds tcpdump -i eth0 -U -Z root -w /captures/cyclone-fastdds/my-qos-mismatch.pcap 'udp or igmp'
   ```
2. 터미널 B에서 구독자를 RELIABLE로 시작하고 3초 기다린다.
   ```bash
   docker compose exec fastdds fastdds_shape -S -d 0 -t Square -x 1 -r
   ```
3. 터미널 C에서 발행자를 BEST_EFFORT로 시작한다.
   ```bash
   docker compose exec cyclone cyclone_shape -P -d 0 -t Square -x 1 -b -c BLUE --write-period 100 --num-iterations 100
   ```
4. 터미널 B와 C에 `incompatible_qos` 줄이 출력된다. 샘플 줄은 없다. 발행자는 약 10초 뒤 스스로 끝난다. 5초 더 기다린다.
5. 터미널 B에서 Ctrl+C (`Done.`), 2초 뒤 터미널 A에서 Ctrl+C. chapter3 2.1.2 점검 명령으로 남은 프로세스가 없는지 확인한다.

만든 `my-qos-mismatch.pcap`을 Wireshark로 열어 참고 캡처와 같은지 본다. writer 정보와 reader 정보가 있고, 필터 `rtps.sm.wrEntityId == 0x00000202`에 남는 프레임 (사용자 DATA)이 없고, 터미널 B와 C의 출력에 `incompatible`이 있으면 된다.

파티션 불일치를 직접 보려면 같은 절차에서 세 곳을 바꾼다. 1단계의 파일 이름을 `my-partition-mismatch.pcap`으로, 2단계의 구독 명령 끝에 `-p A`를 붙이고, 3단계의 `-b`를 `-r`로 바꾼다. 이때는 4단계에서 두 터미널에 아무 줄도 더 출력되지 않는다. 캡처에서는 reader 정보에 `PID_PARTITION` > `name[0]: A`가 있고 사용자 DATA가 없으면 된다.

#### 2.2.4 기대 결과

- 두 경우 모두 디스커버리는 성공한다. writer 정보와 reader 정보가 상대에게 전달되고 ACKNACK으로 확인된다 (`qos-mismatch`는 28과 31, 35와 40-41. `partition-mismatch`는 23과 28, 33과 40-41). 그런데 사용자 DATA가 없고, 사용자 writer의 HEARTBEAT도 없다. 매칭되지 않은 reader는 writer가 보낼 상대가 아니다.
- QoS 불일치는 양쪽 로그에 남는다. 발행자 쪽은 `on_offered_incompatible_qos`, 구독자 쪽은 `on_requested_incompatible_qos`이고, 둘 다 `11 (RELIABILITY)`를 출력한다. 11은 DDS 1.4가 RELIABILITY 정책에 붙인 번호다 (3.4절). shape_main은 DDS의 상태 (status) 변화를 알리는 Listener 콜백을 그대로 출력한다.
- 파티션 불일치는 로그에 남지 않는다. 규격이 파티션 불일치를 "incompatible QoS"로 보지 않고 Listener도 호출하지 않는다고 정했기 때문이다 (3.3절). 로그에 아무것도 없는데 데이터가 오지 않으면 파티션부터 의심한다.

#### 2.2.5 실패하면

- 터미널 B에 `matched writers 1`과 샘플 줄이 출력되면 이전 실습의 발행자가 남아 있거나 옵션이 틀린 것이다. 터미널 B, C를 Ctrl+C로 끝내고 chapter3 2.1.2 점검부터 다시 한다.
- `incompatible_qos` 줄이 한쪽에만 출력되면 한쪽이 상대의 정보를 아직 받지 못한 것이다. 몇 초 더 기다린다.
- 파티션 불일치 실행에서 `matched`가 출력되면 `-p A`가 빠진 것이다.
- 캡처가 비어 있거나 7400 문제가 있으면 [chapter3 2.1.5](../03-first-communication/README.md#215-실패하면)을 본다.

#### 2.2.6 이 chapter에서 답하지 않는 것

- 매칭되지 않은 reader에게 writer가 HEARTBEAT를 보내지 않는 이유, writer가 HEARTBEAT를 보내는 상대의 목록 -> chapter10, 11
- `incompatible_qos` 상태 (IncompatibleQosStatus)를 API에서 어떻게 읽는지, Listener와 WaitSet -> chapter14
- 파티션과 QoS가 동시에 어긋났을 때 어느 쪽을 먼저 보고하는가 (규격이 정하지 않았고 두 벤더가 다르다. 6절) -> chapter11

## 3. 규격 읽기

### 3.1 엔티티와 와이어

DCPS의 엔티티는 계층을 이룬다 (DDS 1.4 Sec. 2.2.1.2.2, [용어 정리](../glossary.md#모델)). DomainParticipant가 Publisher, Subscriber, Topic을 만들고, Publisher가 DataWriter를, Subscriber가 DataReader를 만든다. DataWriter와 DataReader는 Topic 하나에 묶인다. 모든 엔티티는 QoS, Listener (상태 변화를 받는 콜백), StatusCondition (응용이 상태 (status) 변화를 기다릴 때 쓰는 객체. condition의 한 종류다)을 갖는 같은 패턴이다. Lab 5-2의 로그 줄은 shape_main이 DomainParticipant에 단 Listener가 출력한 것이다.

와이어에 알려지는 것은 참가자 (SPDP)와 DataWriter / DataReader (SEDP)다 (chapter3의 내장 엔드포인트). 규격은 writer 정보의 내용을 PublicationBuiltinTopicData, reader 정보의 내용을 SubscriptionBuiltinTopicData라고 부른다 (Publication이 writer 쪽, Subscription이 reader 쪽이다). Publisher와 Subscriber는 알려지지 않는다. 그 대신 그 QoS 가운데 매칭에 필요한 것 (PARTITION, PRESENTATION, GROUP_DATA)은 소속 writer와 reader의 정보에 PID로 실린다 (RTPS 2.5 표 9.19). Lab 5-2에서 reader의 `-p A`가 `PID_PARTITION`으로 실린 것이 그 예다.

Topic의 QoS는 writer와 reader에 자동으로 적용되지 않는다. 응용이 Topic의 QoS를 복사해 쓰는 편의 연산이 따로 있다 (DDS 1.4 Sec. 2.2.2.4.1.17). 3.4절 표의 "적용 엔티티"에 Topic이 있는 정책은 Topic도 그 정책을 가질 수 있다는 뜻이다. 샘플에 붙는 상태 ([SampleInfo](../glossary.md#기본-개념))는 chapter13에서 다룬다.

### 3.2 requested, offered, RxO

QoS는 reader가 요구 (requested)하고 writer가 제공 (offered)한다. DDS 1.4 Sec. 2.2.3의 표는 정책마다 RxO 열을 두어, 양쪽 값이 호환되어야 하는 정책 (Yes), 양쪽에 있지만 서로 무관한 정책 (No), 한쪽에만 있는 정책 (N/A)을 가른다. Yes인 8개는 각 정책의 절에 호환 조건이 있고 (OWNERSHIP만 "정확히 같아야 한다"는 문장이다), 하나라도 어긋나면 매칭되지 않는다. 어긋난 결과는 writer 쪽의 OFFERED_INCOMPATIBLE_QOS, reader 쪽의 REQUESTED_INCOMPATIBLE_QOS 상태 (status)이고, Lab 5-2의 로그 줄이 그것이다. 여기에 DDS-XTypes 1.3이 더한 DATA_REPRESENTATION 하나가 같은 방식 (request-offer)으로 매칭 여부를 정한다. 이 책은 이 9개를 "매칭 검사 9개 (RxO 8개와 DATA_REPRESENTATION)"라고 부른다.

| 정책 (번호) | 뜻 | 호환 조건 | 순서 | 출처 |
|---|---|---|---|---|
| DURABILITY (2) | 늦게 온 reader에게 이전 샘플을 줄지 (kind) | offered kind >= requested kind | VOLATILE < TRANSIENT_LOCAL < TRANSIENT < PERSISTENT | DDS 1.4 Sec. 2.2.3.4 |
| PRESENTATION (3) | 여러 인스턴스 / 토픽의 변경을 묶어 (coherent) 또는 순서대로 (ordered) 보일지와 그 범위 (access_scope) | offered access_scope >= requested access_scope. coherent_access는 requested가 FALSE이거나 둘 다 TRUE. ordered_access도 같다 | INSTANCE < TOPIC < GROUP | Sec. 2.2.3.6 |
| DEADLINE (4) | 인스턴스마다 새 샘플이 이 주기 (period) 안에 와야 한다 | offered period <= requested period | 시간 | Sec. 2.2.3.7 |
| LATENCY_BUDGET (5) | 쓴 뒤 reader에게 닿기까지 허용하는 지연 (duration) | offered duration <= requested duration | 시간 | Sec. 2.2.3.8 |
| OWNERSHIP (6) | 한 인스턴스를 여러 writer가 쓸 수 있는지 (SHARED) 한 writer만 쓰는지 (EXCLUSIVE) | 같아야 한다 | 없음 | Sec. 2.2.3.9 |
| LIVELINESS (8) | writer가 살아 있음을 알리는 방식 (kind)과 주기 (lease_duration) | offered kind >= requested kind, 그리고 offered lease_duration <= requested lease_duration | AUTOMATIC < MANUAL_BY_PARTICIPANT < MANUAL_BY_TOPIC | Sec. 2.2.3.11 |
| RELIABILITY (11) | 빠진 샘플을 다시 받을지 (kind) | offered kind >= requested kind | BEST_EFFORT < RELIABLE | Sec. 2.2.3.14 |
| DESTINATION_ORDER (12) | 같은 인스턴스의 샘플을 받은 순서로 볼지 쓴 시각 순서로 볼지 | offered kind >= requested kind | BY_RECEPTION_TIMESTAMP < BY_SOURCE_TIMESTAMP | Sec. 2.2.3.17 |
| DATA_REPRESENTATION (23) | 직렬화 표현 (XCDR, XCDR2)의 목록 | writer가 제공하는 표현 (목록의 첫 원소)이 reader의 목록에 있어야 한다. 빈 목록은 [XCDR] | 없음 | XTypes 1.3 Sec. 7.6.3.1.1 |

순서는 규격이 정책마다 "considered ordered such that ..."으로 명시했다. 읽는 법은 한 가지다. writer는 reader가 요구한 것 이상을 제공해야 한다. 더 강한 보장 (RELIABLE, 긴 보관, 짧은 deadline)을 제공하는 것은 괜찮고, 충족하지 못하면 불일치다. 번호는 DDS 1.4의 IDL PSM (Platform Specific Model. Sec. 2.2의 모델을 IDL 인터페이스로 옮긴 부분, Sec. 2.3.3)이 정한 QosPolicyId_t 값이고 Lab 5-2 로그의 `11`이 그것이다.

기본값끼리는 호환된다. 그런데 RELIABILITY의 기본값은 엔티티마다 다르다. DataWriter는 RELIABLE, DataReader와 Topic은 BEST_EFFORT다 (Sec. 2.2.3.14). 아무 옵션 없이 만든 writer와 reader가 매칭되는 것은 이 때문이고, Lab 5-1에서 Fast DDS가 `-r`인 writer와 `-b`인 reader에 RELIABILITY를 싣지 않은 것도 이 기본값이 다르기 때문이다. 다만 shape_main은 옵션이 없어도 writer와 reader를 모두 RELIABLE로 만든다. 그래서 이 책의 lab은 늘 `-r` 또는 `-b`를 명시한다.

### 3.3 PARTITION

PARTITION은 Publisher / Subscriber의 QoS로, 문자열 목록이다. writer와 reader는 토픽이 같아도 공통 파티션이 있어야 통신한다 (DDS 1.4 Sec. 2.2.3.13). 규칙은 RxO와 다르다.
- 표의 RxO 열은 "No"다. 호환 조건이 아니라 별도의 매칭 조건이다. 그래서 불일치는 "incompatible QoS"가 아니고, Listener도 호출되지 않으며 condition (3.1절의 StatusCondition)도 바뀌지 않는다. Lab 5-2의 두 번째 캡처에서 로그에 아무것도 없었던 이유다.
- 빈 목록은 빈 문자열 `""` 하나와 같다 (기본값). 그래서 파티션을 주지 않은 writer (`[""]`)와 `-p A`인 reader (`["A"]`)는 일치하지 않는다.
- 이름은 POSIX fnmatch (셸이 `*.txt` 같은 파일 이름 패턴을 맞출 때 쓰는 규칙을 정한 POSIX 함수)의 와일드카드를 가질 수 있다. `*`는 0개 이상의 문자, `?`는 정확히 한 문자와 맞고, `[...]`는 괄호 안의 문자 하나와 맞는다. 어느 쪽의 이름이든 패턴이 될 수 있다 (`"A*"`와 `"AB"`는 일치). 다만 두 이름이 모두 와일드카드를 포함하면 그 쌍은 일치하지 않는다 (`"A*"`와 `"A*"`는 불일치). 비교는 목록 단위가 아니라 이름 쌍 단위다. 한 쌍이라도 일치하면 된다.
- 바꿀 수 있는 정책이다. 바꾸면 매칭이 새로 생기거나 끊긴다.

### 3.4 QoS 정책 지도

DDS 1.4 Sec. 2.2.3의 표 22행에 XTypes 1.3의 2행 (Sec. 7.6.3)을 더한 것이다. "적용 엔티티"는 그 정책을 가질 수 있는 엔티티 (표의 Concerns), "SEDP"는 writer 정보 (w) / reader 정보 (r)에 PID로 실리는지 (RTPS 2.5 표 9.19. 23, 24는 XTypes 1.3 Sec. 7.6.3)다. 모든 정책을 알 필요는 없다. 매칭 여부를 정하는 것은 RxO가 Yes인 8개, request-offer인 DATA_REPRESENTATION, 그리고 PARTITION뿐이고, 표에서 그 정책 이름을 굵게 적었다.

| 번호 | 정책 | 적용 엔티티 | RxO | 변경 가능 | SEDP |
|---|---|---|---|---|---|
| 1 | USER_DATA | DomainParticipant, DataReader, DataWriter | No | Yes | w, r (참가자 정보에도) |
| 2 | **DURABILITY** | Topic, DataReader, DataWriter | Yes | No | w, r |
| 3 | **PRESENTATION** | Publisher, Subscriber | Yes | No | w, r |
| 4 | **DEADLINE** | Topic, DataReader, DataWriter | Yes | Yes | w, r |
| 5 | **LATENCY_BUDGET** | Topic, DataReader, DataWriter | Yes | Yes | w, r |
| 6 | **OWNERSHIP** | Topic, DataReader, DataWriter | Yes | No | w, r |
| 7 | OWNERSHIP_STRENGTH | DataWriter | N/A | Yes | w |
| 8 | **LIVELINESS** | Topic, DataReader, DataWriter | Yes | No | w, r |
| 9 | TIME_BASED_FILTER | DataReader | N/A | Yes | r |
| 10 | **PARTITION** | Publisher, Subscriber | No (별도 매칭 조건) | Yes | w, r |
| 11 | **RELIABILITY** | Topic, DataReader, DataWriter | Yes | No | w, r |
| 12 | **DESTINATION_ORDER** | Topic, DataReader, DataWriter | Yes | No | w, r |
| 13 | HISTORY | Topic, DataReader, DataWriter | No | No | 없음 (토픽 정보에만) |
| 14 | RESOURCE_LIMITS | Topic, DataReader, DataWriter | No | No | 없음 (토픽 정보에만) |
| 15 | ENTITY_FACTORY | DomainParticipantFactory, DomainParticipant, Publisher, Subscriber | No | Yes | 없음 |
| 16 | WRITER_DATA_LIFECYCLE | DataWriter | N/A | Yes | 없음 |
| 17 | READER_DATA_LIFECYCLE | DataReader | N/A | Yes | 없음 |
| 18 | TOPIC_DATA | Topic | No | Yes | w, r |
| 19 | GROUP_DATA | Publisher, Subscriber | No | Yes | w, r |
| 20 | TRANSPORT_PRIORITY | Topic, DataWriter | N/A | Yes | 없음 |
| 21 | LIFESPAN | Topic, DataWriter | N/A | Yes | w |
| 22 | DURABILITY_SERVICE | Topic, DataWriter | No | No | w |
| 23 | **DATA_REPRESENTATION** (XTypes) | Topic, DataReader, DataWriter | request-offer | No | w, r |
| 24 | TYPE_CONSISTENCY_ENFORCEMENT (XTypes) | DataReader | N/A | No | r |

읽는 법:
- QoS로 매칭 여부를 정하는 것은 매칭 검사 9개와 PARTITION이다. TYPE_CONSISTENCY_ENFORCEMENT (24)는 QoS 값의 호환이 아니라 타입 검사의 방식을 정하는 것으로, 타입이 맞지 않으면 역시 매칭되지 않는다 (chapter16). 나머지는 값이 무엇이든 매칭과 무관하다.
- HISTORY와 RESOURCE_LIMITS는 규격의 표에서는 writer / reader 정보에 실리지 않는다. 상대가 샘플을 몇 개 보관하는지는 매칭과 무관하다. HISTORY는 샘플을 몇 개 보관할지 정하는 정책이고, 기본값은 KEEP_LAST 1 (인스턴스마다 가장 최근 샘플 1개만 보관)이다. chapter3에서 구독 로그의 샘플 수가 캡처보다 적었던 것은 reader의 이 기본값 때문이다 (chapter13). Cyclone이 writer / reader 정보에 `PID_RESOURCE_LIMIT`를 싣는 것 (Lab 5-1)은 규격의 표와 다른 벤더의 선택이다.

### 3.5 이 책이 구현하는 범위

DDS 1.4 Annex A는 적합성 프로파일 다섯 개 (Minimum, Content-subscription, Persistence, Ownership, Group access. [용어 정리](../glossary.md#기본-개념))를 둔다. RELIABILITY의 두 kind, DURABILITY의 TRANSIENT_LOCAL, PARTITION은 모두 Minimum 안에 있다.

mdds는 Minimum 가운데 두 벤더와의 상호운용 시험 (chapter11, 14)과 chapter0의 예 (failover)에 필요한 것, 그리고 Ownership 프로파일 (EXCLUSIVE와 OWNERSHIP_STRENGTH는 chapter14, HISTORY depth > 1은 chapter13)을 구현한다. 매칭 검사 9개는 이 chapter에서 모두 구현하지만, 정책의 동작까지 구현하는 것은 RELIABILITY, DURABILITY (VOLATILE, TRANSIENT_LOCAL), HISTORY, PARTITION, OWNERSHIP, LIVELINESS, DEADLINE이다. Content-subscription, Persistence, Group access는 하지 않는다. 어느 chapter에서 무엇을 하는지는 [목차](../README.md)에 있다.

## 4. 구현

### 4.1 Lab 5-3: QoS 타입과 RxO 검사

#### 4.1.1 목표

3.2절의 표와 3.3절의 규칙을 `my-code/crates/mdds-rtps/src/qos.rs`에 옮기고 단위 테스트로 확인한다. 완성본은 [`code/crates/mdds-rtps/src/qos.rs`](code/crates/mdds-rtps/src/qos.rs)에 있다. 진행이 어려우면 완성본과 비교한다.

#### 4.1.2 준비

1.1절대로 `my-code/`를 만들고 `qos.rs`와 `pub mod qos;`를 지운 상태에서 시작한다. `my-code/`에서 `cargo test -p mdds-rtps`가 테스트 0개로 통과하면 준비된 것이다.

#### 4.1.3 설계

이 모듈의 타입은 DDS API의 DataWriterQos / DataReaderQos가 아니라, SEDP로 상대에게 알려지는 QoS 부분이다 (3.1절의 PublicationBuiltinTopicData / SubscriptionBuiltinTopicData). 매칭이 보는 것이 이것이고, chapter11의 디스커버리 데이터가 이 타입을 필드로 갖는다. 이름도 그에 맞춰 `PublicationQos` (offered), `SubscriptionQos` (requested)다. 이 chapter에서는 매칭 검사 9개와 PARTITION만 필드로 둔다.

- kind는 enum이고, 선언 순서가 규격의 순서다. `Ord`를 파생하면 `offered >= requested`를 그대로 쓸 수 있다. 판별값은 와이어 값이 아니다 (RTPS는 BEST_EFFORT를 1, RELIABLE을 2로 보낸다. 변환은 chapter11). OWNERSHIP은 순서가 없으므로 `Ord`를 파생하지 않는다.
- 데이터 표현 식별자는 규격이 정한 2바이트 정수 값 (XCDR = 0, XML = 1, XCDR2 = 2)이라, 모르는 값이 와도 담을 수 있게 enum이 아니라 값을 감싼 struct로 둔다.
- 시간 길이 `Duration`은 유한 (`core::time::Duration`)과 무한대의 enum이다. 무한대를 뒤에 선언하면 가장 큰 값이 되어 DEADLINE과 lease 비교가 그대로 맞는다.
- `Default`는 PID가 없을 때의 값이다. writer와 reader의 기본값이 다른 정책 (RELIABILITY)이 있어 타입을 둘로 나눈다. 두 타입의 필드는 같다.
- `QosPolicyId`는 1-22에 XTypes의 23, 24를 더한 enum (`repr(u32)`)이다.
- `check_rxo`는 어긋난 정책을 번호 오름차순의 `Vec`으로 돌려준다. 비어 있으면 호환이다. `partitions_match`는 따로 둔다. 파티션 불일치는 incompatible QoS가 아니기 때문이다.
- 토픽 이름, 타입, 파티션, RxO를 묶어 "매칭되는가"를 답하는 함수는 여기서 만들지 않는다. 토픽과 타입은 QoS가 아니고 디스커버리 데이터의 다른 필드이며, 검사 순서는 규격이 정하지 않았다. chapter11에서 만든다.

#### 4.1.4 절차

1. `my-code/crates/mdds-rtps/src/lib.rs`에 `pub mod qos;`를 두고 `src/qos.rs`를 만든다.
2. 값 타입을 선언한다. 모든 타입에 `Debug, Clone, Copy, PartialEq, Eq`를 파생하고, 순서가 있는 것에는 `PartialOrd, Ord`를 더한다.

   | 타입 | 변이 (선언 순서) 또는 필드 | 순서 | 출처 |
   |---|---|---|---|
   | `Duration` (enum) | `Finite(core::time::Duration)`, `Infinite`. 연관 상수 `ZERO`와 생성자 `from_secs(u64)`, `from_millis(u64)`는 모두 `const`다 (`pub const fn`) | 있음 | DDS 1.4 Duration_t |
   | `ReliabilityKind` | `BestEffort`, `Reliable` | 있음 | Sec. 2.2.3.14 |
   | `DurabilityKind` | `Volatile`, `TransientLocal`, `Transient`, `Persistent` | 있음 | Sec. 2.2.3.4 |
   | `LivelinessKind` | `Automatic`, `ManualByParticipant`, `ManualByTopic` | 있음 | Sec. 2.2.3.11 |
   | `DestinationOrderKind` | `ByReceptionTimestamp`, `BySourceTimestamp` | 있음 | Sec. 2.2.3.17 |
   | `PresentationAccessScope` | `Instance`, `Topic`, `Group` | 있음 | Sec. 2.2.3.6 |
   | `OwnershipKind` | `Shared`, `Exclusive` | 없음 | Sec. 2.2.3.9 |
   | `DataRepresentationId` (struct) | `pub i16`. 상수 `XCDR1` = 0, `XML` = 1, `XCDR2` = 2 | 없음 | XTypes 1.3 Sec. 7.6.3.1.1 |
   | `Presentation` (struct) | `access_scope: PresentationAccessScope`, `coherent_access: bool`, `ordered_access: bool` | 없음 | Sec. 2.2.3.6 |
   | `Liveliness` (struct) | `kind: LivelinessKind`, `lease_duration: Duration` | 없음 | Sec. 2.2.3.11 |

   `Presentation`과 `Liveliness`는 `Default`를 직접 구현한다 (`impl Default`). 값은 3단계 표의 `presentation`, `liveliness` 행이다.

   완성본의 첫 두 타입은 다음과 같다.
   `code/crates/mdds-rtps/src/qos.rs`
   ```rust
   /// 시간 길이. DDS 1.4의 Duration_t 의미이고, 무한대가 가장 크다 (선언 순서로 `Ord`를 파생).
   ///
   /// RTPS의 와이어 표현 `Duration_t` (seconds + fraction, RTPS 2.5 Sec. 9.3.2.3)와는 다른 타입이다.
   /// 와이어 변환은 디스커버리 쪽에서 한다.
   #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
   pub enum Duration {
       Finite(core::time::Duration),
       Infinite,
   }
   ```
   ```rust
   /// DDS 1.4 Sec. 2.2.3.14. BEST_EFFORT < RELIABLE.
   #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
   pub enum ReliabilityKind {
       BestEffort,
       Reliable,
   }
   ```
3. `PublicationQos`와 `SubscriptionQos`를 선언한다 (`Debug, Clone, PartialEq, Eq`. `Vec` 필드가 있어 `Copy`는 아니다). 필드는 둘 다 같다.

   | 필드 | 타입 | `PublicationQos` 기본값 | `SubscriptionQos` 기본값 |
   |---|---|---|---|
   | `durability` | `DurabilityKind` | `Volatile` | 같음 |
   | `presentation` | `Presentation` | `Instance`, `false`, `false` | 같음 |
   | `deadline` | `Duration` | `Infinite` | 같음 |
   | `latency_budget` | `Duration` | `ZERO` | 같음 |
   | `ownership` | `OwnershipKind` | `Shared` | 같음 |
   | `liveliness` | `Liveliness` | `Automatic`, `Infinite` | 같음 |
   | `reliability` | `ReliabilityKind` | `Reliable` | `BestEffort` |
   | `destination_order` | `DestinationOrderKind` | `ByReceptionTimestamp` | 같음 |
   | `partition` | `Vec<String>` | 빈 목록 | 같음 |
   | `data_representation` | `Vec<DataRepresentationId>` | 빈 목록 | 같음 |

   기본값은 정책마다 DDS 1.4 Sec. 2.2.3.N에 있다. `Default`를 직접 구현한다 (`#[derive(Default)]`는 RELIABLE을 만들 수 없다).
4. `QosPolicyId`를 선언한다 (`#[repr(u32)]`, 변이 24개, 값은 3.4절의 표. `PartialOrd, Ord`도 파생한다).
5. `check_rxo`를 쓴다. 3.2절 표의 행 하나가 `if` 하나이고, 번호 순서대로 검사하면 결과가 오름차순이 된다.
   `code/crates/mdds-rtps/src/qos.rs`
   ```rust
   pub fn check_rxo(offered: &PublicationQos, requested: &SubscriptionQos) -> Vec<QosPolicyId> {
       let mut incompatible = Vec::new();

       // DDS 1.4 Sec. 2.2.3.4: offered kind >= requested kind
       if offered.durability < requested.durability {
           incompatible.push(QosPolicyId::Durability);
       }
       // DDS 1.4 Sec. 2.2.3.6: access_scope는 offered >= requested,
       // coherent_access와 ordered_access는 requested가 FALSE이거나 둘 다 TRUE
       if !presentation_compatible(offered.presentation, requested.presentation) {
           incompatible.push(QosPolicyId::Presentation);
       }
       // DDS 1.4 Sec. 2.2.3.7: offered period <= requested period
       if offered.deadline > requested.deadline {
           incompatible.push(QosPolicyId::Deadline);
       }
       // ... (LATENCY_BUDGET, OWNERSHIP, LIVELINESS, RELIABILITY, DESTINATION_ORDER,
       //      DATA_REPRESENTATION도 같은 형식. 완성본 참고)
       incompatible
   }
   ```
   DATA_REPRESENTATION은 writer 목록의 첫 원소 (없으면 `XCDR1`)가 reader 목록 (비어 있으면 `[XCDR1]`)에 있는지 본다.
6. `partitions_match`를 쓴다. 시그니처는 다음과 같다. 제네릭인 이유는 테스트에서 `&["A"]` (`&[&str]`)와 `Vec<String>`을 둘 다 넘기기 위해서다.
   `code/crates/mdds-rtps/src/qos.rs`
   ```rust
   pub fn partitions_match<A: AsRef<str>, B: AsRef<str>>(offered: &[A], requested: &[B]) -> bool {
       let offered = normalize(offered);
       let requested = normalize(requested);
       offered.iter().any(|o| {
           requested
               .iter()
               .any(|r| name_matches(o, r) || name_matches(r, o))
       })
   }
   ```
   보조 함수 셋의 시그니처는 다음과 같다 (모두 모듈 안에서만 쓰므로 `pub`이 아니다).
   ```rust
   fn normalize<S: AsRef<str>>(names: &[S]) -> Vec<&str>
   fn name_matches(pattern: &str, name: &str) -> bool
   fn glob(pattern: &[u8], name: &[u8]) -> bool
   ```
   `normalize`는 빈 목록을 `[""]`로 바꾸고, 아니면 각 이름을 `&str`로 바꾼 목록을 돌려준다. `name_matches`는 패턴에 와일드카드 (`*`, `?`)가 없으면 두 문자열이 같은지, 이름에도 와일드카드가 있으면 거짓, 아니면 `glob(pattern.as_bytes(), name.as_bytes())`를 돌려준다. `[`는 와일드카드로 보지 않고 문자 그대로 비교한다 (chapter14).

   `glob`은 바이트 단위로 `*`와 `?`를 해석한다. 재귀 대신, 마지막으로 만난 `*`의 위치와 그때의 이름 위치를 기억해 두었다가 뒤에서 맞지 않으면 거기로 되돌아간다 (backtracking). 뼈대는 다음과 같다.
   ```rust
   fn glob(pattern: &[u8], name: &[u8]) -> bool {
       let (mut p, mut n) = (0, 0);
       let mut star: Option<(usize, usize)> = None; // (마지막 `*`의 다음 위치, 그때의 n)
       while n < name.len() {
           match pattern.get(p) {
               Some(b'*') => { /* star = Some((p + 1, n)), p를 한 칸 */ }
               Some(b'?') => { /* p, n을 한 칸씩 */ }
               Some(&c) if c == name[n] => { /* p, n을 한 칸씩 */ }
               _ => { /* star가 있으면 `*`가 한 문자를 더 맞춘 것으로 하고
                         p = 그 다음 위치, n = 그때의 n + 1, star의 n도 갱신.
                         없으면 return false */ }
           }
       }
       pattern[p..].iter().all(|&c| c == b'*') // 남은 패턴은 `*`뿐이어야 한다
   }
   ```
7. 테스트를 쓴다. 모듈 안 `#[cfg(test)] mod tests`에 두고 `use super::*;`로 시작한다. 헬퍼 셋을 먼저 둔다: `offered()`와 `requested()`는 각각 `PublicationQos::default()`와 `SubscriptionQos::default()`를 돌려주고, `names(&[&str]) -> Vec<String>`은 파티션 목록을 만든다. 테스트는 다음과 같다.

   | 테스트 | 확인하는 것 |
   |---|---|
   | `kind_order_matches_spec` | 변이 순서가 규격의 순서다 (`<`로 단언) |
   | `policy_id_values_match_dds_1_4`, `data_representation_values_match_xtypes_1_3` | 번호와 식별자 값 |
   | `duration_infinite_is_greatest` | 무한대가 어떤 유한한 값보다 크다 |
   | `defaults_match_dds_1_4`, `default_writer_and_default_reader_are_compatible` | 기본값과 기본값끼리의 호환 |
   | 정책마다 하나 (`reliability_...`, `durability_...`, `deadline_...`, `latency_budget_...`, `ownership_...`, `liveliness_...`, `destination_order_...`, `presentation_...`, `data_representation_...`) | 불호환 쌍 하나와 호환 쌍 하나. 시간은 같은 값 (`<=`)도 호환 |
   | `multiple_incompatible_policies_are_returned_in_id_order` | 둘 이상 어긋나면 번호 오름차순 |
   | `partition_empty_list_equals_empty_name`, `partition_literal_names`, `partition_wildcards_on_either_side`, `partition_two_wildcard_names_never_match`, `partition_star_backtracks`, `partition_bracket_is_literal_for_now` | 3.3절의 규칙. `a*b*c`는 `axxbxxc`와 일치하고 `axxbxx`와는 불일치 |

   `code/crates/mdds-rtps/src/qos.rs`
   ```rust
   #[test]
   fn reliability_best_effort_offered_vs_reliable_requested_is_incompatible() {
       let w = PublicationQos {
           reliability: ReliabilityKind::BestEffort,
           ..offered()
       };
       let r = SubscriptionQos {
           reliability: ReliabilityKind::Reliable,
           ..requested()
       };
       assert_eq!(check_rxo(&w, &r), vec![QosPolicyId::Reliability]);
       // 반대 방향 (RELIABLE 제공, BEST_EFFORT 요구)은 호환
       let w = PublicationQos {
           reliability: ReliabilityKind::Reliable,
           ..offered()
       };
       let r = SubscriptionQos {
           reliability: ReliabilityKind::BestEffort,
           ..requested()
       };
       assert_eq!(check_rxo(&w, &r), vec![]);
   }
   ```
8. `my-code/`에서 검사를 실행한다.
   ```bash
   cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test -p mdds-rtps
   ```

#### 4.1.5 기대 결과

`cargo test -p mdds-rtps`의 마지막 줄이 `test result: ok. N passed; 0 failed`다 (N은 자기가 쓴 테스트 수. 완성본은 이 lab의 것이 22개이고 Lab 5-4의 6개를 더해 28개다). clippy 경고가 없다. 완성본과 비교하려면 1.1절의 diff 명령을 쓴다.

#### 4.1.6 확인 질문

- `DurabilityKind`의 변이 순서를 `Persistent, Transient, TransientLocal, Volatile`로 바꾸면 어느 테스트들이 실패하는가? 규격의 어느 문장이 그 순서를 정하는가?

#### 4.1.7 실패하면

- `partitions_match::<&str, &str>(&[], &[])`처럼 빈 배열을 넘길 때는 타입을 적어 주어야 컴파일된다. 빈 배열에서는 원소 타입을 추론할 수 없기 때문이다.
- `..offered()` 문법 (struct update)이 안 되면 `Clone`이 빠진 것이고, `assert_eq!`가 안 되면 `Debug`나 `PartialEq`가 빠진 것이다.
- 테스트 수가 22가 아니어도 된다. 표의 항목이 모두 있는지만 본다.
- `diff -r`에서 차이가 나면 먼저 `cargo fmt`를 실행한다. 형식 차이가 사라진 뒤 남는 것이 내용 차이다.

### 4.2 설계 메모

- **왜 `Vec<QosPolicyId>`인가.** DDS의 IncompatibleQosStatus는 어긋난 정책마다 횟수와 마지막 정책 번호를 갖는다 (chapter14). 어긋난 정책 목록이면 그것을 만들 수 있다. 정책이 9개로 고정이므로 비트마스크 (정책마다 비트 하나를 두는 정수. 예: `u32`)로 두면 할당이 없다. 학습용으로는 목록이 읽기 쉬워 그대로 둔다.
- **왜 `glob`을 재귀로 쓰지 않는가.** 재귀로 쓰면 `*`마다 갈림길이 생겨 `a*a*a*b` 같은 패턴에서 시간이 패턴 길이에 대해 지수로 늘어난다. 마지막 `*`로만 되돌아가는 방식은 최악의 경우에도 패턴 길이와 이름 길이의 곱에 비례한다.
- **API 쪽 타입과의 관계.** chapter11의 최소 API는 `PublicationQos` / `SubscriptionQos`를 그대로 받는다. DDS API의 QoS 묶음 (DataWriterQos 등. 와이어에 실리지 않는 HISTORY, RESOURCE_LIMITS를 포함하고 PARTITION은 PublisherQos에 있다)은 chapter14에서 만들고, 그것을 이 타입으로 바꾸는 함수를 둔다.

## 5. 벤더와 맞춰 보기

### 5.1 Lab 5-4: 캡처의 QoS 쌍

#### 5.1.1 목표

Lab 5-1과 5-2에서 읽은 writer / reader QoS 쌍을 테스트로 적고, `check_rxo`와 `partitions_match`의 판정이 벤더의 로그와 같은지 확인한다.

#### 5.1.2 준비

Lab 5-3을 마친 `my-code/crates/mdds-rtps/src/qos.rs`의 테스트 모듈에 이어서 쓴다. 캡처는 Lab 5-1, 5-2의 다섯 개에 [`xcdr2.pcap`](../../captures/cyclone-fastdds/README.md#xcdr2) (양쪽 `-x 2`. chapter4의 Lab 4-4)을 더한 여섯 개다.

#### 5.1.3 절차

캡처마다 테스트 하나를 쓴다. 이름은 `capture_<캡처 이름>_...`로 하고, 어느 프레임에서 읽었는지 주석에 적는다. 정보에 없는 PID는 그 엔티티의 기본값이므로 `..offered()` / `..requested()`가 채운다. Wireshark의 `XCDR_DATA_REPRESENTATION (0x0)`은 `DataRepresentationId::XCDR1`, `XCDR2_DATA_REPRESENTATION (0x2)`는 `XCDR2`다. 아직 pcap을 읽는 코드가 없으므로 값은 Wireshark에서 직접 옮긴다 (chapter7 뒤에는 캡처에서 바로 읽는 테스트로 바뀐다).

| 캡처 | writer (offered) | reader (requested) | 기대 판정 | 로그 |
|---|---|---|---|---|
| reliable-cyclone-pub | 22: RELIABLE | 47: VOLATILE, RELIABLE, [XCDR] | 호환 | matched |
| best-effort-cyclone-pub | 27: BEST_EFFORT | 32: VOLATILE, [XCDR] | 호환 | matched |
| reliable-fastdds-pub | 17: VOLATILE, [XCDR] | 23: RELIABLE | 호환 | matched |
| xcdr2 | 29: RELIABLE, [XCDR2] | 38: VOLATILE, RELIABLE, [XCDR2] | 호환 | matched |
| qos-mismatch | 28: BEST_EFFORT | 35: VOLATILE, RELIABLE, [XCDR] | `[Reliability]` | `11 (RELIABILITY)` |
| partition-mismatch | 23: RELIABLE, 파티션 없음 | 33: VOLATILE, RELIABLE, `["A"]`, [XCDR] | RxO 호환, 파티션 불일치 | 없음 |

`code/crates/mdds-rtps/src/qos.rs`
```rust
#[test]
fn capture_qos_mismatch_reports_reliability() {
    // 28: Cyclone writer 정보. PID_RELIABILITY BEST_EFFORT
    let w = PublicationQos {
        reliability: ReliabilityKind::BestEffort,
        ..offered()
    };
    // 35: Fast DDS reader 정보. DURABILITY VOLATILE, RELIABILITY RELIABLE, DATA_REPRESENTATION [XCDR]
    let r = SubscriptionQos {
        durability: DurabilityKind::Volatile,
        reliability: ReliabilityKind::Reliable,
        data_representation: vec![DataRepresentationId::XCDR1],
        ..requested()
    };
    // 로그: on_offered_incompatible_qos / on_requested_incompatible_qos ... 11 (RELIABILITY)
    assert_eq!(check_rxo(&w, &r), vec![QosPolicyId::Reliability]);
    assert_eq!(QosPolicyId::Reliability as u32, 11);
}
```

#### 5.1.4 기대 결과

여섯 테스트가 모두 통과한다. 표의 "기대 판정"이 로그와 일치한다. 특히 reliable-fastdds-pub은 writer 정보에 RELIABILITY가 없는데 reader가 RELIABLE을 요구한다. `PublicationQos::default()`의 RELIABLE이 대신 쓰여 호환이 된다. 기본값을 reader의 것 (BEST_EFFORT)으로 잘못 두면 이 테스트가 실패한다.

#### 5.1.5 확인 질문

- xcdr2 캡처에서 reader를 `-x 1`로 바꾸면 (writer만 XCDR2) `check_rxo`는 무엇을 돌려주는가?

#### 5.1.6 실패하면

- reliable-fastdds-pub 테스트만 실패하면 `PublicationQos`의 기본 RELIABILITY가 BEST_EFFORT로 되어 있는 것이다.
- partition-mismatch 테스트에서 `partitions_match`가 참이면 빈 목록을 `[""]`로 바꾸지 않은 것이다 (빈 목록끼리만 참이 되어야 한다).

## 6. 두 벤더는 어떻게 하는가

두 벤더 모두 상대의 SEDP 정보를 받으면 토픽 이름, 타입, QoS, 파티션을 검사해 매칭을 정한다. 검사 항목과 순서가 다르다.

| 항목 | Fast DDS 3.6.1 | Cyclone DDS 11.0.1 (`f2aaccb4`) | mdds |
|---|---|---|---|
| 검사 순서 | 토픽 이름, 타입, key 유무, QoS, 파티션 | 토픽 이름, 파티션, QoS, 타입 | 정하지 않는다 (`check_rxo`와 `partitions_match`가 따로 있다. 묶는 것은 chapter11) |
| 파티션을 보는 시점 | 마지막. QoS가 하나라도 어긋나면 파티션은 보지 않는다 | QoS보다 먼저. 파티션이 다르면 QoS 불일치를 보고하지 않는다 (소스 주석: 파티션은 RxO 정책이 아니기 때문) | 따로 판정 |
| RxO 정책 가운데 검사하는 것 | RELIABILITY, DURABILITY, OWNERSHIP, DEADLINE, LIVELINESS 5개 (그 밖에 벤더 전용 정책 하나와 DATA_REPRESENTATION). PRESENTATION, LATENCY_BUDGET, DESTINATION_ORDER는 검사하지 않는다 | 8개 모두와 DATA_REPRESENTATION | 8개 모두와 DATA_REPRESENTATION |
| 어긋난 것을 보고하는 방식 | 모두 비트마스크에 모은다 | 첫 불일치 하나만 돌려준다 | 모두를 번호 오름차순의 `Vec`으로 |
| 타입 비교 | TypeInformation이 양쪽에 있으면 그 동일성, 없으면 타입 이름 | QoS 다음에 검사한다 (방식은 chapter16) | 하지 않는다 (chapter11, 16) |

근거: Fast DDS `src/cpp/rtps/builtin/discovery/endpoint/EDP.cpp`의 `EDP::valid_matching`, Cyclone `src/core/ddsi/src/ddsi_qosmatch.c`의 `ddsi_qos_match_mask_p`와 `src/core/ddsi/src/ddsi_discovery_endpoint.c` (없는 PID를 엔티티별 기본값으로 채운다).

mdds에 적용하면 이렇다. 두 벤더 모두 kind를 숫자 순서로 비교하고 없는 PID를 엔티티별 기본값으로 채우므로, 3.2절의 규칙과 `Default`는 두 벤더와 같다. 다른 것은 네 가지다. Fast DDS는 mdds가 불일치로 판정한 쌍 (PRESENTATION 등이 어긋난 쌍)을 매칭할 수 있다. 파티션과 QoS가 동시에 어긋나면 Fast DDS는 incompatible을 보고하고 Cyclone은 보고하지 않는다. 둘 이상 어긋나면 로그에 찍히는 정책 번호가 벤더마다 다를 수 있다. 두 이름이 모두 와일드카드일 때 Fast DDS는 규격과 달리 일치할 수 있다 (Linux에서는 fnmatch를 양방향으로 적용하고 이 경우를 막지 않는다. Cyclone은 `*`와 `?`만 와일드카드로 보고 이 경우를 불일치로 본다). 어느 쪽에 맞출지는 chapter11에서 정한다.

## 7. 스스로 확인하기

명령이나 수치를 외웠는지가 아니라 동작의 원리를 이해했는지 확인하는 질문이다. 먼저 스스로 답해 본 뒤 "해설"을 펼쳐 비교해 보라.

1. writer가 BEST_EFFORT이고 reader가 RELIABLE이면 왜 불일치이고, 반대는 왜 호환인가?

   <details>
   <summary>해설</summary>

   RELIABILITY의 호환 조건은 offered kind >= requested kind이고 BEST_EFFORT < RELIABLE이다 (DDS 1.4 Sec. 2.2.3.14). reader가 RELIABLE을 요구한다는 것은 빠진 샘플을 다시 받겠다는 뜻인데, BEST_EFFORT writer는 다시 보내지도, HEARTBEAT로 가진 것을 알리지도 않으므로 (chapter3 3.2) 그 요구를 채울 수 없다. 반대로 reader가 BEST_EFFORT를 요구하면 받기만 하면 되고, RELIABLE writer가 보내는 HEARTBEAT는 무시하면 된다. Lab 5-2의 `qos-mismatch`에서 구독자를 `-b`, 발행자를 `-r`로 바꾸면 이 경우가 되어 매칭된다. 모든 RxO 규칙이 같은 방향이다. writer가 더 많이 제공하는 것은 괜찮고, 충족하지 못하면 불일치다.

   </details>

2. 두 shape_main의 로그에 `matched`도 `incompatible`도 없고 데이터도 오지 않는다. 무엇을 먼저 확인하는가? 왜 로그가 없는가?

   <details>
   <summary>해설</summary>

   파티션 (shape_main의 `-p`)을 먼저 확인한다. RxO 불일치는 양쪽에 `incompatible_qos`가 남지만, 파티션 불일치는 규격이 incompatible QoS로 보지 않고 Listener도 호출하지 않는다고 정했기 때문에 (Sec. 2.2.3.13) 아무 흔적이 없다. 그다음은 토픽 이름과 타입 이름이다 (shape_main의 `-t`. 다르면 아예 같은 토픽이 아니다). 파티션이 같은데도 로그가 없으면 디스커버리 자체를 의심한다 (chapter3 2.1.5).

   </details>

3. Fast DDS가 `-r`인 writer에 PID_RELIABILITY를 싣지 않는 것은 규격 위반인가? mdds는 기본값인 정책을 생략해도 되는가?

   <details>
   <summary>해설</summary>

   위반이 아니다. RTPS 2.5 Sec. 8.5.4.4는 정보에 없는 항목을 기본값으로 가정할 수 있다고 적었고, DataWriter의 RELIABILITY 기본값이 RELIABLE이므로 (DDS 1.4 Sec. 2.2.3.14) 생략해도 뜻이 같다. mdds도 생략할 수 있다. 다만 받는 쪽에서는 writer 정보와 reader 정보의 기본값이 다르다는 것을 기억해야 한다. 같은 "RELIABILITY 없음"이 writer 정보에서는 RELIABLE, reader 정보에서는 BEST_EFFORT다. `PublicationQos`와 `SubscriptionQos`의 `Default`가 다른 이유이고, Lab 5-4의 reliable-fastdds-pub 테스트가 그것을 확인한다.

   </details>

4. writer가 DURABILITY VOLATILE, DEADLINE 1초이고 reader가 DURABILITY TRANSIENT_LOCAL, DEADLINE 500 ms이면 `check_rxo`는 무엇을 돌려주는가? 두 벤더의 로그에는 어느 번호가 남겠는가?

   <details>
   <summary>해설</summary>

   `[Durability, Deadline]` (번호 2, 4). DURABILITY는 offered (VOLATILE) < requested (TRANSIENT_LOCAL)라 불일치이고, DEADLINE은 offered (1초) > requested (500 ms)라 불일치다. 로그는 벤더마다 다를 수 있다. Fast DDS는 둘 다 마스크에 모으고, Cyclone은 검사 순서상 먼저 만나는 DURABILITY 하나만 돌려준다 (6절). 규격은 어느 쪽을 "마지막 정책"으로 보고할지 정하지 않았다.

   </details>

5. Lab 5-4의 xcdr2 캡처에서 reader만 `-x 1`로 바꾸면 (writer는 XCDR2, reader는 XCDR) 두 벤더도 이 쌍을 불일치로 보는가?

   <details>
   <summary>해설</summary>

   소스로 보면 둘 다 불일치로 본다. `check_rxo`는 `[DataRepresentation]` (번호 23)을 돌려준다. writer가 제공하는 표현 (목록의 첫 원소 XCDR2)이 reader의 목록 [XCDR]에 없기 때문이다 (XTypes 1.3 Sec. 7.6.3.1.1). 6절의 표에서 DATA_REPRESENTATION은 두 벤더의 검사 목록에 모두 있다. Fast DDS가 검사하지 않는 것은 PRESENTATION, LATENCY_BUDGET, DESTINATION_ORDER뿐이다. 이 책에서 이 조합을 직접 실행해 보지는 않았으므로, 로그에 무엇이 찍히는지는 직접 실행해 확인한다.

   </details>

## 참고 자료

- [DDS 1.4 규격 (PDF)](https://www.omg.org/spec/DDS/1.4/PDF), Sec. 2.2.3 Supported QoS: QoS 정책 표 (적용 엔티티, RxO, 변경 가능)와 정책별 호환 조건 (3.2절, 3.4절), Sec. 2.2.3.13 PARTITION (3.3절), Sec. 2.2.1.2.2 엔티티 모델 (3.1절), Annex A 적합성 프로파일 (3.5절)
- [DDS-XTypes 1.3 규격 (PDF)](https://www.omg.org/spec/DDS-XTypes/1.3/PDF), Sec. 7.6.3.1.1 DataRepresentationQosPolicy: request-offer 규칙과 기본값 (3.2절)
- [DDSI-RTPS 2.5 규격 (PDF)](https://www.omg.org/spec/DDSI-RTPS/2.5/PDF), Sec. 8.5.4.4 (정보에 없는 항목은 기본값으로 가정), Sec. 9.6.3 표 9.19 (SEDP 정보에 실리는 PID) (Lab 5-1, 3.4절)
- [The Open Group Base Specifications, fnmatch](https://pubs.opengroup.org/onlinepubs/9699919799/functions/fnmatch.html): 파티션 이름의 와일드카드 규칙 (3.3절)
- [Fast DDS v3.6.1 `EDP.cpp`](https://github.com/eProsima/Fast-DDS/blob/v3.6.1/src/cpp/rtps/builtin/discovery/endpoint/EDP.cpp): `EDP::valid_matching` (6절)
- [Cyclone DDS `f2aaccb4` `ddsi_qosmatch.c`](https://github.com/eclipse-cyclonedds/cyclonedds/blob/f2aaccb4/src/core/ddsi/src/ddsi_qosmatch.c): `ddsi_qos_match_mask_p` (6절)
