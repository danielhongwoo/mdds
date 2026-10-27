# Phase 08 - DCPS API 완성 + 인스턴스 수명주기

상태: 대기

[roadmap](../roadmap.md) | 이전: [Phase 07](phase-07-sedp-matching.md) | 다음: [Phase 09](phase-09-fragmentation-perf.md)

## 읽을 규격
- DDS 1.4 Sec. 2.2.2.4 ~ 2.2.2.5, Sec. 2.2.4 (Listener / Condition / WaitSet), Sec. 2.2.5 (내장 토픽)
- RTPS 2.5: 9.6.4 (인라인 QoS: KEY_HASH 0x70, STATUS_INFO 0x71), 8.4.13 (Liveliness)

## 구현
- 엔티티 계층, read/take + SampleInfo
- 키 인스턴스, dispose / unregister
- HISTORY / RESOURCE_LIMITS, TRANSIENT_LOCAL
- DEADLINE / LIFESPAN / LIVELINESS
- Status / Listener / WaitSet

## KeyHash (RTPS 2.5 Sec. 9.6.4.8에서 확인함)
- 키 멤버만 남긴 FINAL 홀더를 memberId 순으로 **PLAIN_CDR2 빅엔디안**(최대 정렬 4, 패딩 0)으로 직렬화한다.
- 최대 크기가 16B 이하면 0으로 채우고, 그 외에는 MD5를 쓴다.
- 수신 측은 KeyHash를 믿지 말고 페이로드에서 다시 계산하는 편이 안전하다. 구버전 구현과 계산 방식이 다를 수 있다.

## 완료 기준
- `tools/shape_main`이 `omg-dds/dds-rtps`의 `interoperability_report.py`로 주요 시나리오를 Cyclone / Fast DDS와 양방향 통과한다.

## 진행 기록
- (진행하면서 배운 점, 결정 사항, 막힌 점을 날짜와 함께 적는다)
