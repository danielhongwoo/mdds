# 조사 노트: CDR, XTypes, IDL, DDS-Security

[roadmap](../roadmap.md)

- 조사일: 2026-10-27, 원문 검증: 2026-10-28
- 출처 (모두 `omg-specs/`에 PDF와 텍스트 추출본이 있다)
  - DDS-XTypes 1.3 (formal/2020-02-04): `omg-specs/dds-xtypes-1.3.pdf`
  - CORBA 3.3 Part 2 (formal/2012-11-14): `omg-specs/corba-3.3-part2.pdf`. OMG의 3.4 URL에서 받았지만 표지는 3.3이다. CDR 장 번호는 같다.
  - IDL 4.2 (formal/18-01-05): `omg-specs/idl-4.2.pdf`
  - DDS-Security 1.1 (formal/2018-04-01), 1.2 (formal/25-03-06): `omg-specs/dds-security-1.1.pdf`, `omg-specs/dds-security-1.2.pdf`
- 이 노트의 섹션 번호는 모두 원문에서 확인했다. 쪽 번호는 규격에 인쇄된 번호다.

## 1. CORBA CDR (CORBA 3.3 Part 2)
- Sec. 9.3 CDR Transfer Syntax (71쪽)
- Sec. 9.3.1.1 Alignment (72쪽, Table 9.1): 원시 타입은 자기 크기 (1, 2, 4, 8)로 정렬한다. 기준은 스트림 시작이고, 중첩된 encapsulation은 0부터 다시 센다.
- Sec. 9.3.1 원시 타입, Sec. 9.3.2 구성 타입, Sec. 9.3.3 Encapsulation (79쪽)
- enum은 4바이트, wchar는 GIOP 1.2 이후 1바이트 정렬

## 2. DDS-XTypes 1.3

### 섹션 지도

| 내용 | 섹션 |
|---|---|
| 타입 시스템 모델, 타입 종류 | 7.2.2 (23쪽). 원시 타입 7.2.2.2, 구성 타입 7.2.2.4 |
| 확장성 (FINAL / APPENDABLE / MUTABLE) | **7.2.3** Type Extensibility and Mutability (57쪽, Table 12). 1.1에서 APPENDABLE의 이름은 EXTENSIBLE이었다 |
| 멤버 ID 범위 (28비트), 이름 해시 | 7.2.2.4.4.4.4, 7.2.2.4.4.4.5 (NameHash = UTF-8 이름 MD5의 앞 4바이트) |
| must-understand, optional, key 멤버 | 7.2.2.4.4.4.6 ~ 7.2.2.4.4.4.8 |
| 타입 호환성, 할당 가능성 | 7.2.4 (58쪽), 7.2.4.4 Assignability Rules (60쪽), 집합 타입 규칙 7.2.4.4.8 |
| 내장 어노테이션 | **7.3.1.2.1** Built-in Annotations (75쪽). 멤버 ID 7.3.1.2.1.1, 확장성 7.3.1.2.1.8 |
| TypeIdentifier / TypeObject | 7.3.4 (107쪽). 직렬화 제약 7.3.4.5, 동등성 7.3.4.7, 해시 알고리즘 7.3.4.9.2. IDL은 Annex B (261쪽) |
| 데이터 표현 전체 | 7.4 (118쪽) |
| XCDR1: PLAIN_CDR | 7.4.1.1 (120쪽). 문자 7.4.1.1.2, enum / bitmask 7.4.1.1.3, map 7.4.1.1.4, struct (상속, optional) 7.4.1.1.5 |
| XCDR1: PL_CDR | 7.4.1.2 (124쪽). PID 값 해석 7.4.1.2.1 (Table 34), 멤버 ID -> PID 매핑 7.4.1.2.2 ~ 7.4.1.2.4 |
| XCDR2: PLAIN_CDR2, DELIMITED_CDR, PL_CDR2 | 7.4.2 (128쪽) |
| 확장성과 버전별 형식 | 7.4.3.1 (Table 35) |
| 스트림 상태, MAXALIGN | 7.4.3.2 ~ 7.4.3.3 (Table 36, 37) |
| DHEADER | 7.4.3.4.1 (135쪽) |
| EMHEADER, LC, NEXTINT | 7.4.3.4.2 (136쪽) |
| 타입별 직렬화 규칙 전체 | 7.4.3.5.3 (140쪽 부근부터) |
| Dynamic API | 7.5.2 Dynamic Language Binding. DynamicType 7.5.2.8, DynamicTypeBuilder 7.5.2.9 |
| DDS에서 타입 시스템 사용 | 7.6 (209쪽) |
| DataRepresentation QoS | 7.6.3.1 |
| 캡슐화 식별자 | 7.6.3.1.2 (Table 60, 215 ~ 216쪽) |
| TypeInformation, 디스커버리 추가 멤버 | 7.6.3.2.1, 7.6.3.2.2 |
| TypeLookup 서비스 | 7.6.3.3 (220쪽) |
| TypeConsistencyEnforcement QoS | 7.6.3.4 |
| 키 토픽 상호운용 (KeyHash) | 7.6.8 (237쪽) |
| 내장 토픽 데이터 타입 | Annex D |

### 형식 선택 (7.4.3.1, Table 35)

| 확장성 | XCDR1 | XCDR2 |
|---|---|---|
| FINAL | PLAIN_CDR | PLAIN_CDR2 |
| APPENDABLE | PLAIN_CDR | DELIMITED_CDR |
| MUTABLE | PL_CDR | PL_CDR2 |

규격 용어는 "DELIMITED_CDR"이다 (DELIMITED_CDR2가 아니다). 캡슐화 식별자 이름은 D_CDR2_BE / D_CDR2_LE다.

### 캡슐화 식별자 (7.6.3.1.2, Table 60)

| 이름 | 값 |
|---|---|
| CDR_BE / CDR_LE | 0x0000 / 0x0001 |
| PL_CDR_BE / PL_CDR_LE | 0x0002 / 0x0003 |
| XML | 0x0004 |
| CDR2_BE / CDR2_LE | 0x0006 / 0x0007 |
| D_CDR2_BE / D_CDR2_LE | 0x0008 / 0x0009 |
| PL_CDR2_BE / PL_CDR2_LE | 0x000a / 0x000b |

Table 60 마지막 행은 "PL_CDR_LE"로 오타가 있다. 0x000b는 PL_CDR2_LE다.

### 핵심 규칙
- **최대 정렬** (7.4.3.2, MAXALIGN): XCDR1 = 8, XCDR2 = 4. XCDR2에서 8바이트 타입은 4로 정렬한다.
- **DHEADER** (7.4.3.4.1): 뒤따르는 객체의 직렬화 크기 (자기 자신 제외)를 담은 UInt32. 4바이트 정렬, 스트림 엔디안
- **EMHEADER** (7.4.3.4.2): `(M_FLAG << 31) + (LC << 28) + (MemberId & 0x0FFFFFFF)`

| LC | 헤더 크기 | 멤버 길이 |
|---|---|---|
| 0 / 1 / 2 / 3 | 4 | 1 / 2 / 4 / 8바이트 |
| 4 | 8 | NEXTINT |
| 5 | 8 | NEXTINT. NEXTINT를 멤버 자신의 앞쪽 길이 필드 (예: string, sequence 길이)로 재사용해 4바이트 절약 |
| 6 | 8 | 4 * NEXTINT. NEXTINT를 멤버의 원소 개수로 재사용 |
| 7 | 8 | 8 * NEXTINT. NEXTINT를 멤버의 원소 개수로 재사용 |

- **XCDR1 PL_CDR** (7.4.1.2.1, Table 34): PID_EXTENDED 0x3F01, PID_LIST_END 0x3F02. must-understand 비트를 붙인 PID_EXTENDED는 0x7F01
- **멤버 ID 기본값** (7.3.1.2.1.1): `@autoid`가 없으면 SEQUENTIAL (선언 순서). 이름 해시는 `@autoid(HASH)` / `@hashid`일 때만
- **TypeObject 해시** (7.3.4.9.2, Annex B): TypeObject를 XCDR2 리틀엔디안으로 직렬화한 MD5의 앞 14바이트. EK_MINIMAL 0xF1, EK_COMPLETE 0xF2
- **디스커버리**: PID_TYPE_INFORMATION 0x0075 (`@id(0x0075) @optional TypeInformation type_information`), type_name은 `@id(0x0007)`, 1.1의 `type`은 `@id(0x0072)` (7.6.3.2.2, Annex D)
- **KeyHash** (XTypes 7.6.8, RTPS 2.5 9.6.4.8): 키 홀더를 FINAL로 만들고 키가 아닌 멤버를 지운 뒤 멤버 ID 순으로 PLAIN_CDR2 빅엔디안 (최대 정렬 4, 패딩 0)으로 직렬화. 최대 직렬화 크기가 16바이트 이하면 16바이트로 0을 채우고, 아니면 MD5

### 구현 현황 [추정, 원문 아님]
- Cyclone DDS: XCDR1 / XCDR2 모두 지원. 타입 디스커버리는 빌드 옵션
- Fast DDS: 둘 다 지원. 3.x는 non-final 타입에 XCDR2를 선호
- ROS 2: 모든 메시지가 FINAL이라 XCDR1 CDR_LE

## 3. IDL 4.2

| 내용 | 섹션 |
|---|---|
| 어휘 규칙 | 7.2 |
| 전처리 | 7.3 |
| 문법 전체 | 7.4 |
| Core Data Types | 7.4.1 |
| Any | 7.4.2 |
| Interfaces - Basic (생략 대상) | 7.4.3 |
| Extended Data-Types (struct 상속 7.4.13.4.1, map / bitset / bitmask / int8 / uint8 7.4.13.4.3 ~ 7.4.13.4.4) | 7.4.13 |
| Anonymous Types | 7.4.14 |
| Annotations (문법) | 7.4.15 |
| 빌딩 블록 간 관계 | 7.4.16 |
| 이름과 스코프 | 7.5 |
| 표준 어노테이션 | 8 |
| @id / @autoid / @optional | 8.3.1.1 / 8.3.1.2 / 8.3.1.3 |
| @extensibility / @final / @appendable / @mutable | 8.3.1.6 / 8.3.1.7 / 8.3.1.8 / 8.3.1.9 |
| @key / @must_understand | 8.3.2.1 / 8.3.2.2 |

- `#pragma keylist`는 IDL 4.2, DDS 1.4, XTypes 1.3, RTPS 2.5 어디에도 없다. 벤더 도구 (RTI, OpenSplice, OpenDDS 등)의 관례다 [추정].
- @data_representation, @topic 같은 DDS 전용 어노테이션은 IDL 4.2가 아니라 XTypes 쪽이다.

## 4. DDS-Security

### 버전
- 1.0, 1.1 (formal/2018-04-01, 2018-07), 1.2 (formal/25-03-06)
- **1.2에서 장 번호가 바뀌었다.** 이 프로젝트는 1.2 기준으로 적는다.
- 1.2의 주요 변경 (목차 비교, 변경 이력 문서는 없음)
  - 7.3에 새 타입 (CryptoAlgorithmSet, ParticipantSecurity*AlgorithmInfo 등). ParticipantSecurityInfo, EndpointSecurityInfo를 대체
  - 공통 암호 알고리즘 장 신설. RSA-2048과 일부 ECDH / ECDSA는 1.2에서 deprecated
  - 보안 TypeLookup 토픽과 엔드포인트 (7.5.5, 7.5.11)
  - PSK 플러그인: DDS:Auth:PSK (10.7), DDS:Access:PSK (10.8), DDS:Crypto:PSK (10.9)
  - 새 내장 토픽 DCPSParticipantSecure, DCPSPublicationsSecure, DCPSSubscriptionsSecure (7.5.1.6 ~ 7.5.1.8)
  - 기준 RTPS 버전이 2.3에서 2.5로

### 섹션 지도 (1.2 기준, 괄호는 1.1)

| 내용 | 1.2 섹션 (1.1) |
|---|---|
| 보안 관련 타입 | 7.3 |
| 와이어 보안 | 7.4 (7.3). 보안 RTPS 메시지 7.4.2 |
| 보안 서브메시지 정의 | 7.4.6.3 ~ 7.4.6.7 |
| 보안 내장 EntityId | 7.4.7.1 (Table 9) |
| UDP/IP 와이어 매핑 | 7.4.7.5 ~ 7.4.7.9 |
| DCPSParticipantMessageSecure | 7.5.2 (7.4.2) |
| DCPSParticipantStatelessMessage | 7.5.3 (7.4.3) |
| DCPSParticipantVolatileMessageSecure | 7.5.4 (7.4.4) |
| 보안 내장 엔드포인트 보호 | 7.5.13 |
| 플러그인 구조 | 9 (8) |
| Authentication / AccessControl / Cryptographic SPI | 9.3 / 9.4 / 9.5 |
| Logging / Data Tagging | 9.6 / 9.7 (Logging은 1.1에서 8.6) |
| 플러그인 동작 | 9.8 |
| 내장 플러그인 | 10 (9) |
| DDS:Auth:PKI-DH | 10.3. 토큰 10.3.2.5, 핸드셰이크 프로토콜 10.3.4.2 (Table 62) |
| DDS:Access:Permissions (Permissions / Governance XML) | 10.4 |
| DDS:Crypto:AES-GCM-GMAC | 10.5. 키 자료와 인코드 / 디코드 10.5.2 ~ 10.5.3 |
| 플러그인 언어 API | 11 |

### 보안 서브메시지 ID (7.4.7.5 ~ 7.4.7.9)

| ID | 이름 | 규격의 타입 이름 |
|---|---|---|
| 0x30 | SEC_BODY | SecureBodySubMsg |
| 0x31 | SEC_PREFIX | SecurePrefixSubMsg |
| 0x32 | SEC_POSTFIX | SecurePostfixSubMsg |
| 0x33 | SRTPS_PREFIX | SecureRTPSPrefixSubMsg |
| 0x34 | SRTPS_POSTFIX | SecureRTPSPostfixSubMsg |

### 핸드셰이크
- HandshakeRequest -> HandshakeReply -> HandshakeFinal. GUID가 작은 참가자가 시작한다 (10.3.4.2).
- SPI 호출: begin_handshake_request, begin_handshake_reply, process_handshake (9.3.2.11.x)
- 핸드셰이크는 DCPSParticipantStatelessMessage (best-effort, stateless)로, 그 뒤 키 교환은 DCPSParticipantVolatileMessageSecure (reliable, volatile)로 보낸다.
