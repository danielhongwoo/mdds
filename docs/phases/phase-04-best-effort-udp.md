# Phase 04 - Best-effort 전송 + UDP

상태: 대기

[roadmap](../roadmap.md) | 이전: [Phase 03](phase-03-rtps-messages.md) | 다음: [Phase 05](phase-05-spdp.md)

## 읽을 규격
- RTPS 2.5: 8.4.1 ~ 8.4.8 (행동 모듈 개요, **8.4.2 상호운용 요구사항 먼저**), StatelessWriter/Reader, 9.6.2 (포트, 기본 로케이터)

## 구현
- HistoryCache / CacheChange
- StatelessWriter (ReaderLocator), Best-effort Reader
- UDP 포트 공식: PB=7400, DG=250, PG=2, d0=0, d1=10, d2=1, d3=11
- 멀티캐스트 239.255.0.1 가입, SO_REUSEADDR / SO_REUSEPORT
- participantId 탐색 (0부터 유니캐스트 포트 바인딩 시도)
- 코어는 I/O-free core로 만들고, UDP 드라이버를 바깥에 둔다 ([roadmap](../roadmap.md) 설계 원칙 참고).

## 완료 기준
- 피어 로케이터를 하드코딩한 mdds 프로세스 두 개가 DATA를 주고받는다.

## 진행 기록
- (진행하면서 배운 점, 결정 사항, 막힌 점을 날짜와 함께 적는다)
