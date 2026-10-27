# Phase 11 - DDS-XTypes

상태: 대기

[roadmap](../roadmap.md) | 이전: [Phase 10](phase-10-ros2-interop.md) | 다음: [Phase 12](phase-12-idl-compiler.md)

## 읽을 규격
- XTypes 1.3 Sec. 7.2 (타입 시스템, 확장성), Sec. 7.3 (어노테이션, TypeObject - Annex B IDL), Sec. 7.4.3 (XCDR2), Sec. 7.6 (디스커버리 연동, TypeLookup), Sec. 7.2.4 (할당 가능성), Sec. 7.5 (Dynamic API)

## 구현 순서
1. PLAIN_CDR2 (최대 정렬 4)
2. DELIMITED_CDR2 (DHEADER, appendable)
3. PL_CDR2 (EMHEADER: M비트 + LC 3비트 + 28비트 memberId, mutable)
4. @optional
5. XCDR1 PL_CDR mutable (PID_EXTENDED 0x3F01 / LIST_END 0x3F02)
6. TypeObject (minimal/complete, MD5 앞 14바이트 해시)
7. PID_TYPE_INFORMATION
8. TypeLookup 서비스
9. 할당 가능성, TypeConsistencyEnforcement
10. DynamicType / DynamicData

## 주의
- memberId 기본값은 `@autoid(SEQUENTIAL)`(선언 순서)다. 이름 해시는 `@autoid(HASH)` / `@hashid`일 때만 쓴다.

## 확인할 것
- Cyclone / Fast DDS가 기본으로 고르는 데이터 표현(XCDR1/2) 규칙
- XTypes 1.4 beta 진행 상황

## 완료 기준
- `shape_main -x 2` (XCDR2) 상호운용
- appendable/mutable 타입의 버전 차이가 있어도 통신된다.

## 진행 기록
- (진행하면서 배운 점, 결정 사항, 막힌 점을 날짜와 함께 적는다)
