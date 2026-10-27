# Phase 03 - RTPS 메시지 계층

상태: 대기

[roadmap](../roadmap.md) | 이전: [Phase 02](phase-02-cdr.md) | 다음: [Phase 04](phase-04-best-effort-udp.md)

## 읽을 규격
- RTPS 2.5: 8.2 (구조), 8.3 (메시지, Receiver 규칙 8.3.4), 9.3 (타입 매핑), 9.4 (와이어 레이아웃), 9.6.2.2 / 9.4.2.11 (ParameterList)

## 구현
- 타입
  - GuidPrefix(12) + EntityId(4), 내장 EntityId 표
  - SequenceNumber (high i32 + low u32)
  - **SequenceNumberSet**: bit 0 = 첫 워드의 MSB. 버그 단골.
  - Locator (kind, port, 16B addr), Time_t (초 + 2^-32 분수)
- 헤더 20B, 서브메시지 헤더 4B (**서브메시지마다 E 플래그로 엔디안 결정**)
- 서브메시지: DATA / HEARTBEAT / ACKNACK / GAP / INFO_TS / INFO_DST / INFO_SRC / PAD, 이후 DATA_FRAG / NACK_FRAG / HEARTBEAT_FRAG, RTPS_HE (2.5)
- Receiver 상태 (sourceGuidPrefix, destGuidPrefix, timestamp ...)

## 규칙 (RTPS 2.5 원문으로 확인함)
- 모르는 서브메시지 ID: 길이로 건너뛴다.
- 모르는 PID: 건너뛴다.
- must-understand 비트(0x4000)가 붙은 모르는 PID: 해당 서브메시지를 무시한다. HeaderExtension 안이면 메시지 전체를 무시한다.
- 0x8000 이상은 벤더 전용 PID다.
- `octetsToInlineQos`는 항상 따른다.

## 확인할 것
- 벤더 ID: Cyclone(Eclipse) 0x0110, Fast DDS(eProsima) 0x010F. OMG 벤더 목록에서 다시 확인한다.

## 완료 기준
- Phase 0 캡처 전부를 파싱 -> 재인코딩해 바이트가 같다.
- 파서 fuzz 1시간 동안 패닉이 없다.

## 진행 기록
- (진행하면서 배운 점, 결정 사항, 막힌 점을 날짜와 함께 적는다)
