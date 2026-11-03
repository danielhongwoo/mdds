# 저자 노트: RTPS 메시지 계층

상태: 대기

[roadmap](../roadmap.md) | chapter: 7 ([목차](../../book/README.md#목차))

## 읽을 규격
- RTPS 2.5: 8.2 (구조), 8.3 (메시지, Receiver 규칙 8.3.4), 9.3 (타입 매핑), 9.4 (와이어 레이아웃), 9.6.2.2 / 9.4.2.11 (ParameterList)

## 구현
- 본문: `mdds-rtps` crate를 소개한다. 소켓, 시계, 스레드를 쓰지 않는 I/O-free core라는 것까지만 말하고, 드라이버와 나누는 이유는 8에서.
- 타입
  - GuidPrefix(12) + EntityId(4), 내장 EntityId 표
  - SequenceNumber (high i32 + low u32)
  - **SequenceNumberSet**: bit 0 = 첫 워드의 MSB. 버그 단골.
  - Locator (kind, port, 16B addr), Time_t (초 + 2^-32 분수)
  - 와이어의 Duration_t (초 + 2^-32 분수, 무한대 {0x7fffffff, 0xffffffff})는 chapter5의 `qos::Duration` (DDS 의미, 무한대 변이)과 다른 타입이다. 같은 crate에 둘이 있게 되므로 와이어 쪽 이름을 다르게 짓는다 (예: `DurationT`). 변환은 chapter11.
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
- Fast DDS는 사용자 DATA 앞에 INFO_DST를 붙이고 Cyclone은 붙이지 않는다 ([reliable-fastdds-pub](../../captures/cyclone-fastdds/README.md#reliable-fastdds-pub)). 수신 측에서 INFO_DST가 바꾸는 상태 (destGuidPrefix)와, 없을 때의 기본값을 확인한다.
- mdds-cdr 후보 기능 (chapter6 검토에서 나옴): ParameterList 값마다 새 `CdrReader`를 만들면 오류 `offset`이 그 값 슬라이스 기준이 된다. 페이로드 기준 위치가 필요하면 기준 오프셋을 받는 생성자 (예: `with_base`)를 둔다. octet 배열 (GuidPrefix 12, EntityId 4, Locator 주소 16)은 `read_bytes(N)?.try_into()`를 감싼 `read_array::<N>()`이 있으면 편하다.

## 완료 기준
- cyclone-fastdds 캡처 전부를 파싱 -> 재인코딩해 바이트가 같다.
- 파서 fuzz 1시간 동안 패닉이 없다.
- 캡처의 사용자 DATA `serializedData`를 mdds-cdr의 `from_payload`로 역직렬화해 chapter6 Lab 6-3의 값과 같다 (end-to-end 테스트. Lab 6-3의 hex 리터럴은 바꾸지 않는다).

## 진행 기록
- (날짜, 결정이나 막힌 점 한 줄, 근거 문서 링크. 규칙은 CLAUDE.md "문서 갱신 규칙")
