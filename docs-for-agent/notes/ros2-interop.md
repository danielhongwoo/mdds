# 저자 노트: ROS 2 상호운용 (권장)

상태: 대기

[roadmap](../roadmap.md) | 장: 12 ([목차](../../book/README.md#목차))

## 내용
- 11장까지 만든 최소 API만으로 가능한, XTypes 없이 도달할 수 있는 가장 쉬운 실사용 상호운용 목표다.
- ROS 2 메시지는 모두 FINAL 타입이라 XCDR1 CDR_LE로 충분하다.
- 토픽 이름 `rt/<name>`, 타입 이름 `pkg::msg::dds_::Name_` (예: `std_msgs::msg::dds_::String_`)
- ROS 2 기본 QoS: RELIABLE, VOLATILE, KEEP_LAST(10)
- ROS 2 공식 Docker 이미지로 Fast DDS (기본 RMW)와 Cyclone DDS (`RMW_IMPLEMENTATION=rmw_cyclonedds_cpp`) 양쪽을 시험한다.

## 완료 기준
- `ros2 run demo_nodes_cpp talker` -> mdds listener, mdds talker -> `ros2 run demo_nodes_cpp listener`
- RMW를 Fast DDS, Cyclone DDS로 바꿔 각각 확인한다.

## 진행 기록
- (날짜, 결정이나 막힌 점 한 줄, 근거 문서 링크. 규칙은 CLAUDE.md "문서 갱신 규칙")
