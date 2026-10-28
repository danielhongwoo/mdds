# Phase 06 - Reliable 프로토콜

상태: 대기

[roadmap](../roadmap.md) | 이전: [Phase 05](phase-05-spdp.md) | 다음: [Phase 07](phase-07-sedp-matching.md)

## 읽을 규격
- RTPS 2.5: 8.4.9 (StatefulWriter, ReaderProxy), 8.4.10 ~ 8.4.12 (WriterProxy, Reader), 8.4.15 (구현 가이드: Count 단조 증가 등)

## 구현
- ReaderProxy의 change 상태: UNSENT / UNACKNOWLEDGED / REQUESTED / ACKNOWLEDGED
- WriterProxy의 상태: MISSING / RECEIVED / LOST
- HEARTBEAT (F/L 플래그) / ACKNACK / GAP
- 타이머: heartbeatPeriod, nackResponseDelay, heartbeatResponseDelay ...

## 참고
- SEDP 자체가 reliable이라 **디스커버리를 끝내려면 이 단계가 먼저 필요**하다.

## 완료 기준
- I/O-free core 테스트 하네스에서 무작위 손실/중복/재정렬을 주입해도 순서대로 빠짐없이 전달된다 (property test).

## 진행 기록
- (날짜, 결정이나 막힌 점 한 줄, 근거 문서 링크. 규칙은 CLAUDE.md "문서 갱신 규칙")
