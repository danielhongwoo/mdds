# Phase 10 - ROS 2 상호운용 (선택)

상태: 대기

[roadmap](../roadmap.md) | 이전: [Phase 09](phase-09-fragmentation-perf.md) | 다음: [Phase 11](phase-11-xtypes.md)

## 내용
- ROS 2 메시지는 모두 FINAL 타입이라 XCDR1 CDR_LE로 충분하다.
- 토픽 이름 `rt/<name>`, 타입 이름 `pkg::msg::dds_::Name_`

## 완료 기준
- `ros2 run demo_nodes_cpp talker` <-> mdds listener

## 진행 기록
- (진행하면서 배운 점, 결정 사항, 막힌 점을 날짜와 함께 적는다)
