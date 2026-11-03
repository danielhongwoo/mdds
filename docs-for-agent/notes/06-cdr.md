# 저자 노트: CDR 직렬화 (XCDR1)

상태: 대기

[roadmap](../roadmap.md) | chapter: 6 ([목차](../../book/README.md#목차))

## 읽을 규격
- CORBA 3.3 Part 2 Sec. 9.3 CDR Transfer Syntax, 특히 Sec. 9.3.1.1 Alignment (Table 9.1)
- XTypes 1.3 Sec. 7.4.1.1 PLAIN_CDR (XCDR1), Sec. 7.4.3.2 스트림 상태와 MAXALIGN
- XTypes 1.3 Sec. 7.6.3.1.2 캡슐화 식별자 (Table 60)
- RTPS 2.5 Ch.10 (직렬화 페이로드 헤더)
- 섹션 지도: [research/xtypes-cdr-idl-security.md](../research/xtypes-cdr-idl-security.md)

## 구현
- 본문: 워크스페이스와 crate 세 개의 역할은 chapter5 1.1이 소개했다. 이 chapter는 `mdds-cdr`를 처음 쓴다.
- `CdrWriter` / `CdrReader`
  - BE/LE
  - 원시 타입 크기 기준 정렬, 기준점은 4바이트 캡슐화 헤더 직후
  - string / sequence / array / enum / union
- 캡슐화 헤더: CDR_BE 0x0000, CDR_LE 0x0001, PL_CDR_BE/LE 0x0002/0x0003, options 하위 2비트 = 끝 패딩 수
- FINAL struct용 derive 매크로
- serde는 쓰지 않는다. serde 데이터 모델로는 정렬, DHEADER, key-only 직렬화를 표현할 수 없다.

## 완료 기준
- ShapeType (dds-rtps `526dc56`의 shape.idl: `@appendable struct ShapeType { @key string<128> color; int32 x; int32 y; int32 shapesize; sequence<uint8> additional_payload_size; }`) 직렬화 결과가 cyclone-fastdds 캡처 바이트와 일치한다. 캡처의 사용자 DATA 끝에 있는 `00 00 00 00`은 다섯 번째 멤버 (길이 0의 sequence)다.

## 진행 기록
- (날짜, 결정이나 막힌 점 한 줄, 근거 문서 링크. 규칙은 CLAUDE.md "문서 갱신 규칙")
