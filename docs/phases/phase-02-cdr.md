# Phase 02 - CDR 직렬화 (XCDR1)

상태: 대기

[roadmap](../roadmap.md) | 이전: [Phase 01](phase-01-dcps-concepts.md) | 다음: [Phase 03](phase-03-rtps-messages.md)

## 읽을 규격
- CORBA CDR 정렬 규칙 (CORBA 3.x Part 2 Sec. 9.3)
- XTypes 1.3 Sec. 7.4 (XCDR1 부분)
- RTPS 2.5 Ch.10 (직렬화 페이로드 헤더)

## 구현
- `CdrWriter` / `CdrReader`
  - BE/LE
  - 원시 타입 크기 기준 정렬, 기준점은 4바이트 캡슐화 헤더 직후
  - string / sequence / array / enum / union
- 캡슐화 헤더: CDR_BE 0x0000, CDR_LE 0x0001, PL_CDR_BE/LE 0x0002/0x0003, options 하위 2비트 = 끝 패딩 수
- FINAL struct용 derive 매크로
- serde는 쓰지 않는다. serde 데이터 모델로는 정렬, DHEADER, key-only 직렬화를 표현할 수 없다.

## 완료 기준
- ShapeType (`@key string color; long x; long y; long shapesize;`) 직렬화 결과가 Phase 0 캡처 바이트와 일치한다.

## 진행 기록
- (진행하면서 배운 점, 결정 사항, 막힌 점을 날짜와 함께 적는다)
