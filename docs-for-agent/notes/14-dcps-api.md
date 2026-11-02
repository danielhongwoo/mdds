# 저자 노트: DCPS API 완성 + OMG 상호운용 테스트

상태: 대기

[roadmap](../roadmap.md) | chapter: 14 ([목차](../../book/README.md#목차))

## 읽을 규격
- DDS 1.4 Sec. 2.2.2 (엔티티 계층 전체: Factory, DomainParticipant, Publisher / Subscriber, Topic)
- DDS 1.4 Sec. 2.2.4 (Listener / Condition / WaitSet, 통신 상태), Sec. 2.2.5 (내장 토픽)
- DDS 1.4 Sec. 2.2.3 중 DEADLINE, LIFESPAN, LIVELINESS, OWNERSHIP
- RTPS 2.5: 8.4.13 (Liveliness, ParticipantMessage 엔드포인트)

## 구현
- 엔티티 계층과 생성/삭제 규칙, QoS 설정과 변경 가능 여부 검사 (변경 가능 열은 chapter 5의 3.4절 표). 한 엔티티 안의 정책 일관성 (DEADLINE >= TIME_BASED_FILTER 등, INCONSISTENT_POLICY)
- API 쪽 QoS 묶음 (DataWriterQos, DataReaderQos, PublisherQos, SubscriberQos, TopicQos)을 여기서 만들고, 매칭용 `mdds_rtps::qos::PublicationQos` / `SubscriptionQos` (chapter 5)로 바꾸는 함수를 둔다. chapter 11의 최소 API는 매칭용 타입을 그대로 받는다.
- 파티션 와일드카드 `[...]` (chapter 5의 `partitions_match`는 `*`, `?`만 해석하고 `[`는 문자 그대로 비교한다). fnmatch의 `[!...]`, 범위, 이스케이프까지 두 벤더와 맞춘다.
- Status (PUBLICATION_MATCHED, REQUESTED_INCOMPATIBLE_QOS 등), Listener, StatusCondition / ReadCondition / GuardCondition, WaitSet
- DEADLINE, LIFESPAN, LIVELINESS (AUTOMATIC, MANUAL_BY_PARTICIPANT)
- 내장 토픽 (DCPSParticipant, DCPSPublication, DCPSSubscription)
- `tools/shape_main` (OMG 상호운용 테스트 규약에 맞춘 CLI) - dds-rtps 커밋 526dc56e에 Rust 구현용 shape_main이 있다 (`srcRs/DustDDS`, `srcRs/HDDS`, `RustDDS/`). 옵션 처리와 출력 문자열의 참고 코드로 본다

## 완료 기준
- `tools/shape_main`이 `omg-dds/dds-rtps`의 `interoperability_report.py`로 주요 시나리오를 Cyclone / Fast DDS와 양방향 통과한다.

## 진행 기록
- (날짜, 결정이나 막힌 점 한 줄, 근거 문서 링크. 규칙은 CLAUDE.md "문서 갱신 규칙")
