# 저자 노트: key 인스턴스와 수명주기

상태: 대기

[roadmap](../roadmap.md) | chapter: 13 ([목차](../../book/README.md#목차))

## 읽을 규격
- DDS 1.4 Sec. 2.2.2.5.1 (샘플/뷰/인스턴스 상태), Sec. 2.2.2.4.2 (DataWriter: register / unregister / dispose), Sec. 2.2.2.5.3 (DataReader: read / take)
- DDS 1.4 Sec. 2.2.3 중 HISTORY, RESOURCE_LIMITS, DURABILITY
- RTPS 2.5: 9.6.4 (인라인 QoS: KEY_HASH 0x70, STATUS_INFO 0x71), 9.6.4.8 (KeyHash)

## 구현
- key 인스턴스 관리 (writer, reader 양쪽)
- read / take + SampleInfo (sample_state, view_state, instance_state, generation count)
- dispose / unregister (STATUS_INFO 송수신)
- HISTORY (KEEP_LAST / KEEP_ALL), RESOURCE_LIMITS
- DURABILITY TRANSIENT_LOCAL (늦게 들어온 reader에게 이력 전송)

## KeyHash (RTPS 2.5 Sec. 9.6.4.8에서 확인함)
- key 멤버만 남긴 FINAL 홀더를 memberId 순으로 **PLAIN_CDR2 빅엔디안**(최대 정렬 4, 패딩 0)으로 직렬화한다.
- 최대 크기가 16B 이하면 0으로 채우고, 그 외에는 MD5를 쓴다.
- 수신 측은 KeyHash를 믿지 말고 페이로드에서 다시 계산하는 편이 안전하다. 구버전 구현과 계산 방식이 다를 수 있다.

## 확인할 것
cyclone-fastdds 캡처에서 나온 관찰과 가설이다. 이 chapter에서 규격으로 확인하고, 맞으면 구현이나 규칙으로 옮기고 지운다.
- 구독 로그의 샘플 수가 전달된 DATA 수보다 적다 (rel-cyclone-pub 84 / 89). shape_main 구독자의 history (KEEP_LAST)와 읽기 주기 때문으로 보인다 [추정] ([rel-cyclone-pub](../../captures/cyclone-fastdds/README.md#rel-cyclone-pub)).
- dispose-on-exit 캡처: 마지막 사용자 DATA에 `PID_STATUS_INFO` Disposed와 key만 실린다. 구독 로그 끝은 `NOT_ALIVE_DISPOSED_INSTANCE_STATE`.

## 완료 기준
- ShapesDemo에서 도형을 dispose / 삭제하면 mdds reader의 instance_state가 맞게 바뀐다.
- mdds reader를 늦게 띄워도 Cyclone writer의 TRANSIENT_LOCAL 이력을 받는다. 반대 방향도 확인한다.
- mdds가 계산한 KeyHash가 캡처의 PID_KEY_HASH 값과 같다.

## 진행 기록
- (날짜, 결정이나 막힌 점 한 줄, 근거 문서 링크. 규칙은 CLAUDE.md "문서 갱신 규칙")
