# Phase 09 - 키 인스턴스와 수명주기

상태: 대기

[roadmap](../roadmap.md) | 이전: [Phase 08](phase-08-ros2-interop.md) | 다음: [Phase 10](phase-10-dcps-api.md)

## 읽을 규격
- DDS 1.4 Sec. 2.2.2.5.1 (샘플/뷰/인스턴스 상태), Sec. 2.2.2.4.2 (DataWriter: register / unregister / dispose), Sec. 2.2.2.5.3 (DataReader: read / take)
- DDS 1.4 Sec. 2.2.3 중 HISTORY, RESOURCE_LIMITS, DURABILITY
- RTPS 2.5: 9.6.4 (인라인 QoS: KEY_HASH 0x70, STATUS_INFO 0x71), 9.6.4.8 (KeyHash)

## 구현
- 키 인스턴스 관리 (writer, reader 양쪽)
- read / take + SampleInfo (sample_state, view_state, instance_state, generation count)
- dispose / unregister (STATUS_INFO 송수신)
- HISTORY (KEEP_LAST / KEEP_ALL), RESOURCE_LIMITS
- DURABILITY TRANSIENT_LOCAL (늦게 들어온 reader에게 이력 전송)

## KeyHash (RTPS 2.5 Sec. 9.6.4.8에서 확인함)
- 키 멤버만 남긴 FINAL 홀더를 memberId 순으로 **PLAIN_CDR2 빅엔디안**(최대 정렬 4, 패딩 0)으로 직렬화한다.
- 최대 크기가 16B 이하면 0으로 채우고, 그 외에는 MD5를 쓴다.
- 수신 측은 KeyHash를 믿지 말고 페이로드에서 다시 계산하는 편이 안전하다. 구버전 구현과 계산 방식이 다를 수 있다.

## 완료 기준
- ShapesDemo에서 도형을 dispose / 삭제하면 mdds reader의 instance_state가 맞게 바뀐다.
- mdds reader를 늦게 띄워도 Cyclone writer의 TRANSIENT_LOCAL 이력을 받는다. 반대 방향도 확인한다.
- mdds가 계산한 KeyHash가 캡처의 PID_KEY_HASH 값과 같다.

## 진행 기록
- (진행하면서 배운 점, 결정 사항, 막힌 점을 날짜와 함께 적는다)
