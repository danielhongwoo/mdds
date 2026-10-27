# Phase 12 - IDL 4.2 컴파일러

상태: 대기

[roadmap](../roadmap.md) | 이전: [Phase 11](phase-11-xtypes.md) | 다음: [Phase 13](phase-13-security.md)

## 읽을 규격
- IDL 4.2 Sec. 7.4 (빌딩 블록: Core Data Types, Extended Data Types, Anonymous Types, Annotations, Preprocessing. 인터페이스/CORBA 블록은 생략)
- IDL 4.2 Sec. 8 (표준 어노테이션), `#pragma keylist`

## 구현
- 전처리 (#include) -> 파서 (pest / nom / chumsky) -> 스코프 이름 해석 -> Rust 코드 + TypeObject 생성 (`build.rs` 연동)

## 확인할 것
- OMG "IDL4 -> Rust 언어 매핑" RFP (mars/2026-3-6) 진행 상황. 공개된 제안서가 있으면 Rust 타입 매핑을 맞춘다 ([roadmap](../roadmap.md) 참고).

## 완료 기준
- ShapeType.idl과 ROS 2 IDL 몇 개로 생성한 코드가 derive 매크로 결과와 같은 바이트를 낸다.

## 진행 기록
- (진행하면서 배운 점, 결정 사항, 막힌 점을 날짜와 함께 적는다)
