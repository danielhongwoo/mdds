# 저자 노트: DCPS 개념 모델

상태: 완료

[roadmap](../roadmap.md) | chapter: 5 ([목차](../../book/README.md#목차)) | 본문: [book/05-dcps-concepts/README.md](../../book/05-dcps-concepts/README.md)

## 범위
- 질문: 무엇이 맞아야 writer와 reader가 매칭되는가. chapter 3, 4에서 본 SEDP의 writer 정보 / reader 정보에 실린 QoS가 어떻게 매칭을 가르는지.
- 읽을 규격: DDS 1.4 Sec. 2.2.1.2.2 (엔티티), Sec. 2.2.2.5.1 (SampleInfo의 상태, 개요만), Sec. 2.2.3 (QoS 표와 정책별 호환 규칙), Annex A (적합성 프로파일). RTPS 2.5 표 9.19 (SEDP에 실리는 QoS). XTypes 1.3 Sec. 7.6.3.1.1 (DATA_REPRESENTATION의 request-offer 규칙).
- 구현: `crates/mdds-rtps/src/qos.rs`. QoS 값 타입, `PublicationQos` / `SubscriptionQos`, `QosPolicyId`, `check_rxo` (RxO 8개 + DATA_REPRESENTATION), `partitions_match`. 이 책의 첫 코드다.
- 미루는 것: 토픽 이름 / 타입 / 파티션 / RxO를 묶는 매칭 함수와 그 결과 타입 (chapter 11, `DiscoveredWriterData`를 받는다). 타입 호환 (TypeInformation, assignability. chapter 16). 인스턴스 수명주기와 SampleInfo 구현 (chapter 13). 상태 (IncompatibleQosStatus), Listener, 한 엔티티 안의 정책 일관성 검사 (DEADLINE >= TIME_BASED_FILTER 등), 나머지 QoS 타입 (chapter 14). 파티션 와일드카드 `[...]` (chapter 14. 지금은 문자 그대로 비교한다).

## 틀
1. 왜 필요한가. 1.1 이 책의 코드가 사는 곳 (워크스페이스, `cargo test -p mdds-rtps`, 스냅샷 `code/`).
2. 캡처에서 보기. Lab 5-1 와이어에 실리는 QoS (기존 캡처의 SEDP PID). Lab 5-2 QoS가 맞지 않으면 (`qos-mismatch`, `partition-mismatch`, 직접 실행).
3. 규격 읽기. 3.1 엔티티와 와이어 (Publisher / Subscriber / Topic은 와이어에 없다). 3.2 requested / offered와 RxO 규칙 표 (9행). 3.3 PARTITION. 3.4 QoS 정책 지도 (전체 표). 3.5 이 책이 구현하는 범위 (Annex A).
4. 구현. Lab 5-3 `qos.rs` (타입, `check_rxo`, `partitions_match`, 단위 테스트). 4.2 설계 메모.
5. 벤더와 맞춰 보기. Lab 5-4 캡처와 로그에서 본 QoS 쌍을 테스트로.
6. 두 벤더는 어떻게 하는가 (검사 순서, 보고 방식, 와일드카드 처리의 차이).
7. 스스로 확인하기.

## 결정
- **첫 코드를 chapter 5에 둔다.** 규격의 표를 타입과 테스트로 옮기는 일이라 I/O가 없고, chapter 11의 매칭과 chapter 14의 API가 같은 타입을 쓴다. 버린 대안: 코드를 chapter 11에 두기. chapter 11은 이미 SEDP, 내장 엔드포인트, 최소 API, 첫 상호운용으로 가장 무거운 chapter다. 이 결정으로 chapter 1 / 2 본문, 노트 1, 목차 6 항목, crate 설명의 "코드는 chapter 6부터"를 5로 고쳤다.
- **독자는 루트 `crates/`가 아니라 `my-code/`에서 쓴다.** 루트 `crates/`에 저자의 완성 코드가 들어가므로 clone한 독자는 "빈 crate"를 만나지 않는다. 규칙은 [book.md "코드 스냅샷"](../conventions/book.md#코드-스냅샷). chapter 1 본문과 노트 1의 "빈 crate 세 개"를 고쳤다.
- **코드 위치는 `crates/mdds-rtps/src/qos.rs`.** mdds (API)가 mdds-rtps에 의존하고 디스커버리 매칭이 mdds-rtps에 있으므로 QoS 타입은 mdds-rtps나 그 아래에 있어야 한다. 버린 대안: 새 crate `mdds-dcps`. roadmap의 "세 개로 시작하고 경계가 분명해지면 나눈다"와 어긋나고, 지금은 모듈 하나다. 커지면 그때 뗀다.
- **타입 이름은 `PublicationQos` / `SubscriptionQos`.** DDS의 DataWriterQos / DataReaderQos가 아니라 SEDP의 PublicationBuiltinTopicData / SubscriptionBuiltinTopicData에 실리는 QoS 부분 (RTPS 2.5 표 9.19)을 모델링한다. 매칭이 보는 것이 이것이고, chapter 11의 `DiscoveredWriterData`가 이 타입을 필드로 갖고, chapter 14가 DataWriterQos + PublisherQos에서 이 타입으로 바꾼다. 기본값 (`Default`)은 PID가 없을 때의 값, 곧 DDS 1.4의 엔티티별 기본값이다 (RELIABILITY가 writer RELIABLE, reader BEST_EFFORT로 다르므로 타입을 둘로 나눈다).
- **`Duration`은 DDS 의미의 값 타입 (`Finite(core::time::Duration)` / `Infinite`).** RTPS의 `Duration_t` (seconds, fraction. 무한대 표현도 다름)는 chapter 7에서 와이어 타입으로 따로 만들고 변환은 chapter 11에서 한다. 버린 대안: 하나의 타입으로 합치기. 무한대와 나노초 / 분수 표현이 섞인다.
- **kind enum은 선언 순서가 규격의 순서이고 `Ord`를 파생한다.** 판별값을 와이어 값에 맞추지 않는다 (RTPS 2.5 Sec. 9.3.2의 BEST_EFFORT = 1, RELIABLE = 2는 PSM의 0, 1과 다르다). 순서를 단언하는 테스트를 둔다. OWNERSHIP은 같음만 본다 (`Ord` 없음).
- **`check_rxo`는 어긋난 정책을 `Vec<QosPolicyId>`로 QosPolicyId 오름차순으로 돌려준다.** DATA_REPRESENTATION (id 23)을 포함해 9개를 검사한다. 버린 대안: 비트마스크. 할당은 없지만 학습용으로는 목록이 읽기 쉽다. 설계 메모에 적었다.
- **`QosPolicyId`는 DDS 1.4의 1-22에 XTypes 1.3의 23, 24를 더한 enum (`repr(u32)`).** mdds가 내보내는 값이라 닫힌 enum으로 둔다. 벤더 전용 번호는 없다.
- **`DataRepresentationId`는 `i16`을 감싼 struct와 상수 (`XCDR1` = 0, `XML` = 1, `XCDR2` = 2).** 와이어에서 모르는 값이 와도 담아야 하므로 enum으로 두지 않는다 (chapter 11의 PID 해석에서 버리면 "첫 원소" 규칙이 틀어진다).
- **파티션 매칭은 이름 쌍 단위로 양방향 fnmatch, 두 이름이 모두 와일드카드를 포함하면 불일치, 빈 목록은 `""` 하나.** 규격 (DDS 1.4 Sec. 2.2.3.13)과 Cyclone `f2aaccb4` `ddsi_qosmatch.c`의 동작이 같다. 와일드카드는 `*`, `?`만 구현하고 `[`는 문자 그대로 비교한다 (테스트로 고정, chapter 14에서 구현).
- **매칭 함수 (`can_match`)는 chapter 11로 미룬다.** 토픽 이름과 타입은 QoS가 아니고, 타입 비교는 chapter 16에서 TypeInformation으로 바뀐다. 검사 순서 (파티션 먼저인지 RxO 먼저인지)는 규격이 정하지 않으며 두 벤더가 다르다. chapter 11에서 정한다 ([노트 11](11-sedp-matching.md)).
- **코드 스냅샷 `book/NN-slug/code/`는 그 시점의 `crates/` 전체를 복사한 작은 워크스페이스다.** 루트와 같은 `Cargo.toml` (members, workspace.package)을 두어 독자가 `cd code && cargo test --workspace`를 그대로 쓰고 `diff -r crates code/crates`로 비교한다. 버린 대안: 바뀐 crate 하나만 복사. chapter 7부터 crate 사이에 path 의존이 생겨 바로 깨진다. `Cargo.lock`은 커밋하고 `target/`은 `.gitignore`로 뺀다 ([book.md](../conventions/book.md)).
- **스스로 확인하기를 둔다 (연습 문제 대신).** 이 chapter의 핵심은 코드보다 개념 (부등호의 방향, PARTITION이 RxO가 아님)이라 해설이 있는 확인 질문이 맞다. 캡처 README의 질문 가운데 이 chapter가 답하는 것을 여기에 넣는다.
- **새 참고 캡처 둘 (`qos-mismatch`, `partition-mismatch`).** 지금까지의 캡처에는 PID_PARTITION이 실린 것이 없고, 불일치 QoS가 실린 SEDP 바이트는 chapter 11의 골든 테스트가 된다. 스크립트에는 `subReliability`와 `subExtra`를 더했다 (기존 시나리오의 인자열은 그대로임을 확인). 시나리오 표는 [노트 3](03-first-communication.md#시나리오-표).

## Lab
| Lab | 하는 일 | 기대 결과 |
|---|---|---|
| 5-1 | 기존 캡처의 SEDP DATA(w) / DATA(r)에서 QoS PID를 읽는다 | 표 (캡처 / 프레임 / 실린 QoS PID). 기본값인 정책은 생략될 수 있고 두 벤더가 생략하는 것이 다르다 |
| 5-2 | `qos-mismatch`, `partition-mismatch`를 보거나 직접 실행한다 | 사용자 DATA 없음. 양쪽 로그에 `incompatible_qos ... 11 (RELIABILITY)` / 파티션은 로그 없음 |
| 5-3 | `qos.rs`를 쓴다 | `cargo test -p mdds-rtps` 통과 (테스트 수는 본문에) |
| 5-4 | 캡처 여섯 개 (rel / best-effort-cyclone-pub, reliable-fastdds-pub, xcdr2, qos-mismatch, partition-mismatch)의 QoS 쌍을 테스트로 적는다 | `check_rxo` / `partitions_match`의 판정이 벤더 로그와 같다 |

## 확인한 것 (원문, 2026-11-03)
- DDS 1.4: RxO 정의 (5274-5280행), DURABILITY 순서와 규칙 (5818-5820), PRESENTATION 세 조건 (5920-5928), DEADLINE (5943), LATENCY_BUDGET (5958), OWNERSHIP (6015), LIVELINESS 두 조건 (6054-6067), RELIABILITY (6150-6152), DESTINATION_ORDER (6199-6201), PARTITION (6100-6118: incompatible이 아니고 listener도 울리지 않음, fnmatch, 두 이름이 모두 와일드카드면 불일치), 빈 파티션 목록 = `""` (5568-5572), RELIABILITY 기본값 (5594-5604), QosPolicyId 1-22 (8078-8100), Annex A (9114-9136), copy_from_topic_qos는 편의 연산이며 Topic QoS가 자동으로 상속되지 않음 (2914-2917).
- XTypes 1.3: DATA_REPRESENTATION request-offer 규칙 (10035-10079), id 23 (14443), 24 (14463).
- RTPS 2.5: 표 9.19 (10026 이후. Default는 "See DDS"뿐), 정보에 없는 항목은 PSM의 기본값으로 가정할 수 있다 (Sec. 8.5.4.4, 6739-6740), Topic 전파는 선택 (Sec. 8.5.4.3, 6697-6701), 와이어의 BEST_EFFORT = 1, RELIABLE = 2 (7955-7962), Duration_t는 seconds + fraction (9.3.2.3). PID_TRANSPORT_PRIORITY는 표 9.19에 없고 표 9.20 (inline QoS)에 있다.
- 캡처: `qos-mismatch` 28 (Cyclone writer 정보, PID_RELIABILITY BEST_EFFORT), 35 (Fast DDS reader 정보, PID_DURABILITY VOLATILE, PID_RELIABILITY RELIABLE, PID_DATA_REPRESENTATION XCDR). `partition-mismatch` 23, 33 (PID_PARTITION "A"). Cyclone writer 정보의 PID 전체는 Wireshark가 Malformed로 끊으므로 바이트에서 읽었다: TOPIC_NAME, TYPE_NAME, RELIABILITY, RESOURCE_LIMITS, TYPE_INFORMATION, PROTOCOL_VERSION, VENDORID, ENDPOINT_GUID, 0x800c, SENTINEL. PID_DURABILITY와 PID_DATA_REPRESENTATION은 없다.
- Fast DDS `v3.6.1` `src/cpp/rtps/builtin/discovery/endpoint/EDP.cpp` `EDP::valid_matching`: 토픽 이름, 타입, topic_kind를 먼저 보고 RELIABILITY, DURABILITY, OWNERSHIP, DEADLINE, disable_positive_acks, LIVELINESS, DATA_REPRESENTATION을 마스크에 모은 뒤 파티션을 본다. PRESENTATION, LATENCY_BUDGET, DESTINATION_ORDER는 검사하지 않는다. 파티션은 `fnmatch(a, b) || fnmatch(b, a)`로, 두 이름이 모두 와일드카드여도 막지 않는다. (로컬 체크아웃 `~/practice/Fast-DDS`는 master라 `git show v3.6.1:<path>`로 읽는다.)
- Cyclone `f2aaccb4` `src/core/ddsi/src/ddsi_qosmatch.c` `ddsi_qos_match_mask_p`: 토픽 이름, 파티션, RELIABILITY, DURABILITY, PRESENTATION (세 항목), DEADLINE, LATENCY_BUDGET, OWNERSHIP, LIVELINESS (두 항목), DESTINATION_ORDER, DATA_REPRESENTATION, 타입 순서. 첫 불일치에서 그 정책 하나만 돌려준다. 와일드카드 판정은 `*`, `?`만이고 C 문자열을 바이트 단위로 비교한다. 같은 커밋 `ddsi_discovery_endpoint.c`: 받은 정보에 없는 QoS는 `ddsi_xqos_mergein_missing`으로 `ddsi_default_qos_writer / _reader`를 채우고, 보낼 때는 기본값과 다른 것만 싣는다 (`sedp_write_endpoint_impl`). RELIABILITY를 늘 싣는 이유는 확인하지 못했다. (로컬 체크아웃 `~/practice/cyclonedds`는 6896b29a라 GitHub의 f2aaccb4 원문으로 확인했다.)
- shape_main (526dc56e): 신뢰성 기본값은 양쪽 모두 RELIABLE (`shape_main.cxx` 293행). 참가자 listener로 matched와 incompatible_qos 콜백만 출력한다. 파티션 불일치에 대응하는 status는 DDS에 없다.

## 확인할 것
- `partitions_match`의 `[...]`: chapter 14에서 구현할 때 fnmatch의 `[!...]`, 범위, 이스케이프까지 Cyclone / Fast DDS와 맞추기.
- 파티션과 RxO가 동시에 어긋날 때 두 벤더의 로그가 다른지 (Fast DDS는 incompatible 보고, Cyclone은 보고하지 않을 것으로 예상). 직접 실행해 보지 않았다. chapter 11에서 매칭 순서를 정할 때 확인.
- Fast DDS가 PRESENTATION, LATENCY_BUDGET, DESTINATION_ORDER를 검사하지 않으므로 mdds가 불일치로 판정한 쌍을 Fast DDS는 매칭할 수 있다. chapter 11의 상호운용 시험에서 어느 쪽에 맞출지.
- `glob`은 바이트 단위라 `?`가 UTF-8 여러 바이트 문자 하나와 맞지 않는다. 규격은 파티션 이름의 문자 집합을 정하지 않았다. 비ASCII 파티션 이름을 두 벤더가 어떻게 다루는지는 확인하지 않았다.
- DEADLINE, LATENCY_BUDGET을 엔티티 생성 뒤에 바꿨을 때 매칭을 다시 계산하는지는 규격 (DDS 1.4 Sec. 2.2.3.7, 2.2.3.8)에 문장이 없다 (PARTITION만 6108-6109에 있다). 두 벤더의 동작은 chapter 14에서 확인.

## 완료 기준
- QoS 표 (적용 엔티티 / RxO / 변경 가능 / SEDP 전송)를 스스로 설명할 수 있다.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`가 통과하고, 스냅샷에서도 같은 검사가 통과한다 (`cd book/05-dcps-concepts/code && cargo test --workspace`). `diff -r crates book/05-dcps-concepts/code/crates`가 비어 있다.
- Lab 5-4의 테스트가 네 캡처의 로그와 같은 판정을 낸다.

## 진행 기록
- 2026-11-03: 틀을 네 관점 (아키텍트, DDS 전문가, 시니어 엔지니어, 독자)으로 검토. 첫 코드를 5에 두기로, 타입은 표 9.19 기준, `can_match`는 11로 -> [결정](#결정)
- 2026-11-03: 새 캡처 `qos-mismatch`, `partition-mismatch`. 스크립트에 구독 전용 옵션 추가 -> [캡처 README](../../captures/cyclone-fastdds/README.md#qos-mismatch), [노트 3 시나리오 표](03-first-communication.md#시나리오-표)
- 2026-11-03: 본문 초안을 네 관점으로 검토해 반영. 독자 작업 디렉터리 `my-code/` 결정, `DataRepresentationId`를 값 타입으로, 벤더 차이 네 가지 -> [결정](#결정), [book.md](../conventions/book.md#코드-스냅샷)
