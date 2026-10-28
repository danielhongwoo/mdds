# Phase 07 - SEDP + 매칭 (첫 상호운용)

상태: 대기

[roadmap](../roadmap.md) | 이전: [Phase 06](phase-06-reliable.md) | 다음: [Phase 08](phase-08-ros2-interop.md)

## 읽을 규격
- RTPS 2.5: 8.5.4 ~ 8.5.5, 9.6.3 (DiscoveredWriterData / ReaderData PID)
- DDS 1.4 Sec. 2.2.3 RxO 규칙, PARTITION

## 구현
- publications / subscriptions (+topics) 내장 엔드포인트: transient-local 이력, 새 매칭 시 재전송
- PID: ENDPOINT_GUID 0x5a, TOPIC_NAME 0x05, TYPE_NAME 0x07, RELIABILITY 0x1a, DURABILITY 0x1d ...
- 매칭: 토픽/타입 이름, RxO, 파티션
- 이 단계의 사용자 API는 최소한으로 둔다 (participant 생성, 토픽 하나의 writer / reader, 샘플 송수신). Square는 키가 있는 토픽이지만 이 단계에서는 키를 페이로드에만 싣고 인스턴스 관리는 하지 않는다 (Phase 09). 정식 API는 Phase 10에서 만든다.
- 모르는 벤더 PID와 XTypes PID (예: PID_TYPE_INFORMATION 0x75)를 안전하게 건너뛴다.

## 완료 기준
- **Fast DDS / Cyclone ShapesDemo의 Square 토픽을 mdds가 구독/발행**한다 (best-effort, reliable 둘 다).

## 진행 기록
- (날짜, 결정이나 막힌 점 한 줄, 근거 문서 링크. 규칙은 CLAUDE.md "문서 갱신 규칙")
