# 저자 노트: IDL 4.2 컴파일러

상태: 대기

[roadmap](../roadmap.md) | chapter: 17 ([목차](../../book/README.md#목차))

## 읽을 규격
- IDL 4.2 (섹션은 원문 확인함)
  - Sec. 7.2 어휘 규칙, Sec. 7.3 전처리, Sec. 7.5 이름과 스코프
  - Sec. 7.4.1 Core Data Types, Sec. 7.4.13 Extended Data-Types, Sec. 7.4.14 Anonymous Types, Sec. 7.4.15 Annotations. 인터페이스 / CORBA 블록 (7.4.3 등)은 생략
  - Sec. 8.3.1 ~ 8.3.2 표준 어노테이션 (@id, @autoid, @optional, @extensibility, @key 등)
- XTypes 1.3 Sec. 7.3.1.2.1 (DDS 전용 어노테이션은 IDL이 아니라 XTypes에 있다)
- `#pragma keylist`는 어느 OMG 규격에도 없는 벤더 관례다. 호환을 위해 지원할지는 이 chapter에서 정한다.

## 구현
- 전처리 (#include) -> 파서 (pest / nom / chumsky) -> 스코프 이름 해석 -> Rust 코드 + TypeObject 생성 (`build.rs` 연동)

## 확인할 것
- OMG "IDL4 -> Rust 언어 매핑" RFP (mars/2026-3-6) 진행 상황. 공개된 제안서가 있으면 Rust 타입 매핑을 맞춘다 ([roadmap](../roadmap.md) 참고).

## 완료 기준
- ShapeType.idl과 ROS 2 IDL 몇 개로 생성한 코드가 derive 매크로 결과와 같은 바이트를 낸다.

## 진행 기록
- (날짜, 결정이나 막힌 점 한 줄, 근거 문서 링크. 규칙은 CLAUDE.md "문서 갱신 규칙")
