# 저자 노트: Reliable 프로토콜

상태: 대기

[roadmap](../roadmap.md) | 장: 10 ([목차](../../book/README.md#목차))

## 읽을 규격
- RTPS 2.5: 8.4.9 (StatefulWriter, ReaderProxy), 8.4.10 ~ 8.4.12 (WriterProxy, Reader), 8.4.15 (구현 가이드: Count 단조 증가 등)

## 구현
- ReaderProxy의 change 상태: UNSENT / UNACKNOWLEDGED / REQUESTED / ACKNOWLEDGED
- WriterProxy의 상태: MISSING / RECEIVED / LOST
- HEARTBEAT (F/L 플래그) / ACKNACK / GAP
- 타이머: heartbeatPeriod, nackResponseDelay, heartbeatResponseDelay ...

## 참고
- SEDP 자체가 reliable이라 **디스커버리를 끝내려면 이 장이 먼저 필요**하다.

## 확인할 것
cyclone-fastdds 캡처에서 나온 관찰과 가설이다. 이 장에서 규격으로 확인하고, 맞으면 구현이나 규칙으로 옮기고 지운다.
- HEARTBEAT의 firstSN > lastSN (예: `HB 12-11`, `HB 2-1`): firstSN은 "샘플이 없으면 다음에 쓸 가장 작은 번호"다 (RTPS 2.5 Sec. 8.3.8.6 표). 매칭 전에 쓴 샘플이 VOLATILE reader에게 가지 않는 것 ([rel-cyclone-pub](../../tests/captures/cyclone-fastdds/README.md#rel-cyclone-pub) seq 1-11, rel-fastdds-pub seq 1)과 함께 확인한다.
- HEARTBEAT 전에 나가는 ACKNACK (pre-emptive): Sec. 8.4.2.3은 reader가 writer를 처음 발견했을 때 최적화로 보낼 수 있고, writer는 답하지 않아도 된다고 한다. Cyclone은 Fast DDS의 사용자 reader가 보낸 이 ACKNACK (rel-cyclone-pub 33-46)에 답하지 않았다.
- Fast DDS의 pre-emptive ACKNACK는 bitmapBase가 0이다 (rel-cyclone-pub 33). Sec. 9.4.2.6은 유효한 SequenceNumberSet에 bitmapBase >= 1을 요구한다. mdds writer가 이것을 받으면 어떻게 할지 정한다.
- HEARTBEAT를 DATA에 얹는 빈도: Cyclone은 DATA마다, Fast DDS는 두 번에 한 번. Sec. 8.4.15.4가 허용하는 범위와 mdds의 기본값.

## 완료 기준
- I/O-free core 테스트 하네스에서 무작위 손실/중복/재정렬을 주입해도 순서대로 빠짐없이 전달된다 (property test).

## 진행 기록
- (날짜, 결정이나 막힌 점 한 줄, 근거 문서 링크. 규칙은 CLAUDE.md "문서 갱신 규칙")
