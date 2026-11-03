# 저자 노트: SEDP + 매칭 (첫 상호운용)

상태: 대기

[roadmap](../roadmap.md) | chapter: 11 ([목차](../../book/README.md#목차))

## 읽을 규격
- RTPS 2.5: 8.5.4 ~ 8.5.5, 9.6.3 (DiscoveredWriterData / ReaderData PID)
- DDS 1.4 Sec. 2.2.3 RxO 규칙과 PARTITION은 chapter 5에서 읽고 `mdds_rtps::qos`로 구현했다 (`check_rxo`, `partitions_match`). 여기서는 그것을 쓴다.

## 구현
- publications / subscriptions (+topics) 내장 엔드포인트: transient-local 이력, 새 매칭 시 재전송
- PID: ENDPOINT_GUID 0x5a, TOPIC_NAME 0x05, TYPE_NAME 0x07, RELIABILITY 0x1a, DURABILITY 0x1d ...
- 매칭: 토픽/타입 이름, RxO, 파티션. 토픽 이름 / 타입 / 파티션 / RxO를 묶는 함수와 결과 타입 (어느 조건이 어긋났는지)은 여기서 만든다 (chapter 5는 `check_rxo`와 `partitions_match`만 둔다). 검사 순서는 규격이 정하지 않았고 두 벤더가 다르다: Cyclone (`ddsi_qos_match_mask_p`)은 파티션을 먼저 보고 어긋나면 RxO를 보고하지 않으며, Fast DDS (`EDP::valid_matching`)는 RxO를 먼저 모아 incompatible로 보고한 뒤 파티션을 본다. 또 Fast DDS는 PRESENTATION, LATENCY_BUDGET, DESTINATION_ORDER를 검사하지 않는다. 어느 쪽에 맞출지 여기서 정한다 ([노트 5](05-dcps-concepts.md#확인할-것)).
- PID -> QoS 변환: `PublicationQos`와 `SubscriptionQos`는 필드가 같으므로 (chapter 5) 공통 필드의 파싱을 내부 struct나 macro로 한 번만 쓴다. 없는 PID는 각 타입의 `Default` (엔티티별 기본값, RTPS 2.5 Sec. 8.5.4.4). kind의 와이어 값 (BEST_EFFORT = 1, RELIABLE = 2)과 `qos::Duration` <-> 와이어 Duration_t 변환도 여기서.
- 이 chapter의 사용자 API는 최소한으로 둔다. QoS는 chapter 5의 `PublicationQos` / `SubscriptionQos`를 그대로 받는다 (API 쪽 QoS 묶음은 chapter 14) (participant 생성, 토픽 하나의 writer / reader, 샘플 송수신). Square는 키가 있는 토픽이지만 이 chapter에서는 키를 페이로드에만 싣고 인스턴스 관리는 하지 않는다 (chapter 13). 정식 API는 chapter 14에서 만든다.
- 모르는 벤더 PID와 XTypes PID (예: PID_TYPE_INFORMATION 0x75)를 안전하게 건너뛴다.

## 확인할 것
cyclone-fastdds 캡처에서 나온 관찰과 가설이다. 이 chapter에서 규격으로 확인하고, 맞으면 구현이나 규칙으로 옮기고 지운다.
- 매칭은 양쪽이 상대의 SEDP 데이터를 받는 시점에 각자 하는 것으로 보인다. reliable-cyclone-pub에서는 Fast DDS 쪽 (22)과 Cyclone 쪽 (47)이 1.1초 차이가 났고, reliable-fastdds-pub에서는 10ms였다 ([reliable-cyclone-pub](../../captures/cyclone-fastdds/README.md#reliable-cyclone-pub)).
- reliable-cyclone-pub에서 Cyclone은 Fast DDS의 첫 SEDP HEARTBEAT (21)에 답하지 않고 1초 뒤의 HB (42)에 답했다. lease-expiry에서도 같았고, Cyclone이 발행한 나머지 4개 캡처에서는 첫 HB에 바로 답했다. 21이 SPDP (18) 바로 뒤에 도착해 Cyclone이 아직 상대 참가자를 등록하기 전이었던 것으로 보인다 [추정]. mdds는 같은 상황에서 어떻게 할지 정한다.
- 발행자 종료 순서: SEDP `DATA(w[UD])` (status_info Unregistered + Disposed) 다음에 SPDP `DATA(p[UD])`. 구독자의 `NOT_ALIVE_NO_WRITERS`가 어느 메시지로 정해지는지.

## 완료 기준
- **Fast DDS / Cyclone ShapesDemo의 Square 토픽을 mdds가 구독/발행**한다 (best-effort, reliable 둘 다).

## 진행 기록
- (날짜, 결정이나 막힌 점 한 줄, 근거 문서 링크. 규칙은 CLAUDE.md "문서 갱신 규칙")
