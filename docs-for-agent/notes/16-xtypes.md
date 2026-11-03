# 저자 노트: DDS-XTypes

상태: 대기

[roadmap](../roadmap.md) | chapter: 16 ([목차](../../book/README.md#목차))

## 읽을 규격
- XTypes 1.3 (섹션은 원문 확인함, 상세 지도: [research/xtypes-cdr-idl-security.md](../research/xtypes-cdr-idl-security.md))
  - Sec. 7.2.2 타입 시스템 모델, Sec. 7.2.3 확장성 (Table 12)
  - Sec. 7.3.1.2.1 내장 어노테이션, Sec. 7.3.4 TypeIdentifier / TypeObject (IDL은 Annex B)
  - Sec. 7.4.1.2 PL_CDR (XCDR1 mutable), Sec. 7.4.2 XCDR2, Sec. 7.4.3 형식 선택과 DHEADER / EMHEADER, Sec. 7.4.3.5.3 타입별 직렬화 규칙
  - Sec. 7.6.3 디스커버리 연동 (TypeInformation 7.6.3.2, TypeLookup 7.6.3.3, TypeConsistencyEnforcement 7.6.3.4)
  - Sec. 7.2.4 할당 가능성, Sec. 7.5.2 Dynamic API

## 구현 순서
1. PLAIN_CDR2 (최대 정렬 4)
2. DELIMITED_CDR (DHEADER, appendable. 캡슐화 식별자 이름은 D_CDR2)
3. PL_CDR2 (EMHEADER: M비트 + LC 3비트 + 28비트 memberId, mutable. LC 5 ~ 7의 NEXTINT 재사용 규칙 주의)
4. @optional
5. XCDR1 PL_CDR mutable (PID_EXTENDED 0x3F01 / LIST_END 0x3F02)
6. TypeObject (minimal/complete, MD5 앞 14바이트 해시)
7. PID_TYPE_INFORMATION
8. TypeLookup 서비스
9. 할당 가능성, TypeConsistencyEnforcement
10. DynamicType / DynamicData

## 주의
- memberId 기본값은 `@autoid(SEQUENTIAL)`(선언 순서)다 (Sec. 7.3.1.2.1.1에서 확인). 이름 해시는 `@autoid(HASH)` / `@hashid`일 때만 쓴다.

## 확인할 것
- TypeObject 계열 타입 (수십 개, union과 appendable / mutable이 섞임)의 `CdrSerialize` 구현을 어떻게 만들지. chapter6은 손으로 썼다 (노트 6 결정). 후보: 의존 없는 `macro_rules!`, Annex B IDL을 chapter17에서 생성. proc-macro (`syn`, `quote`)는 독자 부담 때문에 마지막 후보.
- XCDR1 appendable 타입에 멤버를 덧붙인 새 버전의 샘플을 옛 reader가 받을 수 있는지 (chapter6의 `from_payload`가 남은 바이트를 오류로 보지 않는 근거). 할당 가능성 (Sec. 7.2.4)으로 확인.
- 스트림 origin 표기: Sec. 7.4.3.5.3 규칙 (1)의 `PUSH(ORIGIN = 0)`이 options 앞에 있어 글자 그대로면 헤더 앞이 기준이다. chapter6은 RTPS 2.5 Sec. 10.2 (헤더 뒤)를 따랐고 두 벤더도 같다. XCDR2 (상한 4)에서는 차이가 없다.
- DHEADER는 앞자리를 고쳐 쓰지 않아도 된다. V2의 정렬 상한이 4이고 DHEADER가 4의 배수 위치에 놓이므로, 내용을 기준 0인 새 writer로 쓴 뒤 길이를 앞에 붙여도 같은 바이트다. 고쳐 쓰기 (backpatch)는 복사를 줄이는 최적화다. chapter6의 "부분 슬라이스 위의 새 reader / writer" 방식이 그대로 통한다.
- Cyclone / Fast DDS가 기본으로 고르는 데이터 표현(XCDR1/2) 규칙. 캡처로 확인한 것 (chapter4): Fast DDS는 SEDP에 PID_DATA_REPRESENTATION을 늘 싣고, Cyclone은 기본값 (XCDR)이면 생략하고 `-x 2`일 때만 싣는다 (XTypes 1.3 Sec. 7.6.3.1.1: 비어 있으면 XCDR).
- RepresentationIdentifier 값의 불일치. RTPS 2.5 표 10.3은 CDR2_LE {0x00, 0x11}, D_CDR_LE {0x00, 0x15}를 적고 (열 머리는 XTypes 표 60을 가리킨다), XTypes 1.3의 7.4.3.4 표 39도 0x10-0x15를 쓰는데, XTypes 1.3의 7.6.3.1.2 표 60은 CDR2 0x06 / 0x07, D_CDR2 0x08 / 0x09, PL_CDR2 0x0a / 0x0b를 적는다. 캡처의 Cyclone (`-x 2`)과 Wireshark 4.6.9는 표 60의 D_CDR2_LE 0x0009를 쓴다. 7.6.3.1.2는 RTPS DATA의 식별자를 표 60대로 두라고 쓰므로 와이어에서는 표 60이 규범으로 보인다 [추정]. OMG issue 번호는 확인하지 못했다. mdds는 표 60을 따른다.
- Wireshark 4.6.9는 Cyclone의 SEDP writer 정보를 PID_TYPE_INFORMATION (0x0075)에서 Malformed로 표시한다. Cyclone의 TypeInformation 바이트 (앞부분 DHEADER 0x60, EMHEADER 0x40001001)가 XTypes 1.3에 맞는지 확인한다. PID_TYPE_INFORMATION은 @optional이고 (XTypes 1.3 Sec. 7.6.3.2.2), 없으면 타입 이름으로만 매칭한다. Fast DDS 3.6.1은 싣지 않는다.
- XTypes 1.4 beta 진행 상황
- writer만 XCDR2 (`-x 2`), reader는 XCDR1만 받을 때 (chapter5의 5.1.5 확인 질문): `check_rxo`는 DATA_REPRESENTATION 불일치. 두 벤더가 같은 판정을 내리는지 상호운용으로 확인. TYPE_CONSISTENCY_ENFORCEMENT (QosPolicyId 24)가 타입 검사 방식을 정하므로 (XTypes 1.3 Sec. 7.6.3.4.2) 매칭 함수 (chapter11)에 타입 검사가 들어갈 때 함께 본다.

## 완료 기준
- `shape_main -x 2` (XCDR2) 상호운용
- appendable/mutable 타입의 버전 차이가 있어도 통신된다.

## 진행 기록
- (날짜, 결정이나 막힌 점 한 줄, 근거 문서 링크. 규칙은 CLAUDE.md "문서 갱신 규칙")
