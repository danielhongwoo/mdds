# 저자 노트: SPDP (참가자 디스커버리)

상태: 대기

[roadmap](../roadmap.md) | chapter: 9 ([목차](../../book/README.md#목차))

## 읽을 규격
- RTPS 2.5: 8.5.1 ~ 8.5.3, 9.6.2.4, 9.6.3 (내장 엔드포인트 IDL, PID 표), 9.3.2.12 (BuiltinEndpointSet)

## 구현
- ParticipantBuiltinTopicData를 PL_CDR로 인코딩/디코딩
  - PID_PROTOCOL_VERSION 0x15, VENDORID 0x16, PARTICIPANT_GUID 0x50, LEASE_DURATION 0x02
  - METATRAFFIC_* 0x32 / 0x33, DEFAULT_* 0x31 / 0x48, BUILTIN_ENDPOINT_SET 0x58 ...
- 주기 공지, lease 만료, dispose (K 플래그)

## 확인할 것
cyclone-fastdds 캡처에서 나온 관찰과 가설이다. 이 chapter에서 규격으로 확인하고, 맞으면 구현이나 규칙으로 옮기고 지운다.
- Cyclone은 새 참가자를 발견하면 SPDP를 유니캐스트로 1초 간격 4번 보내고, 이후 8초 주기에 맞춘다. Fast DDS는 3초 주기 멀티캐스트마다 유니캐스트도 보낸다 ([벤더 차이](../../captures/cyclone-fastdds/README.md#두-벤더의-차이-지금까지-확인한-것)). 규격이 정한 부분과 벤더 선택을 나눈다.
- Cyclone은 상대가 떠난 뒤 종료할 때 SPDP `DATA(p[UD])`를 보내지 않았다 (rel-fastdds-pub). 상대가 살아 있을 때는 보냈다 (rel-cyclone-pub). 알려진 참가자가 없으면 보내지 않는 것으로 보인다 [추정].
- lease 만료 (lease-expiry 캡처): Cyclone이 SIGKILL로 죽은 뒤 (마지막 패킷 14.26초, 마지막 SPDP 11.22초, lease 10초) Fast DDS는 21.53초까지 유니캐스트 SPDP를 보내고 24.53초부터 멈춘다. Fast DDS 3.6.1은 상대 참가자의 어떤 RTPS 메시지든 받을 때마다 lease 시계를 되돌린다 (`MessageReceiver::processCDRMsg` 끝에서 `assert_remote_participant_liveliness` -> `ParticipantProxyData::last_received_message_tm_` 갱신, `PDP::check_remote_participant_liveliness`가 그 시각 + lease와 비교. 2026-11-03 소스 확인). 규격 문언 (8.5.3.3: 공지로 갱신)보다 넓다. mdds가 어느 쪽을 따를지는 이 chapter에서 정한다.

## 완료 기준
- Cyclone / Fast DDS 쪽 도구와 Wireshark에서 mdds 참가자가 보인다.
- mdds도 상대 참가자 목록을 유지한다.

## 진행 기록
- (날짜, 결정이나 막힌 점 한 줄, 근거 문서 링크. 규칙은 CLAUDE.md "문서 갱신 규칙")
