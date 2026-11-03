# 저자 노트: CDR 직렬화 (XCDR1)

상태: 완료

[roadmap](../roadmap.md) | chapter: 6 ([목차](../../book/README.md#목차)) | 본문: [book/06-cdr/README.md](../../book/06-cdr/README.md)

## 범위
- 질문: 샘플 (ShapeType 값 하나)은 어떤 규칙으로 바이트가 되는가. chapter3, 4의 캡처에서 본 사용자 DATA의 페이로드 32바이트 (헤더 4 + 데이터 28. `05 00 00 00 42 4c 55 45 00 ...`은 데이터 부분)를 손으로 읽고, 같은 바이트를 내는 코드를 쓴다.
- 읽을 규격
  - CORBA 3.3 Part 2 Sec. 9.3 CDR Transfer Syntax: 9.3.1.1 Alignment (Table 9.1), 9.3.1.2 ~ 9.3.1.5 (정수, 부동소수점, octet, boolean), 9.3.2 구성 타입 (정렬 9.3.2.1, struct 9.3.2.2, union 9.3.2.3, array 9.3.2.4, sequence 9.3.2.5, enum 9.3.2.6, string 9.3.2.7)
  - DDS-XTypes 1.3 Sec. 7.4.1.1 PLAIN_CDR (Table 31 원시 타입 크기와 정렬, Table 32 enum, 7.4.1.1.2 문자열은 UTF-8, 끝 패딩은 0), Sec. 7.4.3.2 스트림 상태 (origin, maxalign = 8), Sec. 7.4.3.5.3 규칙 (1) ~ (3), (5), (7), (8), (10), (13), (17), (18), (26), (29) (XCDR1에서 appendable은 final과 같다), Sec. 7.6.3.1.2 Table 60 (캡슐화 식별자)과 그 뒤의 options 하위 2비트 (끝 패딩 수)
  - RTPS 2.5 Sec. 10.1 (페이로드는 불투명), 10.2 (SerializedPayloadHeader. 정렬 기준은 헤더 4바이트 뒤), 10.5 (사용자 토픽의 식별자 표 10.3)
  - 섹션 지도: [research/xtypes-cdr-idl-security.md](../research/xtypes-cdr-idl-security.md)
- 구현: `crates/mdds-cdr`. `CdrWriter` / `CdrReader` (엔디안, XCDR 버전에 따른 최대 정렬, 정렬 기준은 자기 버퍼의 0), trait `CdrSerialize` / `CdrDeserialize`와 원시 타입, `str` / `String`, `Vec<T>`, `[T; N]` 구현, `encapsulation` 모듈 (식별자 newtype과 상수, 헤더 4바이트 해석, options의 패딩 비트, `to_payload` / `from_payload`). `ShapeType`은 chapter의 테스트 타입으로 `tests/shape_type.rs`에 손으로 구현한다.
- 미루는 것: PL_CDR과 ParameterList 해석 (chapter7, 9. 식별자 값과 헤더 해석만 여기서 둔다), key만 실리는 DATA의 형식과 KeyHash (chapter13), XCDR2의 DHEADER, EMHEADER, 식별자 0x06 ~ 0x0b (chapter16. 최대 정렬 4는 chapter13의 KeyHash가 먼저 쓰므로 writer의 매개변수로 지금 둔다), @optional 멤버 (chapter16), wchar / wstring (UTF-16. 쓰는 타입이 나올 때), union의 구현 (규칙만 적는다. 쓰는 타입이 나올 때), IDL에서 코드 생성 (chapter17), derive 매크로 (아래 결정).

## 틀
1. 왜 필요한가. RTPS는 페이로드를 해석하지 않는다 (RTPS 2.5 Sec. 10.1). 메모리를 그대로 (`repr(C)`) 보내면 안 되는 이유, CDR의 비용 (정렬 틈, 타입 정보가 바이트에 없음. protobuf, JSON / CBOR과 구조만 비교. 속도 판정은 출처가 없어 쓰지 않는다). 1.1 작업 디렉터리 (`my-code/`는 chapter5의 것을 이어 쓴다).
2. 캡처에서 보기. Lab 6-1 ShapeType 바이트 손으로 읽기. 준비에 용어 (페이로드 = 캡슐화 헤더 4 + 본문, 본문 = 데이터 + 끝 패딩)와 "읽는 법 4줄". 프레임 50은 따라 읽기 (빈 표와 접은 정답), 프레임 32는 혼자 읽기, 프레임 268은 끝 패딩 하나만 (dispose는 한 줄). tshark 명령도 둔다.
3. 규격 읽기. 3.1 세 규격의 역할 (두 문장과 표). 3.2 정렬 (기준점은 헤더 뒤, 상한 8, 중첩 struct, 8바이트 멤버의 예 표). 3.3 엔디안. 3.4 캡슐화 헤더 (식별자 4개, XCDR2 식별자의 값 불일치 한 문단, options 하위 2비트). 3.5 타입별 규칙 표.
4. 구현. 4.1 설계 (모듈 넷, 요점 넷, 공개 API 표, 오류 표). Lab 6-2 (단계마다 검사, 매크로 뼈대, 테스트 이름 표). 4.3 설계 메모 (serde, derive, trait에 넣지 않은 것, 남은 바이트).
5. 벤더와 맞춰 보기. Lab 6-3 (`tests/shape_type.rs`. 바이트는 tshark 명령으로 뽑고 헤더를 붙인다. 테스트 여섯 개의 표).
6. 두 벤더는 어떻게 하는가. 표 한 장 (정렬 기준점, options 패딩 비트, 모르는 식별자, 타입 기술 방식, 문자열의 NUL, 틈의 값). 함수와 줄 번호는 조사 노트에.
7. 스스로 확인하기 (chapter5와 같다).

## 결정
- **헤더 4바이트는 `encapsulation` 모듈이 다루고 `CdrWriter` / `CdrReader`는 데이터 스트림만 본다. 정렬 기준은 자기 버퍼의 0이다.** `to_payload`가 헤더 + 데이터 + 끝 패딩을 조립하고 options의 패딩 비트를 채운다. 기준점을 다시 잡는 일 (RTPS 2.5 Sec. 10.2의 헤더 뒤, Sec. 9.4.2.11의 ParameterList 값마다, XTypes 규칙 (24) / (25)의 PL 멤버마다)은 모두 "부분 슬라이스 위의 새 reader / writer"로 표현한다. 벤더 확인 결과가 RTPS 2.5와 다르게 나와도 writer를 만드는 쪽에서 초기 offset 한 줄만 바꾸면 된다. 버린 대안: writer가 헤더를 쓰고 origin = 4로 정렬. 헤더가 있을 때와 없을 때 (ParameterList 값) 두 경우를 한 타입이 알아야 한다. 모듈 이름 `payload`는 chapter7의 serializedPayload와 헷갈려 `encapsulation`으로 한다 (용어 정리의 encapsulation kind, CORBA 9.3.3과 같은 말).
- **정렬 기준점은 RTPS 2.5 Sec. 10.2를 따른다 (헤더 뒤에서 다시 센다).** XTypes 규칙 (1)은 `PUSH(ORIGIN = 0)`을 options 앞에 두어 글자 그대로 읽으면 스트림 시작이 기준이 되지만, 같은 표기를 쓰는 규칙 (24) / (25)가 파라미터 헤더 뒤에서 다시 세는 것과 맞지 않으므로 표기 문제로 본다 [추정]. 차이는 8바이트 정렬 값 (int64, uint64, float64)이 있는 타입 전체에서 난다 (데이터가 절대 위치 4에서 시작하므로 8의 배수 자리가 서로 어긋난다). ShapeType에는 없어 캡처로는 가릴 수 없다. 본문에는 두 문장을 짧게 나란히 두고 RTPS 2.5 각주 5 (2.4 이전 판은 불분명했다)를 적는다. 두 벤더의 소스로 확인한 결과는 6절에 둔다. 테스트 `{i32, i64}`로 고정한다 (헤더 뒤 기준이면 틈 4).
- **`CdrWriter` / `CdrReader`는 엔디안과 XCDR 버전 (`XcdrVersion::V1` / `V2`, 최대 정렬 8 / 4)을 만들 때 받는다.** 이 chapter는 V1만 쓰지만 chapter13의 KeyHash (PLAIN_CDR2 빅 엔디안, RTPS 2.5 Sec. 9.6.4.8)가 chapter16보다 먼저 최대 정렬 4를 쓴다. 버린 대안: 최대 정렬을 chapter16에서 더하기. chapter13에서 trait 밖의 writer 생성자가 바뀐다.
- **`Endianness`는 mdds-cdr에 하나만 둔다.** chapter7의 서브메시지 E 플래그도 이 타입을 쓴다 (의존 방향 mdds-rtps -> mdds-cdr, chapter7에서 추가. roadmap에 적었다). RTPS의 서브메시지 요소도 CDR 인코딩을 따르므로 (RTPS 2.5 Sec. 9.2.2) chapter7은 `CdrReader` / `CdrWriter`의 원시 타입 메서드를 쓸 수 있다. 캡슐화 헤더 층은 사용자 페이로드와 SPDP / SEDP 페이로드에만 쓴다.
- **`serialize`도 `Result<(), CdrError>`다.** `Vec`에 쓰는 일은 실패하지 않지만 Rust 값이 CDR로 표현될 수 없는 경우가 있다: 내부 NUL이 든 `String` (`StringContainsNul`), `u32`를 넘는 길이 (`LengthOverflow`). bound (`string<128>`) 검사는 와이어에 없고 규격이 받는 쪽 검사를 요구하지 않으므로 chapter17의 생성 코드 자리로 남긴다. 버린 대안: `()`를 돌려주고 `as u32`로 자르기. 뒤에 바꾸면 손으로 쓴 impl 전부를 고친다.
- **읽기는 길이 필드를 믿지 않는다.** sequence / string 길이가 남은 바이트보다 크면 즉시 `UnexpectedEnd`, 미리 할당하는 메모리는 남은 바이트 이하 (`min(길이, 남은 바이트 / size_of::<T>())`), 슬라이스는 `get`으로 자른다. chapter7의 fuzz가 같은 reader를 지난다. 오류에는 그 reader의 버퍼 기준 위치 (`offset`)를 싣는다 (`from_payload`에서는 헤더 뒤의 데이터 기준).
- **정수 읽기는 틈과 값을 합쳐 미리 검사한다 (`take_aligned`).** 모자라면 `needed`가 틈 + 값이고 읽기 위치는 움직이지 않는다. 프레임 268을 ShapeType으로 읽은 오류가 `needed: 3`이면 "int32 하나를 읽다 3바이트가 모자랐다"로 오해된다. 버린 대안: 틈을 먼저 건너뛰고 값에서 실패하기 (위치가 움직이고 `needed`가 값의 일부만 가리킨다).
- **헤더 생성자는 `for_data_len(id, data_len)`이다.** 패딩 수를 직접 받는 생성자는 4 이상이 조용히 잘려 options가 틀린 헤더를 만든다.
- **`CdrError` 변이는 이 chapter의 테스트가 실제로 만드는 것만 둔다.** `UnexpectedEnd { offset, needed, remaining }`, `InvalidBool { offset, value }`, `InvalidUtf8 { offset }`, `MissingNul { offset }`, `InvalidEnum { offset, value }`, `StringContainsNul`, `LengthOverflow`, `UnsupportedRepresentation(RepresentationId)`, `InvalidPadding { padding, len }`. `TrailingBytes`는 두지 않는다. 끝 패딩이 있고, 패딩 비트를 채우지 않는 구현이 있을 수 있으며, XCDR1 appendable은 DHEADER가 없어 멤버를 덧붙인 새 타입의 샘플을 옛 reader가 받을 수 있기 때문이다 [추정, chapter16에서 할당 가능성으로 확인]. 대신 `CdrReader::remaining()`을 두고 골든 테스트에서 0을 단언한다.
- **bool은 0 / 1만 받고 그 밖은 `InvalidBool`이다.** 규격은 받는 쪽 동작을 정하지 않았다 (CORBA 9.3.1.5, XTypes Table 31). mdds의 선택이라고 본문에 밝힌다. string 길이 0은 `MissingNul` (빈 문자열의 길이는 1. CORBA 9.3.2.7). 읽을 때 내부 NUL은 검사하지 않는다 (마지막 바이트가 NUL인지만).
- **정렬 틈은 쓸 때 0, 읽을 때 검사하지 않는다.** 규격은 틈의 값을 정하지 않았고 (CORBA 9.3.1.1, XTypes Table 37 ALIGN) 끝 패딩만 0을 요구한다 (XTypes 7.4.1.1). 캡처의 두 벤더는 틈도 0이다. KeyHash (chapter13)는 틈도 0이어야 하므로 (RTPS 2.5 Sec. 9.6.4.8) 쓰는 쪽은 0으로 통일한다.
- **식별자는 newtype `RepresentationId([u8; 2])`와 상수 (CDR_BE, CDR_LE, PL_CDR_BE, PL_CDR_LE, 그리고 Table 60의 XCDR2 여섯 개).** 모르는 값도 담아야 한다 (chapter5의 `DataRepresentationId`와 같은 이유). 헤더 해석 (`EncapsulationHeader::split`)은 어떤 식별자든 받고, 지원 판단은 `from_payload`만 한다 (CDR_BE / CDR_LE 이외는 `UnsupportedRepresentation`). chapter9의 SPDP / SEDP (PL_CDR_LE)가 헤더 해석을 그대로 쓴다. QoS의 `DataRepresentationId` (XCDR = 0)와 캡슐화 식별자 (CDR_LE = 0x0001)는 다른 번호 공간이라고 본문에 한 줄 둔다. XCDR2 값은 RTPS 2.5 표 10.3 (0x10 ~ 0x15)과 XTypes 1.3 표 60 (0x06 ~ 0x0b)이 다르고 와이어 (xcdr2 캡처 0x0009)는 표 60이므로 상수는 표 60을 따른다 ([노트 16 확인할 것](16-xtypes.md#확인할-것)).
- **options는 바이트 `[u8; 2]`로 다루고 둘째 바이트의 하위 2비트만 해석한다.** 쓸 때는 끝 패딩 수, 읽을 때는 그만큼을 본문에서 뺀다 (XTypes 7.6.3.1.2가 받는 쪽에 요구). 나머지 비트는 무시한다. 패딩 수가 본문 길이보다 크면 `InvalidPadding`. 패딩 바이트가 0인지는 읽을 때 검사하지 않는다. RTPS 2.5 Sec. 10.3의 "보내는 쪽은 0, 받는 쪽은 무시"는 디스커버리 내장 엔드포인트에만 적힌 문장이라 사용자 토픽에는 쓰지 않는다.
- **원시 타입 매핑: u8, i8, u16, i16, u32, i32, u64, i64, f32, f64, bool.** Rust `char` (4바이트)와 `usize`에는 구현하지 않는다 (IDL char는 1바이트 Char8이고 해석하지 않는다. XTypes 7.4.1.1.2). f32 / f64는 비트를 그대로 옮기고 테스트는 비트로 비교한다 (NaN). Float128은 범위 밖. `write_bytes` / `read_bytes`를 두어 chapter7의 octet 배열 (GuidPrefix 12, Locator 주소 16)에 쓴다. `[T]`에는 trait를 구현하지 않는다 (sequence와 array가 `&` 하나 차이로 다른 바이트가 된다). sequence 쓰기는 `write_sequence(&[T])`. `[T; N]` 역직렬화는 `Vec`으로 모은 뒤 `try_into`.
- **enum은 사용자가 trait를 직접 구현한다. 기본 holder는 i32다.** CORBA는 unsigned long이지만 XTypes 1.3 Table 32는 bit bound에 따라 Int8 / Int16 / Int32이고 기본 32다. `@bit_bound`와 `@value`가 있으면 크기와 값이 달라진다고 표에 적는다. 모르는 값은 `InvalidEnum`. union은 규칙 표만 둔다 (판별값 타입은 integer, char, boolean, enum. IDL 4.2 7.4.1. 어느 멤버도 고르지 않는 판별값과 default case가 있다. XTypes 규칙 (26)의 `?`). `Option<T>`에는 구현하지 않는다. @optional의 인코딩이 멤버 단위이고 버전마다 다르기 때문이다 (XCDR1은 멤버 헤더, XCDR2는 boolean. XTypes 7.4.3.5.2. chapter16).
- **derive 매크로는 이 chapter에서 만들지 않는다.** `ShapeType`의 구현은 멤버마다 한 줄이라 손으로 쓰는 편이 규칙을 그대로 보여 준다. proc-macro crate (`syn`, `quote` 의존)를 더하면 독자가 Rust 매크로 작성법을 배워야 하고 그것은 이 책의 주제가 아니다. 반복이 실제로 부담되는 곳은 chapter16 (TypeObject 계열 타입 수십 개)이므로 거기서 다시 정한다 (후보: 의존 없는 `macro_rules!`, Annex B IDL을 chapter17에서 생성). [노트 16](16-xtypes.md#확인할-것)에 적었다. 이 결정으로 book/README.md 6의 소제목, roadmap의 IDL 줄과 `mdds-derive` 후보, 노트 17 완료 기준을 고쳤다.
- **serde를 쓰지 않는다 (규약). 이유를 정확히 적는다.** serde Serializer는 타입을 알므로 정렬도 구현할 수 있다. 쓰지 않는 이유는 (1) XTypes가 멤버에 붙이는 정보 (@id, @key, @optional, @must_understand, 확장성)가 serde 데이터 모델에 없어 별도 속성 체계가 필요하고, (2) key만 직렬화하기 (chapter13)는 같은 타입의 두 번째 view이며, (3) 바이트 하나하나를 직접 쓰는 것이 이 책의 목적이기 때문이다. 비용: serde 생태계 (파생 매크로, 다른 형식)를 재사용하지 못한다. XCDR1 final 타입만 쓰는 응용 (ROS 2 메시지)이라면 serde 위의 CDR 구현도 충분하다. roadmap의 옛 이유 ("정렬을 표현할 수 없다")를 고쳤다.
- **key 직렬화, 최대 직렬화 크기, 타입 이름은 `CdrSerialize`에 넣지 않는다.** chapter13 (key holder, KeyHash의 16바이트 판정), chapter11 (타입 이름 매칭)이 위층 trait로 더한다. mdds-cdr은 인코딩만 맡는다.
- **골든 바이트는 Wireshark에서 옮긴 hex 리터럴로 `tests/shape_type.rs`에 둔다.** pcap을 읽는 코드는 chapter7에 mdds-rtps에 생기므로 mdds-cdr의 테스트가 쓰면 의존이 거꾸로 된다. 리터럴마다 캡처 이름과 프레임 번호를 주석에 적는다. chapter7은 이 리터럴을 바꾸지 않고, 캡처의 사용자 DATA를 mdds-cdr로 역직렬화하는 end-to-end 테스트를 mdds-rtps `tests/`에 더한다 ([노트 7](07-rtps-messages.md) 완료 기준). `ShapeType`을 chapter8의 첫 전송에서 어디에 둘지는 [노트 8](08-best-effort-udp.md)에 남겼다.
- **프레임 268 (key만 실린 DATA)은 끝 패딩 하나를 배우는 데만 쓴다.** key 페이로드의 바이트 형식을 정의하는 문장은 RTPS 2.5에도 XTypes 1.3에도 없다 (RTPS는 "key의 값"이라고만 한다. Sec. 9.4.2.12). "key 멤버만 가진 struct를 writer의 표현으로 직렬화한 것으로 보인다"는 읽기와 캡처가 맞지만 본문은 단정하지 않고 chapter13으로 넘긴다. Lab 6-3은 헤더 층이 패딩 3을 읽고 본문을 `String` 하나로 읽는 것만 단언한다.
- **스스로 확인하기를 둔다 (연습 문제 대신. chapter5와 같다).** 캡처 README의 chapter6 질문 (xcdr2의 DHEADER)은 "XCDR1에서는 appendable도 final과 같은 바이트"까지 답하고 DHEADER는 chapter16으로 넘긴다 (캡처 README의 화살표를 나누었다).

## Lab
| Lab | 하는 일 | 기대 결과 |
|---|---|---|
| 6-1 | 사용자 DATA 세 개의 페이로드 바이트를 "읽는 법 4줄"로 손으로 읽는다 | 50: 데이터 28바이트가 BLUE 161 156 20 [] (sub.log와 같다). 32: BLUE 194 66 20 []. 268: 데이터 12바이트 = key 9 + 끝 패딩 3, options `00 03` |
| 6-2 | `mdds-cdr`의 writer / reader / trait / encapsulation을 쓴다 | `cargo test -p mdds-cdr` 통과 (완성본은 단위 34: writer 10, reader 8, serialize 7, encapsulation 9). 단계마다 검사 (정렬 행렬, string, sequence, reader가 writer의 역, 헤더와 끝 패딩) |
| 6-3 | 세 캡처의 바이트를 골든 테스트로 적는다 | 50과 바이트 단위로 같은 32바이트, 32 역직렬화와 남은 바이트 0, 268의 패딩 3과 BLUE, xcdr2 41은 `UnsupportedRepresentation(D_CDR2_LE)` |

## 확인한 것 (원문, 2026-11-04)
- CORBA 3.3 Part 2: 자연 정렬과 최소 틈, 틈의 값은 정하지 않음 (Sec. 9.3.1.1, 3962-3975행), 기준은 스트림 시작 (4006-4009), Table 9.1 (char 1, short 2, long 4, long long 8, float 4, double 8, long double 8, boolean 1, enum 4), 정수는 2의 보수 (9.3.1.2), boolean은 1바이트 0 / 1 (4145), 구성 타입은 원시 원소의 정렬 외에 정렬이 없다 (9.3.2.1, 4214-4219), struct는 선언 순서 (9.3.2.2), union은 판별값 뒤 선택 멤버 (9.3.2.3), array는 길이 없이 원소만, 다차원은 마지막 차원이 가장 빨리 바뀜 (9.3.2.4), sequence는 unsigned long 길이 뒤 원소 (9.3.2.5), enum은 unsigned long이고 첫 식별자가 0 (9.3.2.6), string은 unsigned long 길이 (NUL 포함) 뒤 바이트와 NUL, 빈 문자열의 길이는 1 (9.3.2.7, 4248-4252). Encapsulation은 첫 바이트가 엔디안 플래그 (9.3.3. RTPS는 이 대신 4바이트 헤더를 쓴다).
- XTypes 1.3: PLAIN_CDR은 final과 appendable에 쓴다 (5850-5851행), RTPS 서브메시지 정렬용 끝 패딩은 0 (5856-5859), Table 31 (Int64 / Float64 정렬 8, Float128 크기 16 정렬 8, UInt8 크기 1 정렬 1), Char8은 해석하지 않고 Byte처럼 (5899-5900), String<Char8>은 UTF-8이고 길이는 NUL 포함 바이트 수 (5901-5908), enum은 bit bound로 Int8 / Int16 / Int32이고 기본 32 (Table 32, 5926-5936), @value (3946-3955), origin과 offset (Table 36, 6307-6321), ALIGN의 패딩 값은 정하지 않음 (6373-6374), MAXALIGN(VERSION1) = 8 (6368), 첫 2바이트는 ENC_HEADER (6389-6397), AsFinal (Table 38, 6414-6418), 규칙 (1) 6752-6761, (3) string 6773-6776, (5) enum 6785-6787, (7) alias 6789-6791, (8) / (10) array 6795-6808, (11) / (13) sequence 6818-6838, (17) / (18) final struct, (24) / (25) PL 멤버 뒤 `PUSH(ORIGIN=0)` 6905-6925, (26) union 6930-6933과 멤버 없는 판별값 (6683-6684), (29) appendable v1 = AsFinal 6999-7001, @optional의 XCDR1 멤버 헤더 (6737-6741), 식별자 첫 바이트 0xC0 ~ 0xFF는 벤더 정의 (10150-10153), Table 60 (10178-10208. 0x0b 행 이름은 오타), options 둘째 바이트 하위 2비트 = 끝 패딩 수이고 받는 쪽이 이것으로 끝을 정한다 (10218-10224), 나머지 비트는 정의하지 않음 (10215-10217), KeyHolder (7.2.2.4.7, 2553-2560).
- RTPS 2.5: Sec. 10.1 페이로드는 불투명 (10721-10735행), 10.2 헤더 4바이트와 "정렬을 위해 스트림은 options 뒤에서 논리적으로 다시 시작한다. 처음 패딩은 없어야 한다" (10765-10767), 각주 5 (2.4 이전 판은 불분명, 10772-10773), 10.3의 "sender 0, receiver 무시"는 디스커버리 내장 엔드포인트 전용 (10783-10785), 표 10.3 (CDR_BE 0x0000, CDR_LE 0x0001, PL_CDR_BE 0x0002, PL_CDR_LE 0x0003, XCDR2는 0x10 ~ 0x15), Sec. 9.2.2 서브메시지 요소도 CDR 인코딩을 따름 (7690), SequenceNumberSet / FragmentNumberSet만 예외 (8427, 8482), ParameterList 값마다 정렬을 다시 센다 (9.4.2.11, 8565-8566), serializedKey는 "key의 값" (9.4.2.12, 8675-8676), KeyHash는 틈도 0 (10458-10459).
- IDL 4.2: union 판별값 타입 (1157-1160, 1688-1689), default case (1704-1709), bounded string은 "maximum size" (1590-1593. 단위는 밝히지 않음. XTypes 1546-1549로 보면 Char8 원소 수 [추정]).
- 캡처 (tshark 4.6.9): reliable-cyclone-pub 50 (seq 12) `CDR_LE`, options `00 00`, 데이터 28바이트 `05000000 424c554500 000000 a1000000 9c000000 14000000 00000000` (sub.log 첫 줄 `BLUE 161 156 [20]`). reliable-fastdds-pub 32 (Fast DDS writer 0x00000102의 첫 DATA) `CDR_LE`, options `00 00`, `... c2000000 42000000 ...` (sub.log `BLUE 194 066 [20]`). dispose-on-exit 268 `serializedKey`, `CDR_LE`, options `00 03` (Wireshark: Padding bytes 3), 데이터 12바이트 `05000000 424c554500 000000`. xcdr2 41 `D_CDR2_LE` 0x0009, `1c000000` (DHEADER 28) 뒤 같은 배치.
- shape.idl (dds-rtps `526dc56e` `srcC/shape.idl`, GitHub 원문): `@appendable struct ShapeType { @key string<128> color; int32 x; int32 y; int32 shapesize; sequence<uint8> additional_payload_size; }`. 같은 커밋의 `srcC/shape_main.c` (1588, 1734행)와 `srcCxx/shape_main.cxx` (1914, 2086행)는 x, y를 `%03d`로 출력한다 (`194 066`은 벤더 차이가 아니다).
- Wireshark 4.6.9: 프레임 268의 Info는 `DATA([_D])` (PID_STATUS_INFO는 Disposed만). `serializedKey` 아래 리프 이름은 여전히 `serializedData`다. options 아래의 `Compression class Id`는 Wireshark의 항목이고 두 규격에 없다. XCDR2 식별자 값은 XTypes 안에서도 Table 39 (0x10 ~ 0x15)와 Table 60 (0x06 ~ 0x0b)이 다르다.
- Fast DDS 이미지의 Fast-CDR은 `v2.3.5` (`7d33a3b5`), fastddsgen `v4.3.0` ([docker/fastdds/Dockerfile](../../docker/fastdds/Dockerfile)).

- 두 벤더의 직렬화 소스 (정렬 기준점은 헤더 뒤, options 패딩 비트는 둘 다 채움, 모르는 식별자는 둘 다 거부, 타입 기술은 opcode 해석 / 생성 코드, 길이 검사 방식의 차이): [research/vendor-cdr-serialization.md](../research/vendor-cdr-serialization.md) (고정 커밋, 2026-11-04).

## 확인할 것
- XCDR1 appendable 타입에 멤버를 덧붙인 새 버전의 샘플을 옛 reader가 받을 수 있는지 (뒤에 남는 바이트를 허용하는 근거). XTypes 할당 가능성 (7.2.4)으로 chapter16에서 확인.
- bool의 0 / 1 이외 값을 두 벤더가 어떻게 받는지 (소스 미확인).
- Fast-CDR은 틈을 쓰지 않고 버퍼 초기화에 기대므로, 버퍼를 다시 쓸 때 이전 샘플의 바이트가 틈에 남을 수 있다 [추정]. color 길이가 다른 샘플을 번갈아 보내는 캡처로 확인할 수 있다.
- Wireshark 4.6.9의 `Copy` > `...as a Hex Stream` 메뉴 문구 (Lab 6-3 준비). GUI를 열어 확인하지 않았다.
- 8바이트 정렬 값이 있는 타입의 상호운용 (정렬 기준점 결정의 실제 확인)은 float64가 있는 ROS 2 메시지로 chapter12에서 한다 ([노트 12](12-ros2-interop.md)).

## 완료 기준
- Lab 6-1을 끝낸 독자가 프레임 50의 데이터 28바이트를 멤버와 틈으로 나누어 설명할 수 있다.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`가 통과하고, 스냅샷 `book/06-cdr/code`에서도 통과한다. `diff -r crates book/06-cdr/code/crates`가 비어 있다.
- ShapeType `{BLUE, 161, 156, 20, []}`을 CDR_LE로 직렬화한 결과 (헤더 포함 32바이트)가 reliable-cyclone-pub 50의 `serializedData` (헤더 포함)와 같다. 데이터 끝의 `00 00 00 00`은 다섯 번째 멤버 (길이 0의 sequence)다.

## 진행 기록
- 2026-11-04: 계획을 네 관점 (독자, 시니어 엔지니어, 아키텍트, DDS 전문가)으로 검토해 반영. 바이트 수 (28 + 4), writer에 XCDR 버전, 쓰기도 `Result`, 길이 필드 불신, `encapsulation` 모듈 분리, union은 규칙만, derive는 chapter16에서 다시 -> [결정](#결정)
- 2026-11-04: 두 벤더의 직렬화 소스 확인 (정렬 기준점과 패딩 비트는 mdds와 같다) -> [research/vendor-cdr-serialization.md](../research/vendor-cdr-serialization.md)
- 2026-11-04: `mdds-cdr` 구현, 스냅샷 `book/06-cdr/code`, 본문 초안 -> [본문](../../book/06-cdr/README.md)
- 2026-11-04: 초안을 네 관점으로 검토해 반영. 용어 (페이로드 / 본문 / 데이터) 통일, `DATA([_D])`, `take_aligned`, `for_data_len`, 6절의 bound 문장 삭제, 추정을 추정으로 표시 (단위 34, 골든 6) -> [결정](#결정)
