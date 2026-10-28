# Phase 05 - SPDP (참가자 디스커버리)

상태: 대기

[roadmap](../roadmap.md) | 이전: [Phase 04](phase-04-best-effort-udp.md) | 다음: [Phase 06](phase-06-reliable.md)

## 읽을 규격
- RTPS 2.5: 8.5.1 ~ 8.5.3, 9.6.2.4, 9.6.3 (내장 엔드포인트 IDL, PID 표), 9.3.2.12 (BuiltinEndpointSet)

## 구현
- ParticipantBuiltinTopicData를 PL_CDR로 인코딩/디코딩
  - PID_PROTOCOL_VERSION 0x15, VENDORID 0x16, PARTICIPANT_GUID 0x50, LEASE_DURATION 0x02
  - METATRAFFIC_* 0x32 / 0x33, DEFAULT_* 0x31 / 0x48, BUILTIN_ENDPOINT_SET 0x58 ...
- 주기 공지, lease 만료, dispose (K 플래그)

## 완료 기준
- Cyclone / Fast DDS 쪽 도구와 Wireshark에서 mdds 참가자가 보인다.
- mdds도 상대 참가자 목록을 유지한다.

## 진행 기록
- (날짜, 결정이나 막힌 점 한 줄, 근거 문서 링크. 규칙은 CLAUDE.md "문서 갱신 규칙")
