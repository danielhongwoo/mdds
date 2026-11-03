# 저자 노트: 같은 규격, 다른 구현 (chapter4, 구현 없음)

상태: 완료

[roadmap](../roadmap.md) | chapter: 4 ([목차](../../book/README.md#목차)) | 이전: [03-first-communication](03-first-communication.md) | 다음: [05-dcps-concepts](05-dcps-concepts.md)

## 목표
같은 RTPS 규격을 따르는 두 구현이 실제로는 어떻게 다르게 행동하는지 캡처로 비교하고, dispose, lease 만료, 큰 샘플, XCDR2처럼 이후 chapter에서 다룰 기능이 와이어에서 어떻게 보이는지 미리 본다. 해석은 하지 않고, 관찰과 질문을 이후 chapter의 숙제로 넘긴다.

## 범위
하는 것
- 방향을 바꾼 캡처 (Fast DDS 발행)로 벤더 차이 비교
- 나머지 시나리오 4개 (dispose-on-exit, lease-expiry, large-sample, xcdr2) 캡처와 흐름도
- 두 벤더의 차이 메모

하지 않는 것
- 각 현상의 규격 근거와 구현: dispose (chapter13), lease (chapter9), 단편화 (chapter15), XCDR2 (chapter16)
- 시나리오 스크립트와 시나리오 표는 chapter3의 것을 쓴다 ([03-first-communication](03-first-communication.md#시나리오-표))

## 산출물
| 산출물 | 위치 |
|---|---|
| 시나리오별 캡처, 양쪽 stdout 로그 | `captures/cyclone-fastdds/{dispose-on-exit,lease-expiry,large-sample,xcdr2}.*` (그리고 비교에 쓰는 `*-fastdds-pub.*`) |
| 캡처 분석 메모와 벤더 차이 | [captures/cyclone-fastdds/README.md](../../captures/cyclone-fastdds/README.md) |
| 교재 chapter | [book/](../../book/README.md) 4 |

## 틀
관찰 chapter의 틀 ([book.md](../conventions/book.md#chapter))에서 4, 5, 6을 뺐다. 중심 개념은 "규격이 값을 정한 것인가, 기본값만 준 것인가, 벤더에게 비워 둔 영역인가, 규격에 없는 것인가"다 (chapter0 6의 약속: 해야 한다와 해도 된다를 구분한다).
1. 왜 필요한가: 상호운용은 규격의 최소 요구만 같으면 되고 나머지는 구현이 고른다. mdds를 만들 때 "어느 벤더를 따라야 하나"가 아니라 "규격이 정한 것인가"를 묻는다. 뒤 chapter에서 풀 네 문제 (정상 종료를 알리는 법, 말없이 죽은 참가자를 아는 법, 큰 샘플 나르기, 바이트 표현)를 미리 본다.
2. 캡처에서 보기: Lab 4-1 두 벤더의 차이 (`reliable-fastdds-pub`과 `reliable-cyclone-pub`), Lab 4-2 사라지는 두 방법 (`dispose-on-exit` 읽기와 SIGKILL을 손으로 하는 `my-lease-expiry`, 참고 `lease-expiry`), Lab 4-3 큰 샘플 (`large-sample`), Lab 4-4 XCDR2 (`xcdr2`). lab마다 프레임 번호는 표지 표에만 두고, 캡처 README의 질문은 "이 chapter에서 답하지 않는 것"으로 둔다.
3. 규격 읽기: 3.1 규격이 정하는 것과 구현에 맡기는 것 (8.4.2의 범위, 8.4.2.1의 조정 가능한 시간과 벤더 확장, 9.6.2.3 / 9.6.2.4의 포트와 멀티캐스트 의무, 8.5.3.2 주기 < lease, 표 9.19의 lease 기본값, 8.3.6.3 버전, 8.3.4.1과 표 9.6의 벤더 영역), 3.2 사라지는 두 방법 (STATUS_INFO의 D / U, DDS 1.4의 인스턴스 상태, lease 만료)
4. 스스로 확인하기: 질문 세 개

**결정: lab은 넷이다. dispose와 lease 만료를 "사라지는 두 방법" 한 lab에 묶는다.** 두 캡처의 대비 (알리고 사라지기 / 말없이 사라지기)가 이 chapter에서 가장 중요한 관찰이고, 구독 로그의 끝이 같아 보이는 두 경우를 나란히 보아야 차이가 드러난다. 큰 샘플과 XCDR2는 프레임 몇 개만 보는 짧은 lab으로 둔다.
- 버린 대안: 노트 초안의 lab 다섯 개 (4-2 dispose, 4-3 lease 따로). 독자 검토에서 chapter3 (lab 셋)보다 길어지고 미리 보기 lab이 "프레임 한 개 보고 끝"이라는 지적이 있었다. 셋으로 줄이는 안 (큰 샘플과 XCDR2를 표 하나로)은 두 캡처가 다른 층 (전송 / 직렬화)이라 묶지 않았다.

**결정: 손으로 하는 것은 SIGKILL (Lab 4-2) 하나다.** 발행자를 `--num-iterations` 없이 실행하고 `pkill -9`로 끝낸 뒤 구독 터미널에서 `matched writers 0`까지 기다리는 경험이 lease의 뜻을 준다. dispose는 Lab 3-1의 명령에 `--final-instance-state d`를 더하는 선택 실습이다. 큰 샘플과 XCDR2는 참고 캡처만 연다 (직접 만들어도 다르지 않고, 옵션의 뜻은 15 / 16의 몫). 저자 확인 (2026-11-02): SIGKILL 뒤 구독자가 `matched writers 0`을 찍기까지 10.1초, 캡처에 `[UD]` 없음, IGMP Leave 두 번, `docker compose exec`의 종료 코드 137. `--final-instance-state d`는 `DATA([_D])`와 `NOT_ALIVE_DISPOSED`로 끝남.

**결정: 벤더 차이 표의 독자용 정본은 Lab 4-1의 기대 결과 (2.1.4)에 둔다.** 열은 항목 / Cyclone / Fast DDS / 규격 (네 값: 값을 정함, 기본값만 줌, 비워 둔 영역, 규격에 없음. 근거 절은 3.1). 캡처 README의 표는 원자료로 남기고 맨 위에 이 표를 가리키는 줄을 둔다. 뒤 chapter (7, 9, 10)는 이 소절을 링크한다.

**결정: 판정 열은 lab이 아니라 3.1절의 지식이다.** lab의 절차에서는 관찰 두 열만 채우고, 판정 열은 기대 결과의 표에 3.1절 근거와 함께 준다. 독자는 그 가운데 한두 행을 규격 원문으로 직접 확인한다.

**결정: 이 여섯 캡처도 고정 데이터다.** 다시 만들면 chapter4 본문의 표지 표, 캡처 README, 9 / 13 노트의 인용을 함께 고친다.

**결정: 추정은 본문에 쓰지 않는다.** IGMP Leave를 누가 보내는지 (-> 8)는 "답하지 않는 것"으로 둔다. lease 갱신 기준, 0x80 서브메시지, 0x13 로케이터, 조각 크기 1344, 임의 포트의 출처는 처음에 추정이었으나 벤더 소스와 설정 문서로 확인한 뒤 본문에 한 줄씩 적었다 (아래 "확인한 것").

## 작업 순서

### Step 1. 시나리오 캡처
- [시나리오 표](03-first-communication.md#시나리오-표)에서 chapter4 행을 `capture-shape-scenario.ts`로 실행해 `captures/cyclone-fastdds/`에 저장한다.
- [x] dispose-on-exit
- [x] lease-expiry
- [x] large-sample
- [x] xcdr2

### Step 2. 캡처 분석 메모
- [x] 시나리오마다 실행 명령, 흐름 (Mermaid 다이어그램), 질문을 캡처 README에 적는다 ([captures.md](../conventions/captures.md#readme-형식)).
- [x] 두 벤더의 차이를 메모한다: 벤더 ID, 프로토콜 버전, 벤더 전용 PID, 데이터 표현 ID, SPDP의 lease 기간 값, 유니캐스트 포트 (Fast DDS 7410 / 7411, Cyclone은 임의 포트), Fast DDS SPDP에 실리는 공유 메모리 로케이터 (벤더 전용 kind), SPDP 멀티캐스트의 IP TTL (Fast DDS 1, Cyclone 32. chapter2에서 TTL과 서브넷 경계를 설명하고 벤더 차이는 여기로 미뤘다).

## lab
규칙은 [conventions/book.md](../conventions/book.md#lab).

| lab | 제목 | 하는 일 -> 기대 결과 | 쓰는 산출물 |
|---|---|---|---|
| Lab 4-1 | 두 벤더의 차이 | `reliable-fastdds-pub`과 `reliable-cyclone-pub`을 나란히 -> vendorId와 버전, lease, SPDP 주기, 유니캐스트 포트와 로케이터, 벤더 전용 PID와 서브메시지, HEARTBEAT 빈도, INFO_DST, SPDP TTL, 사용자 writer entityId가 다름. 표의 "규격" 열은 3.1절 | `reliable-fastdds-pub.*`, `reliable-cyclone-pub.*` (`best-effort-fastdds-pub.*`은 본문에서 열지 않는다. 그 캡처의 질문 (seq 2부터, ACKNACK 뒤의 HB, 종료 때 DATA(p[UD]) 없음)은 모두 reliable-fastdds-pub의 질문과 같아 Lab 4-1의 "답하지 않는 것"이 함께 가진다) |
| Lab 4-2 | 사라지는 두 방법 | `dispose-on-exit`의 끝 (`DATA([_D])`, `DATA(w[UD])`, `NOT_ALIVE_DISPOSED`)을 읽고, 발행자를 `pkill -9`로 끝내는 `my-lease-expiry`를 손으로 만들어 구독자가 약 10초 뒤 `matched writers 0`을 찍는 것을 본다. 참고 `lease-expiry`에서 `[UD]` 없음, IGMP Leave, Fast DDS의 유니캐스트 SPDP가 멈추는 시각 | `dispose-on-exit.*`, `lease-expiry.*` |
| Lab 4-3 | 큰 샘플 | `large-sample` -> IP 조각 행, 재조립된 프레임의 DATA_FRAG 필드 (fragmentSize 1344, sampleSize 100032, 10조각씩), HEARTBEAT_FRAG, 샘플 하나 = 조각 75개 = 메시지 8개 | `large-sample.*` |
| Lab 4-4 | XCDR2 | `xcdr2` -> encapsulation kind `D_CDR2_LE` (0x0009), 페이로드 앞의 DHEADER 4바이트, SEDP의 PID_DATA_REPRESENTATION XCDR2 | `xcdr2.*` |

## 확인한 것 (벤더 소스와 설정 문서, 2026-11-03)
- Fast DDS 3.6.1 (`v3.6.1` 태그)
  - lease 갱신: 상대 참가자의 어떤 RTPS 메시지든 받으면 갱신한다. `src/cpp/rtps/messages/MessageReceiver.cpp`의 `processCDRMsg` 끝에서 `assert_remote_participant_liveliness(source_guid_prefix_)`를 부르고, `ParticipantProxyData::assert_liveliness()`가 `last_received_message_tm_`를 지금으로 놓는다. `PDP::check_remote_participant_liveliness`는 그 시각 + `lease_duration`이 지났으면 참가자를 지운다. lease-expiry 캡처의 "마지막 패킷 + 10초"와 맞는다.
  - 로케이터 종류 0x13: `include/fastdds/rtps/common/Locator.hpp`의 `LOCATOR_KIND_SHM` = `16 + FASTDDS_VERSION_MAJOR` = 19. 공유 메모리 전송의 로케이터다.
  - 서브메시지 0x80: `src/cpp/statistics/rtps/messages/RTPSStatisticsMessages.hpp`의 `FASTDDS_STATISTICS_NETWORK_SUBMESSAGE`. 내용은 `StatisticsSubmessageData` (목적지 로케이터 24 + 시각 8 + 순번과 누적 바이트 24 = 56바이트). `RTPSMessageGroup.cpp`가 메시지마다 붙이고 `OutputTrafficManager.hpp`가 보낼 때 채운다. CMake 옵션 `FASTDDS_STATISTICS`의 기본값이 ON이라 이 빌드에 들어 있다.
- Cyclone DDS `f2aaccb4` (`docs/manual/config/config_file_reference.rst`)
  - `General/FragmentSize` 기본값 1344 B. `General/MaxMessageSize` 기본값 14720 B (UDP 페이로드 상한). 조각 10개 (13,440) + 헤더가 이 안에 들어가고 11개는 넘는다. large-sample의 "메시지당 10조각"은 이 둘의 결과다.
  - `Discovery/ParticipantIndex` 기본값 `default`: 선택한 모든 인터페이스에서 멀티캐스트 디스커버리를 쓰면 `none`, 아니면 `auto`. `none`은 유니캐스트 소켓에 임의 포트를 쓰고 (participantId 제약이 없어짐) 유니캐스트 디스커버리를 불가능하게 한다. 캡처의 임의 포트 (44686 등)가 이것이다.
  - `General/MulticastTimeToLive` 기본값 32. `Discovery/LeaseDuration` 기본값 10 s. `Discovery/SPDPInterval` 기본값 `default` = lease의 약 80% (최대 30 s). 캡처의 8초 주기가 이것이다.

## 확인할 것
- RepresentationIdentifier 값의 불일치 (RTPS 2.5 표 10.3, XTypes 1.3 표 39와 표 60). 16 노트로 옮겼다.
- Wireshark 4.6.9가 Cyclone의 SEDP writer 정보를 PID_TYPE_INFORMATION (0x0075)에서 Malformed로 표시해 그 뒤의 PID가 화면에 보이지 않는다. Cyclone의 바이트가 규격에 맞는지는 판단하지 않았다 (-> 16).

## 완료 기준
- chapter4 시나리오 4개의 캡처와 로그가 저장소에 있고, README에 흐름과 질문이 적혀 있다.
- 캡처에서 STATUS_INFO (dispose)와 DATA_FRAG를 Wireshark로 식별했고, 두 벤더의 차이를 메모했다.

## 진행 기록
결정과 막힌 점만 한 줄씩 적고 근거 문서로 링크한다. 세부 경위는 커밋 메시지에 둔다.
- 2026-10-29: 나머지 시나리오 캡처 (large-sample은 2.9MB라 자르지 않음) -> [캡처 README](../../captures/cyclone-fastdds/README.md#파일)
- 2026-10-29: 캡처 8개 모두 다이어그램과 질문을 적고 벤더 차이를 메모함 -> [캡처 README](../../captures/cyclone-fastdds/README.md)
- 2026-10-30: chapter1-4를 함께 맡던 노트에서 나눔
- 2026-11-03: Fast DDS v3.6.1 소스와 Cyclone f2aaccb4 설정 문서로 lease 갱신 기준, 0x13 로케이터, 0x80 서브메시지, 조각 크기와 임의 포트의 출처를 확인 -> [확인한 것](#확인한-것-벤더-소스와-설정-문서-2026-11-03)
- 2026-11-03: 아키텍트, DDS 전문가, 시니어 엔지니어, 독자 네 관점의 검토로 틀을 정하고 본문 초안 작성 -> [틀](#틀). lab을 다섯에서 넷으로 (dispose와 lease 만료를 "사라지는 두 방법"으로 묶음), 규격 읽기는 둘로. 판정 열은 네 값 (값을 정함 / 기본값만 줌 / 비워 둔 영역 / 규격에 없음). 검토에서 고친 사실: 8.3.6.3은 "major가 같으면 된다"가 아니라 "지원하는 것보다 큰 major만 무효", PID_DATA_REPRESENTATION은 Cyclone이 기본값일 때 생략하는 것, ShapeType은 멤버 다섯 개의 @appendable 타입 (용어집과 6 노트도 고침)
- 2026-11-03: Ubuntu에서 Lab 4-2를 실행해 lease 만료 10.1초를 재확인. Lab 4-1 표의 Fast DDS 프레임 32, 33 Info 열을 Wireshark 4.6.9의 실제 표기 (`DATA -> Square`, `Unknown[80]`)로 고침
- 2026-11-04: 학습자 관점 검토로 고침. 서문을 lab 순서의 세 현상으로, Lab 4-1 절차 표를 항목마다 한 행 (펼칠 위치와 읽을 값)으로, PID와 lease는 첫 자리에서 풀이, Lab 4-1의 lease 확인 질문을 Lab 4-2로 옮김 (답이 Lab 4-2 기대 결과에 있다), 2.1.4 끝의 벤더 설정 이름은 본문에서 빼고 이 노트의 "확인할 것"에만 둠, Lab 4-3 수치를 한 줄 사슬로, Lab 4-4에 바이트와 멤버 대응 표
