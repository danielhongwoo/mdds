# 저자 노트: DCPS 개념 모델

상태: 대기

[roadmap](../roadmap.md) | chapter: 5 ([목차](../../book/README.md#목차))

## 읽을 규격
- DDS 1.4 Sec. 2.2.1 (개념 모델), Sec. 2.2.2 (엔티티), Sec. 2.2.2.5.1 (샘플/뷰/인스턴스 상태), Sec. 2.2.3 (QoS 22종), Annex A (적합성 프로파일)

## 구현
- QoS 정책 Rust 타입
- **RxO 호환성 검사 함수** (순수 함수, 단위 테스트)
- 엔티티 관계는 문서로 정리

## 목표 범위 결정
- **Minimum 프로파일 + RELIABILITY, DURABILITY(VOLATILE/TRANSIENT_LOCAL), HISTORY, PARTITION**
- DDS 1.4 적합성 프로파일은 Minimum / Content-subscription / Persistence / Ownership / Group access 다섯 개다. Object model 프로파일은 DLRL과 함께 빠졌다.

## 완료 기준
- QoS 표 (적용 엔티티 / RxO / 변경 가능 / 와이어 전송 여부)를 스스로 설명할 수 있다.
- 호환성 테스트가 통과한다.

## 진행 기록
- (날짜, 결정이나 막힌 점 한 줄, 근거 문서 링크. 규칙은 CLAUDE.md "문서 갱신 규칙")
