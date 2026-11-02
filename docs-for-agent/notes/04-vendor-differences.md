# 저자 노트: 같은 규격, 다른 구현 (chapter 04, 구현 없음)

상태: 완료

[roadmap](../roadmap.md) | chapter: 04 ([목차](../../book/README.md#목차)) | 이전: [03-first-communication](03-first-communication.md) | 다음: [05-dcps-concepts](05-dcps-concepts.md)

## 목표
같은 RTPS 규격을 따르는 두 구현이 실제로는 어떻게 다르게 행동하는지 캡처로 비교하고, dispose, lease 만료, 큰 샘플, XCDR2처럼 이후 chapter에서 다룰 기능이 와이어에서 어떻게 보이는지 미리 본다. 해석은 하지 않고, 관찰과 질문을 이후 chapter의 숙제로 넘긴다.

## 범위
하는 것
- 방향을 바꾼 캡처 (Fast DDS 발행)로 벤더 차이 비교
- 나머지 시나리오 4개 (dispose-on-exit, lease-expiry, large-sample, xcdr2) 캡처와 흐름도
- 두 벤더의 차이 메모

하지 않는 것
- 각 현상의 규격 근거와 구현: dispose (chapter 13), lease (chapter 09), 단편화 (chapter 15), XCDR2 (chapter 16)
- 시나리오 스크립트와 시나리오 표는 chapter 03의 것을 쓴다 ([03-first-communication](03-first-communication.md#시나리오-표))

## 산출물
| 산출물 | 위치 |
|---|---|
| 시나리오별 캡처, 양쪽 stdout 로그 | `captures/cyclone-fastdds/{dispose-on-exit,lease-expiry,large-sample,xcdr2}.*` (그리고 비교에 쓰는 `*-fastdds-pub.*`) |
| 캡처 분석 메모와 벤더 차이 | [captures/cyclone-fastdds/README.md](../../captures/cyclone-fastdds/README.md) |
| 교재 chapter | [book/](../../book/README.md) 04 |

## 작업 순서

### Step 1. 시나리오 캡처
- [시나리오 표](03-first-communication.md#시나리오-표)에서 chapter 04 행을 `capture-shape-scenario.ts`로 실행해 `captures/cyclone-fastdds/`에 저장한다.
- [x] dispose-on-exit
- [x] lease-expiry
- [x] large-sample
- [x] xcdr2

### Step 2. 캡처 분석 메모
- [x] 시나리오마다 실행 명령, 흐름 (Mermaid 다이어그램), 질문을 캡처 README에 적는다 ([captures.md](../conventions/captures.md#readme-형식)).
- [x] 두 벤더의 차이를 메모한다: 벤더 ID, 프로토콜 버전, 벤더 전용 PID, 데이터 표현 ID, SPDP의 lease 기간 값, 유니캐스트 포트 (Fast DDS 7410 / 7411, Cyclone은 임의 포트), Fast DDS SPDP에 실리는 공유 메모리 로케이터 (벤더 전용 kind), SPDP 멀티캐스트의 IP TTL (Fast DDS 1, Cyclone 32. chapter 02에서 TTL과 서브넷 경계를 설명하고 벤더 차이는 여기로 미뤘다).

## lab
규칙은 [conventions/book.md](../conventions/book.md#lab).

| lab | 제목 | 하는 일 -> 기대 결과 | 쓰는 산출물 |
|---|---|---|---|
| Lab 4-1 | 방향을 바꾸면 | `rel-fastdds-pub`과 비교 -> 벤더 ID, 프로토콜 버전, 벤더 전용 PID, lease 값, 유니캐스트 포트, 공유 메모리 로케이터가 다름 | `rel-fastdds-pub.*`, `be-fastdds-pub.*` |
| Lab 4-2 | 끝날 때 dispose | `dispose-on-exit` -> 마지막 DATA의 `PID_STATUS_INFO`에 dispose 비트, 이어서 unregister와 SEDP writer 제거 | `dispose-on-exit.*` |
| Lab 4-3 | 말없이 사라진 참가자 | `lease-expiry` -> 마지막 SPDP 뒤 약 10초 (Cyclone lease) 지나 구독자가 정리 | `lease-expiry.*` |
| Lab 4-4 | 큰 샘플 | `large-sample` -> DATA_FRAG, HEARTBEAT_FRAG, IP 단편화 | `large-sample.*` |
| Lab 4-5 | XCDR2 | `xcdr2` -> 캡슐화 식별자와 PID_DATA_REPRESENTATION이 XCDR2 | `xcdr2.*` |

## 완료 기준
- chapter 04 시나리오 4개의 캡처와 로그가 저장소에 있고, README에 흐름과 질문이 적혀 있다.
- 캡처에서 STATUS_INFO (dispose)와 DATA_FRAG를 Wireshark로 식별했고, 두 벤더의 차이를 메모했다.

## 진행 기록
결정과 막힌 점만 한 줄씩 적고 근거 문서로 링크한다. 세부 경위는 커밋 메시지에 둔다.
- 2026-10-29: 나머지 시나리오 캡처 (large-sample은 2.9MB라 자르지 않음) -> [캡처 README](../../captures/cyclone-fastdds/README.md#파일)
- 2026-10-29: 캡처 8개 모두 다이어그램과 질문을 적고 벤더 차이를 메모함 -> [캡처 README](../../captures/cyclone-fastdds/README.md)
- 2026-10-30: chapter 01-04를 함께 맡던 노트에서 나눔
