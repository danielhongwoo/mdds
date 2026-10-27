# Phase 07 - SEDP + 매칭 (첫 상호운용)

상태: 대기

[roadmap](../roadmap.md) | 이전: [Phase 06](phase-06-reliable.md) | 다음: [Phase 08](phase-08-dcps-api.md)

## 읽을 규격
- RTPS 2.5: 8.5.4 ~ 8.5.5, 9.6.3 (DiscoveredWriterData / ReaderData PID)
- DDS 1.4 Sec. 2.2.3 RxO 규칙, PARTITION

## 구현
- publications / subscriptions (+topics) 내장 엔드포인트: transient-local 이력, 새 매칭 시 재전송
- PID: ENDPOINT_GUID 0x5a, TOPIC_NAME 0x05, TYPE_NAME 0x07, RELIABILITY 0x1a, DURABILITY 0x1d ...
- 매칭: 토픽/타입 이름, RxO, 파티션
- 모르는 벤더 PID와 XTypes PID (예: PID_TYPE_INFORMATION 0x75)를 안전하게 건너뛴다.

## 완료 기준
- **Fast DDS / Cyclone ShapesDemo의 Square 토픽을 mdds가 구독/발행**한다 (best-effort, reliable 둘 다).

## 진행 기록
- (진행하면서 배운 점, 결정 사항, 막힌 점을 날짜와 함께 적는다)
