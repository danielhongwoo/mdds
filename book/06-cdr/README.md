# 6. CDR 직렬화

[목차](../README.md) | 이전: [5 DCPS 개념 모델](../05-dcps-concepts/README.md) | 다음: 7 RTPS 메시지

chapter3과 4에서 사용자 DATA 안의 `serializedData`를 열어 `05 00 00 00 42 4c 55 45 ...` 같은 바이트를 보았다. 이 chapter는 샘플 (ShapeType 값 하나)이 그 바이트가 되는 규칙, 곧 [CDR](../glossary.md#직렬화) (Common Data Representation)을 다룬다. 캡처의 바이트를 손으로 읽고, 규격에서 규칙을 확인한 뒤, 같은 바이트를 내는 코드를 `mdds-cdr` crate에 쓰고, 캡처와 바이트 단위로 맞춘다.

끝나면 ShapeType의 바이트를 멤버와 틈으로 나누어 설명할 수 있고, 어떤 struct든 CDR 바이트로 바꾸고 되돌리는 코드가 테스트와 함께 있다.

## 1. 왜 필요한가

RTPS는 페이로드를 해석하지 않는다. DATA 서브메시지의 `serializedData` (이 책은 이것을 페이로드라고 부른다)는 RTPS에게 해석하지 않는 바이트열이고, 그 안을 어떻게 채우고 읽는지는 위층 (DDS의 DataWriter와 DataReader)의 일이다 (RTPS 2.5 Sec. 10.1). chapter8에서 첫 샘플을 보내려면 그 바이트를 만드는 규칙이 먼저 있어야 한다.

왜 메모리를 그대로 보내지 않는가. Rust의 struct를 바이트로 복사해 보내면 받는 쪽이 같은 배치로 읽을 수 있을 것 같지만 그렇지 않다. 언어와 컴파일러마다 멤버 사이의 틈과 순서가 다르고 (Rust는 `#[repr(C)]`가 없으면 멤버 순서도 보장하지 않는다), `String`은 글자가 아니라 포인터와 길이이며, CPU마다 정수의 바이트 순서 (엔디안)가 다르다. 그래서 언어와 CPU에 상관없이 같은 바이트가 나오는 규칙이 필요하고, DDS는 CORBA에서 온 CDR을 쓴다. CDR은 값을 종류마다 정해진 크기로, 정해진 위치에 (정렬. 3.2절), 선언 순서대로 이어 붙이는 규칙이다. 타입 정보는 바이트에 없다. DDS-XTypes는 이 규칙을 받아들여 "Extended CDR 버전 1"이라 부르고 (흔히 XCDR1이라 줄여 쓴다), 둘째 판 XCDR2를 더했다. chapter4의 Lab 4-4에서 본 `-x 2`가 XCDR2이고, chapter16에서 다룬다.

CDR의 비용도 적어 둔다. 정렬 때문에 틈이 생긴다. Lab 6-1에서 볼 프레임 50은 데이터 28바이트 가운데 3바이트가 틈이다. 타입 정보가 바이트에 없으므로 받는 쪽이 타입을 미리 알아야 하고, 그래서 DDS는 디스커버리에서 타입 이름 (chapter11)과 타입 정보 (chapter16)를 교환한다. 다른 직렬화 방식과 구조만 비교하면 다음과 같다.
- protobuf: 멤버마다 번호와 길이를 붙인다. 틈이 없고 멤버를 더해도 옛 쪽이 읽지만, 값마다 그 헤더만큼 바이트가 는다.
- JSON, CBOR: 멤버 이름을 실을 수 있어 타입을 모르는 쪽도 읽지만, 그만큼 바이트가 는다.
- CDR: 고정된 타입을 같은 구조로 거듭 보내는 DDS의 쓰임새에 맞춘 것이다. 타입을 미리 알고 바이트만 보낸다.

### 1.1 작업 디렉터리

chapter5의 `my-code/`에 이어서 쓴다. chapter5를 건너뛰었으면 프로젝트 디렉터리에서 chapter5의 완성 코드를 복사한다 (`my-code/`의 뜻은 chapter5 1.1).
```bash
cp -R book/05-dcps-concepts/code my-code
```
`my-code/crates/mdds-cdr/src/lib.rs`에는 주석 한 줄만 있다. 이 chapter가 그 crate를 채운다. 명령은 `my-code/`에서 실행하고 (`cd my-code`), 시작 상태는 다음 명령이 테스트 0개로 통과하는 것이다.
```bash
cargo test -p mdds-cdr
```
완성 코드는 [`code/`](code/)에 있다. 자기 코드와 견주어 읽을 때는 프로젝트 디렉터리에서 diff한다. 테스트와 주석까지 같아질 필요는 없다. 판정 기준은 `cargo test`가 통과하는 것이다.
```bash
diff -r my-code/crates book/06-cdr/code/crates
```

## 2. 캡처에서 보기

### 2.1 Lab 6-1: ShapeType 바이트를 손으로 읽기

#### 2.1.1 목표

사용자 DATA 세 개의 페이로드를 규칙 네 줄만으로 손으로 읽어, 어느 바이트가 어느 멤버이고 어디가 틈인지 표로 만든다. 구독 로그의 값과 같으면 성공이다.

#### 2.1.2 준비

[`reliable-cyclone-pub.pcap`](../../captures/cyclone-fastdds/README.md#reliable-cyclone-pub), [`reliable-fastdds-pub.pcap`](../../captures/cyclone-fastdds/README.md#reliable-fastdds-pub), [`dispose-on-exit.pcap`](../../captures/cyclone-fastdds/README.md#dispose-on-exit)을 Wireshark 4.6.9로 연다 (읽는 법은 [wireshark.md](../../docs/environment/wireshark.md)).

| 캡처 | 프레임 | 무엇 | 역할 |
|---|---|---|---|
| reliable-cyclone-pub | 50 | Cyclone writer의 사용자 DATA (seq 12) | 규칙을 적용하며 따라 읽는다 |
| reliable-fastdds-pub | 32 | Fast DDS writer의 첫 사용자 DATA | 규칙만으로 혼자 읽는다 |
| dispose-on-exit | 268 | Cyclone writer의 마지막 DATA (`DATA([_D])`) | 새 현상 하나, 끝 패딩 |

용어를 정한다. 상세 창에서 `serializedData`를 펼치면 `encapsulation kind`, `encapsulation options`, 그리고 같은 이름의 `serializedData` 리프 (트리의 말단 항목)가 나온다.
- **페이로드**: `serializedData` 전체. 캡슐화 헤더 4바이트와 본문이다.
- **캡슐화 헤더**: 앞의 두 항목 (`encapsulation kind` 2바이트 + `encapsulation options` 2바이트). 3.4절에서 읽는다.
- **본문**: 리프의 바이트. 멤버의 바이트인 **데이터**와, 끝에 붙을 수 있는 **끝 패딩**으로 이루어진다. 프레임 50과 32는 끝 패딩이 없어 본문이 곧 데이터다.

샘플의 타입은 chapter4의 Lab 4-4에서 본 ShapeType이다 (shape_main의 `shape.idl`). `@appendable`은 뒤에 멤버를 더할 수 있는 타입이라는 표시인데, 이 chapter의 범위 (XCDR1)에서는 표시가 없는 타입 (final)과 같은 바이트가 된다 (3.5절).
```
@appendable struct ShapeType {
  @key string<128> color;
  int32 x;
  int32 y;
  int32 shapesize;
  sequence<uint8> additional_payload_size;
};
```

읽는 법은 네 줄이다. 규격의 근거는 3절에서 본다.
1. 캡슐화 헤더 4바이트 뒤의 데이터를 0번부터 센다. 이 위치를 오프셋이라고 한다.
2. 크기가 n바이트인 정수는 오프셋이 n의 배수인 곳에서 시작한다. 그 앞에 생기는 틈은 0으로 채워져 있다.
3. 문자열은 4바이트 길이 (끝의 NUL, 곧 0 바이트 하나를 포함한 바이트 수), 글자, NUL이다. sequence는 4바이트 원소 수 뒤에 원소들이다.
4. 정수는 리틀 엔디언이다. 낮은 자리 바이트가 먼저 온다 (`a1 00 00 00`은 0xa1 = 161).

#### 2.1.3 절차

1. 프레임 50을 열어 `DATA` 아래 `serializedData`를 펼친다. `encapsulation kind: CDR_LE (0x0001)`, `encapsulation options: 0x0000`, 그리고 데이터 28바이트가 보인다.
   ```
   05 00 00 00 42 4c 55 45 00 00 00 00 a1 00 00 00 9c 00 00 00 14 00 00 00 00 00 00 00
   ```
   아래 표를 읽는 법 네 줄로 채운다. 멤버는 IDL의 선언 순서다.

   | 오프셋 | 크기 | 멤버 또는 틈 | 바이트 | 값 |
   |---|---|---|---|---|
   | 0 | 4 | `color`의 길이 | `05 00 00 00` | 5 (`BLUE` 4 + NUL 1) |
   | 4 | 5 | `color`의 글자와 NUL | | |
   | | | 틈 (다음 int32를 4의 배수로) | | |
   | | 4 | `x` | | |
   | | 4 | `y` | | |
   | | 4 | `shapesize` | | |
   | | 4 | `additional_payload_size`의 원소 수 | | |

   <details>
   <summary>해설</summary>

   | 오프셋 | 크기 | 멤버 또는 틈 | 바이트 | 값 |
   |---|---|---|---|---|
   | 0 | 4 | `color`의 길이 | `05 00 00 00` | 5 |
   | 4 | 5 | `color`의 글자와 NUL | `42 4c 55 45 00` | `BLUE` |
   | 9 | 3 | 틈 | `00 00 00` | 다음 int32를 오프셋 12에 맞춘다 |
   | 12 | 4 | `x` | `a1 00 00 00` | 161 |
   | 16 | 4 | `y` | `9c 00 00 00` | 156 |
   | 20 | 4 | `shapesize` | `14 00 00 00` | 20 |
   | 24 | 4 | `additional_payload_size`의 원소 수 | `00 00 00 00` | 0 (원소 없음) |

   합계 28바이트. 구독 로그 `reliable-cyclone-pub.sub.log`의 첫 샘플 줄 `Square BLUE 161 156 [20]` (로그의 공백은 줄여 적었다)과 같다.

   </details>
2. 프레임 32를 같은 방법으로 혼자 읽는다. 벤더가 Fast DDS로 바뀌었고 값이 다르다. 검산은 `reliable-fastdds-pub.sub.log`의 첫 샘플 줄이다.

   <details>
   <summary>해설</summary>

   데이터는 `05 00 00 00 42 4c 55 45 00 00 00 00 c2 00 00 00 42 00 00 00 14 00 00 00 00 00 00 00`이다. 배치는 프레임 50과 같고 `x` = 0xc2 = 194, `y` = 0x42 = 66이다. 로그는 `Square BLUE 194 066 [20]`이다 (shape_main은 x와 y를 0을 채운 세 자리로 출력한다). 같은 타입이면 벤더가 달라도 같은 바이트 배치다. 그것이 상호운용의 바탕이다.

   </details>
3. 프레임 268을 연다. 발행자가 끝나면서 보낸 `DATA([_D])`로, 인스턴스를 지우는 메시지 (dispose)다. 이 메시지는 샘플 전체 대신 key 멤버 (`color`)만 싣는다. 그 이유와 형식은 chapter13의 몫이고, 여기서는 상세 창의 두 가지만 본다. 리프가 `serializedData` 대신 `serializedKey` 아래에 있고, `Encapsulation options (0x03)` 아래에 `Padding bytes: 3`이 있다 (그 옆의 `Compression class Id`는 Wireshark가 붙인 이름으로, 이 책의 규격에는 없는 항목이다). 본문은 12바이트다.
   ```
   05 00 00 00 42 4c 55 45 00 00 00 00
   ```
   이 가운데 데이터는 `color` 9바이트 (길이 4 + 글자와 NUL 5)이고, 뒤의 `00 00 00`은 틈이 아니라 끝 패딩이다. 다음 멤버가 없는데도 3바이트가 붙은 것은 페이로드 길이를 4의 배수로 맞추기 위해서이고, options의 3은 "끝의 3바이트는 데이터가 아니다"라는 뜻이다 (3.4절). 프레임 50과 32는 데이터가 28바이트라 이미 4의 배수이고, 그래서 options가 0이었다.
4. Wireshark 대신 tshark로 같은 바이트를 출력할 수도 있다. 프로젝트 디렉터리에서 실행한다. 데이터 (헤더 뒤 28바이트)와 캡슐화 헤더의 두 값이 탭으로 나뉘어 한 줄로 나온다.
   ```bash
   tshark -r captures/cyclone-fastdds/reliable-cyclone-pub.pcap -Y "frame.number == 50" -T fields -e rtps.issueData -e rtps.param.serialize.encap_kind -e rtps.param.serialize.encap_len
   ```

#### 2.1.4 기대 결과

- 프레임 50의 데이터 28바이트는 `color` 9, 틈 3, `x` 4, `y` 4, `shapesize` 4, 원소 수 4로 나뉘고, 값은 구독 로그와 같다. 마지막 `00 00 00 00`은 비어 있는 다섯째 멤버다. 원소가 없어도 원소 수 4바이트는 있다.
- 프레임 32는 벤더가 달라도 같은 배치다. 틈의 바이트는 두 벤더 모두 0이다.
- 프레임 268은 데이터 9바이트에 끝 패딩 3바이트가 붙고 options가 3이다. 틈 (멤버 사이, 정렬 때문)과 끝 패딩 (페이로드 끝, 4의 배수 때문)은 다른 것이다.

#### 2.1.5 확인 질문

- 발행자의 색이 `BLUE` 대신 `RED`였다면 `x`는 어느 오프셋에서 시작하고 틈은 몇 바이트인가?
- 데이터가 13바이트인 페이로드의 options 값은 얼마이고 본문은 몇 바이트인가?

#### 2.1.6 이 chapter에서 답하지 않는 것

- dispose가 왜 key만 싣는지, key가 여럿일 때의 순서, `PID_KEY_HASH` -> chapter13
- 페이로드가 DATA 서브메시지의 어디에서 시작하는지 (`octetsToInlineQos`, inline QoS) -> chapter7
- `-x 2`일 때 데이터 앞에 붙는 DHEADER -> chapter16

## 3. 규격 읽기

### 3.1 세 규격의 역할

CDR의 원본은 CORBA 규격이다 (CORBA 3.3 Part 2 Sec. 9.3). DDS-XTypes가 그것을 XCDR1의 PLAIN_CDR로 받아들이고 문자열 인코딩과 끝 패딩 같은 빈 곳을 채웠으며, RTPS가 페이로드 맨 앞의 캡슐화 헤더를 정했다. 읽는 곳은 다음과 같다.

| 무엇 | 규격 |
|---|---|
| 정렬, 정수와 부동소수점, struct / string / sequence / array / enum / union의 바이트 | CORBA 3.3 Part 2 Sec. 9.3.1 ~ 9.3.2 |
| 원시 타입의 크기와 정렬 표, 문자열은 UTF-8, enum의 크기, 끝 패딩은 0, appendable은 XCDR1에서 final과 같다 | DDS-XTypes 1.3 Sec. 7.4.1.1 (Table 31, 32), Sec. 7.4.3.5.3 |
| 캡슐화 헤더 4바이트, 정렬 기준점, 식별자 값 | RTPS 2.5 Sec. 10.2, 10.5 (표 10.3), DDS-XTypes 1.3 Sec. 7.6.3.1.2 (표 60) |

### 3.2 정렬

크기가 n바이트인 원시 값은 오프셋이 n의 배수인 곳에서 시작한다. n은 1, 2, 4, 8이다. 그 앞에 필요한 만큼만 [틈](../glossary.md#직렬화) (alignment gap)을 둔다 (CORBA 3.3 Part 2 Sec. 9.3.1.1, Table 9.1). CPU가 정렬된 주소에서 값을 바로 읽게 하려는 규칙이고, 구성 타입 (struct, sequence, array)은 자기 정렬이 없다. 안에 든 원시 값의 정렬만 따른다 (Sec. 9.3.2.1). 그래서 struct 안의 struct는 바깥 멤버와 같은 줄에 그대로 이어지고, sequence의 원소도 하나씩 자기 정렬을 따른다.

오프셋을 어디서부터 세는가가 중요하다. RTPS 2.5 Sec. 10.2는 캡슐화 헤더 4바이트 뒤에서 CDR 스트림이 논리적으로 다시 시작하며 처음에 틈이 없어야 한다고 적었고, 각주에서 2.4 이전 판은 이 위치가 분명하지 않았다고 밝힌다. DDS-XTypes 1.3의 직렬화 규칙 (Sec. 7.4.3.5.3 규칙 (1))은 기준점을 식별자와 options 사이에서 0으로 두는 것으로 읽혀 어느 읽기로도 헤더 뒤가 되지 않는데, 이 책은 이것을 표기 문제로 보고 [추정] RTPS 2.5를 따른다. 두 벤더도 헤더 뒤에서 센다 (6절). Lab 6-1의 세 프레임은 8바이트 값이 없어 어느 쪽으로 세어도 같은 바이트가 나온다. 차이는 8바이트로 정렬하는 값 (int64, uint64, float64, float128)이 있는 타입에서 난다.

| IDL | 오프셋 (헤더 뒤 기준) |
|---|---|
| `struct { int32 a; int64 b; }` | `a` 0, 틈 4, `b` 8. 합계 16 |
| `struct { int64 b; int32 a; }` | `b` 0, `a` 8. 합계 12 |
| `struct { octet c; int16 s; }` | `c` 0, 틈 1, `s` 2. 합계 4 |

XCDR1의 정렬 상한은 8이다 (DDS-XTypes 1.3 Sec. 7.4.3.2 Table 37). XCDR2는 상한이 4라 8바이트 값도 4의 배수면 된다 (chapter16). 틈의 값은 규격이 정하지 않았다 (CORBA Sec. 9.3.1.1은 "undefined"라고 적었다). 캡처의 두 벤더는 0을 쓰고, mdds도 0을 쓴다.

### 3.3 엔디안

정수와 부동소수점은 리틀 엔디언과 빅 엔디언 두 가지로 쓸 수 있다 (CORBA 3.3 Part 2 Sec. 9.3.1.2, Figure 9.1). 어느 쪽인지는 캡슐화 헤더의 식별자가 알리고 (3.4절), 받는 쪽은 자기 CPU와 다르면 바이트 순서를 바꾼다. 캡처의 두 벤더는 모두 리틀 엔디언으로 보냈다. 정수는 2의 보수이므로 int32의 -1은 `ff ff ff ff`이고, 이것은 엔디안과 무관하게 같다. 부동소수점은 IEEE 754의 비트를 그대로 쓴다 (Sec. 9.3.1.3).

### 3.4 캡슐화 헤더

페이로드는 캡슐화 헤더 4바이트로 시작한다 (RTPS 2.5 Sec. 10.2). 앞 2바이트는 식별자 (RepresentationIdentifier. Wireshark의 encapsulation kind. [용어 정리](../glossary.md#프로토콜))로 어떤 규칙과 엔디안으로 썼는지 알리고, 뒤 2바이트는 options다.

| 식별자 | 값 | 뜻 |
|---|---|---|
| CDR_BE | `00 00` | XCDR1 PLAIN_CDR, 빅 엔디언 |
| CDR_LE | `00 01` | XCDR1 PLAIN_CDR, 리틀 엔디언. 캡처의 사용자 DATA가 이것이다 |
| PL_CDR_BE | `00 02` | XCDR1 ParameterList (PID와 길이가 붙은 멤버 목록), 빅 엔디언 |
| PL_CDR_LE | `00 03` | 같은 것, 리틀 엔디언. SPDP / SEDP의 DATA가 이것이다 (chapter7, 9) |

값은 RTPS 2.5 표 10.3과 DDS-XTypes 1.3 표 60이 같다. XCDR2의 식별자 (CDR2, D_CDR2, PL_CDR2의 BE / LE)도 있는데, 규격 안에서 값이 두 가지로 적혀 있다 (RTPS 표 10.3과 XTypes 표 39는 0x10 ~ 0x15, XTypes 표 60은 0x06 ~ 0x0b). 캡처에서 본 것은 표 60의 값이다 (Lab 4-4의 `D_CDR2_LE (0x0009)`). chapter16에서 다룬다. 첫 바이트가 0xC0 ~ 0xFF인 식별자는 벤더가 정하는 것이다 (XTypes Sec. 7.6.3.1.2). 이 식별자와 chapter5의 QoS 값 `DataRepresentationId` (XCDR = 0, XCDR2 = 2)는 다른 번호 공간이다. QoS 값은 "어느 판을 쓰겠다"이고 식별자는 "이 페이로드를 어느 형식으로 썼다"다.

options는 DDS-XTypes 1.3이 뜻을 정했다 (Sec. 7.6.3.1.2). 둘째 바이트의 하위 2비트가 끝 패딩 바이트 수 (0 ~ 3)이고, 받는 쪽은 이 값으로 데이터가 정확히 어디서 끝나는지 정한다. 나머지 비트에는 규격이 정한 뜻이 없다. 끝 패딩은 RTPS가 다음 서브메시지를 4바이트 경계에서 시작하게 하려고 붙이는 것이고, 그 값은 0이어야 한다 (Sec. 7.4.1.1). Lab 6-1의 프레임 268이 그 예다.

### 3.5 타입별 규칙

| 타입 | 바이트 | 정렬 | 출처 |
|---|---|---|---|
| octet, uint8, int8, char, boolean | 1 (boolean은 0 또는 1. char는 해석하지 않는 1바이트) | 1 | CORBA 9.3.1.4 ~ 9.3.1.6, XTypes Table 31 |
| int16, uint16 | 2 | 2 | CORBA 9.3.1.2 |
| int32, uint32, float32 | 4 | 4 | CORBA 9.3.1.2, 9.3.1.3 |
| int64, uint64, float64 | 8 | 8 (XCDR2는 4) | CORBA 9.3.1.2, XTypes Table 31. XCDR2는 Sec. 7.4.2, Table 37 |
| string | uint32 길이 (NUL 포함 바이트 수), UTF-8 글자, NUL. 빈 문자열은 길이 1. `string<128>`의 128은 와이어에 나타나지 않는다 | 길이 필드가 4 | CORBA 9.3.2.7, XTypes 7.4.1.1.2, 규칙 (3) |
| sequence<T> | uint32 원소 수, 원소들 (각각 자기 정렬) | 원소 수가 4 | CORBA 9.3.2.5, XTypes 규칙 (11), (13) |
| T[N] (array) | 원소들만. 길이 없음 | 원소의 정렬 | CORBA 9.3.2.4, XTypes 규칙 (8), (10) |
| enum | 기본은 int32 (첫 식별자가 0, 선언 순서). `@bit_bound`가 있으면 8 / 16비트, `@value`가 있으면 그 값 | 기본 4 | CORBA 9.3.2.6, XTypes 7.4.1.1.3 Table 32, Sec. 7.3.1.2.1.5 |
| union | 판별값 (boolean, 문자, 정수, enum 가운데 하나), 그 뒤 선택된 멤버. 어느 멤버도 고르지 않는 판별값이 있을 수 있고, 명시된 case가 없는 값은 default 멤버를 고른다 | 각각 자기 정렬 | CORBA 9.3.2.3, XTypes Sec. 7.2.2.4.4.3, 규칙 (26), IDL 4.2 Sec. 7.4.1 |
| struct | 멤버를 선언 순서대로, 각각 자기 타입의 규칙으로. struct 자체의 정렬은 없다 | 멤버의 정렬 | CORBA 9.3.2.1, 9.3.2.2 |
| typedef | 와이어에 흔적이 없다. 원래 타입과 같은 바이트 | | XTypes 규칙 (7) |
| `@appendable` struct | XCDR1에서는 final과 같은 바이트 (ShapeType이 그렇다). XCDR2에서는 DHEADER가 앞에 붙는다 (chapter16) | | XTypes 규칙 (29) |

XCDR1에 있지만 이 chapter가 다루지 않는 것은 `@optional` 멤버 (chapter16), `wchar` / `wstring`, `map`, `bitmask`다. 쓰는 타입이 나올 때 더한다.

## 4. 구현

### 4.1 설계

`mdds-cdr`는 바이트 버퍼만 다루고 소켓, 시계, 스레드를 모른다. 모듈은 넷이다.

| 모듈 | 하는 일 |
|---|---|
| `writer` | `CdrWriter`. 정렬과 엔디안을 아는 쓰기 커서. 원시 타입, 바이트, 문자열, sequence를 쓴다 |
| `reader` | `CdrReader`. 같은 것의 읽기 쪽. 길이 필드를 믿지 않고 어떤 입력에도 패닉하지 않는다 |
| `serialize` | trait `CdrSerialize` / `CdrDeserialize`와 원시 타입, `String`, `Vec<T>`, `[T; N]`의 구현 |
| `encapsulation` | 캡슐화 헤더 (`RepresentationId`, `EncapsulationHeader`)와 끝 패딩. `to_payload` / `from_payload` |

설계의 요점은 넷이다.
- **캡슐화 헤더는 writer 밖에 있다.** `CdrWriter`는 데이터만 쓰고 정렬 기준은 자기 버퍼의 0이다. `to_payload`가 헤더, 데이터, 끝 패딩을 조립한다. 그러면 3.2절의 "헤더 뒤에서 다시 센다"가 별도 처리 없이 성립한다. 기준점을 다시 세는 다른 자리 (chapter7)에서도 그 범위의 슬라이스 위에 새 reader를 만들면 된다.
- **엔디안과 XCDR 버전은 커서를 만들 때 정한다.** `XcdrVersion::V1`은 정렬 상한 8, `V2`는 4다. 이 chapter는 V1만 쓰지만 V2가 chapter13에서 먼저 필요하다.
- **쓰기도 `Result`다.** `Vec`에 쓰는 일은 실패하지 않지만 Rust 값이 CDR로 표현될 수 없는 경우가 있다. 안에 NUL이 든 문자열, u32를 넘는 길이다.
- **읽기는 길이 필드를 믿지 않는다.** 와이어의 원소 수나 문자열 길이가 남은 바이트보다 크면 할당하지 않고 바로 오류다. 슬라이스는 `get`으로만 잘라 인덱스 패닉이 없다.

공개 API는 다음과 같다. `CdrError`는 오류 enum이다.

| 항목 | 시그니처 |
|---|---|
| 엔디안, 버전 | `enum Endianness { Big, Little }`, `enum XcdrVersion { V1, V2 }`, `XcdrVersion::max_align(self) -> usize` |
| writer | `CdrWriter::new(Endianness, XcdrVersion)`, `endianness()`, `version()`, `position() -> usize`, `as_bytes() -> &[u8]`, `into_bytes() -> Vec<u8>`, `align(n)`, `write_u8(u8)` ... `write_i64(i64)`, `write_f32`, `write_f64`, `write_bool`, `write_bytes(&[u8])`, `write_string(&str) -> Result<(), CdrError>`, `write_sequence<T: CdrSerialize>(&[T]) -> Result<..>`, `write<T: CdrSerialize + ?Sized>(&T) -> Result<..>` |
| reader | `CdrReader::new(&[u8], Endianness, XcdrVersion)`, `endianness()`, `version()`, `position() -> usize`, `remaining() -> usize`, `align(n) -> Result<..>`, `read_u8() -> Result<u8, CdrError>` ... `read_f64`, `read_bool`, `read_bytes(n) -> Result<&[u8], ..>`, `read_string() -> Result<String, ..>`, `read_sequence<T: CdrDeserialize>() -> Result<Vec<T>, ..>`, `read<T: CdrDeserialize>() -> Result<T, ..>` |
| trait | `CdrSerialize::serialize(&self, &mut CdrWriter) -> Result<(), CdrError>`, `CdrDeserialize::deserialize(&mut CdrReader<'_>) -> Result<Self, CdrError>` |
| 헤더 | `RepresentationId(pub [u8; 2])`와 연관 상수 `RepresentationId::CDR_BE`, `CDR_LE`, `PL_CDR_BE`, `PL_CDR_LE`, XCDR2 여섯 개. `EncapsulationHeader { representation, options: [u8; 2] }`, `LEN` (= 4), `for_data_len(id, data_len)`, `padding_for(data_len) -> usize`, `padding() -> usize`, `to_bytes() -> [u8; 4]`, `split(&[u8]) -> Result<(Self, &[u8]), ..>`, `data(&self, rest) -> Result<&[u8], ..>` |
| 페이로드 | `to_payload<T: CdrSerialize + ?Sized>(&T, Endianness) -> Result<Vec<u8>, CdrError>`, `from_payload<T: CdrDeserialize>(&[u8]) -> Result<T, CdrError>` |

오류는 enum 하나다. 변이마다 그 reader가 읽던 버퍼 안의 위치 (`offset`)를 싣는다. `from_payload`에서는 캡슐화 헤더 뒤의 데이터 기준이라 Wireshark의 리프 바이트와 바로 대조할 수 있다.

| 변이 | 언제 |
|---|---|
| `UnexpectedEnd { offset, needed, remaining }` | 읽을 바이트가 모자란다. `needed`는 틈과 값을 합친 수이고, 오류가 나도 읽기 위치는 움직이지 않는다 |
| `InvalidBool { offset, value }` | boolean이 0도 1도 아니다. 규격은 받는 쪽 동작을 정하지 않았고 mdds가 거부하기로 한 것이다 |
| `InvalidUtf8 { offset }` | 문자열이 UTF-8이 아니다 |
| `MissingNul { offset }` | 문자열 길이가 0이거나 마지막 바이트가 NUL이 아니다 |
| `InvalidEnum { offset, value }` | enum 값이 선언된 식별자 가운데 없다 |
| `StringContainsNul`, `LengthOverflow` | 쓰기 쪽. Rust 값이 CDR로 표현되지 않는다 |
| `UnsupportedRepresentation(RepresentationId)` | 식별자가 CDR_BE / CDR_LE가 아니다 (PL_CDR은 chapter7, XCDR2는 chapter16) |
| `InvalidPadding { padding, len }` | options의 끝 패딩 수가 본문 길이보다 크다 |

### 4.2 Lab 6-2: mdds-cdr 쓰기

#### 4.2.1 목표

4.1절의 설계를 `my-code/crates/mdds-cdr/src/`에 옮기고 단위 테스트로 확인한다. 완성본은 [`code/crates/mdds-cdr/src/`](code/crates/mdds-cdr/src/)에 있다. 진행이 어려우면 완성본과 비교한다.

#### 4.2.2 준비

1.1절대로 `my-code/`에서 `cargo test -p mdds-cdr`가 테스트 0개로 통과하는 상태다.

#### 4.2.3 절차

단계마다 작은 확인을 둔다. 테스트는 모듈 안 `#[cfg(test)] mod tests`에 쓴다.

1. `lib.rs`에 `Endianness`, `XcdrVersion` (`max_align`은 `const fn`), `CdrError` (4.1절의 변이. `Display`와 `std::error::Error`도 구현한다. 메시지 문구는 자유다)를 선언하고 모듈 넷을 `pub mod`로 둔다. 모듈 파일은 비어 있어도 `cargo build -p mdds-cdr`가 통과해야 한다. 자주 쓰는 이름을 다시 내보내는 `pub use`는 5단계가 끝난 뒤에 더한다 (완성본: `encapsulation::{EncapsulationHeader, RepresentationId, from_payload, to_payload}`, `reader::CdrReader`, `serialize::{CdrDeserialize, CdrSerialize}`, `writer::CdrWriter`).
2. `writer.rs`에 `CdrWriter { buf: Vec<u8>, endianness, version }`를 쓴다. 핵심은 `align`이다. 정렬 n을 버전의 상한으로 자르고, 틈을 0으로 채운다.
   `code/crates/mdds-cdr/src/writer.rs`
   ```rust
   pub fn align(&mut self, n: usize) {
       debug_assert!(n > 0);
       let n = n.min(self.version.max_align());
       let pad = (n - self.buf.len() % n) % n;
       self.buf.resize(self.buf.len() + pad, 0);
   }
   ```
   정수 쓰기 여섯 개 (`u16` ~ `i64`)는 `align(size_of::<T>())` 뒤에 `to_be_bytes` / `to_le_bytes`를 엔디안에 따라 붙이는 것이라 매크로 하나로 만든다. 매크로가 낯설면 메서드 여섯 개를 직접 써도 된다.
   ```rust
   macro_rules! write_int {
       ($name:ident, $t:ty) => {
           pub fn $name(&mut self, v: $t) {
               self.align(core::mem::size_of::<$t>());
               match self.endianness {
                   Endianness::Big => self.buf.extend_from_slice(&v.to_be_bytes()),
                   Endianness::Little => self.buf.extend_from_slice(&v.to_le_bytes()),
               }
           }
       };
   }
   ```
   `u8`, `i8`, `bool`은 정렬이 없다. `f32` / `f64`는 `to_bits`로 정수로 바꿔 쓴다. 문자열은 3.5절의 규칙 그대로다.
   ```rust
   pub fn write_string(&mut self, s: &str) -> Result<(), CdrError> {
       if s.as_bytes().contains(&0) {
           return Err(CdrError::StringContainsNul);
       }
       let len = u32::try_from(s.len() + 1).map_err(|_| CdrError::LengthOverflow)?;
       self.write_u32(len);
       self.buf.extend_from_slice(s.as_bytes());
       self.buf.push(0);
       Ok(())
   }
   ```
   `write_sequence`는 원소 수를 쓴 뒤 원소마다 `item.serialize(self)?`를 부른다 (trait는 4단계에서 만든다. 먼저 trait 선언만 `serialize.rs`에 두어도 된다). 확인: `u8` 하나 뒤에 `u32`를 쓰면 8바이트이고 1 ~ 3번 바이트가 0이다.

   | 테스트 | 확인하는 것 |
   |---|---|
   | `u8_then_each_primitive_pads_to_its_size` | u8 뒤에 u16 / i16 / u32 / f32 / u64 / f64를 쓰면 크기 - 1의 틈 |
   | `v2_caps_alignment_at_four` | V2에서는 u8 뒤 u64의 틈이 3 (V1은 7) |
   | `little_and_big_endian_bytes` | `0x01020304`와 -1의 두 엔디안 바이트 (CORBA Figure 9.1) |
   | `string_has_length_including_nul`, `string_with_interior_nul_is_rejected` | `BLUE`는 9바이트, 빈 문자열은 `01 00 00 00 00`, 안의 NUL은 오류 |
   | `sequence_has_count_then_elements`, `sequence_elements_are_aligned_each` | 원소 수 뒤 원소. `sequence<int64>`는 원소 수 4 + 틈 4 + 8 |
   | `padding_bytes_are_zero_even_after_other_data`, `align_writes_nothing_when_already_aligned`, `bool_is_one_byte` | 틈은 0, 이미 맞는 위치에서는 아무것도 쓰지 않음, boolean은 1바이트 |
3. `reader.rs`에 `CdrReader<'a> { buf: &'a [u8], pos, endianness, version }`를 쓴다. 모든 읽기는 비공개 `take`를 지난다. 슬라이스를 `get`으로 자르므로 모자라면 패닉 대신 오류다.
   `code/crates/mdds-cdr/src/reader.rs`
   ```rust
   fn take(&mut self, n: usize) -> Result<&'a [u8], CdrError> {
       let end = self.pos.checked_add(n);
       match end.and_then(|end| self.buf.get(self.pos..end)) {
           Some(bytes) => {
               self.pos += n;
               Ok(bytes)
           }
           None => Err(CdrError::UnexpectedEnd {
               offset: self.pos,
               needed: n,
               remaining: self.remaining(),
           }),
       }
   }
   ```
   정수 읽기는 틈과 값을 합쳐 미리 검사하는 `take_aligned(n)`을 거친다. 모자라면 `needed`가 틈 + 값이고 읽기 위치는 그대로다. 그 뒤 바이트를 `from_be_bytes` / `from_le_bytes`로 바꾸는 매크로는 writer와 같은 모양이다. `align`은 틈을 `take`로 건너뛴다 (값은 검사하지 않는다). 문자열은 길이 0과 NUL 없음을 거부한다.
   ```rust
   pub fn read_string(&mut self) -> Result<String, CdrError> {
       let offset = self.pos;
       let len = self.read_u32()? as usize;
       if len == 0 {
           return Err(CdrError::MissingNul { offset });
       }
       let bytes = self.take(len)?;
       let (body, nul) = bytes.split_at(len - 1);
       if nul != [0] {
           return Err(CdrError::MissingNul { offset });
       }
       core::str::from_utf8(body)
           .map(str::to_owned)
           .map_err(|_| CdrError::InvalidUtf8 { offset })
   }
   ```
   `read_sequence`는 미리 할당하는 메모리를 남은 바이트 수 이하로 묶는다 (`Vec::with_capacity(len.min(remaining / size_of::<T>()))`). 원소 수가 거짓이어도 입력보다 크게 할당하지 않는다. 확인: `[1, 2]`에서 `read_u32`는 `UnexpectedEnd { offset: 0, needed: 4, remaining: 2 }`다.

   | 테스트 | 확인하는 것 |
   |---|---|
   | `reads_back_what_writer_wrote_in_both_endiannesses` | 모든 타입과 빈 문자열, 빈 sequence, `sequence<uint16>`을 쓰고 읽으면 같다 (BE, LE) |
   | `v2_reads_u64_at_offset_four` | V2에서는 u8 뒤 u64가 오프셋 4에서 읽힌다 |
   | `unexpected_end_reports_offset_needed_and_remaining` | 모자랄 때의 오류 값. u8 뒤 u32는 틈 3 + 값 4로 `needed: 7`. 오류 뒤에도 위치는 그대로 |
   | `alignment_gap_is_skipped_but_not_checked` | 틈이 `ff`여도 읽힌다 |
   | `bool_other_than_zero_or_one_is_invalid`, `string_errors` | 2는 `InvalidBool`. 길이 0, NUL 없음, UTF-8 아님, 길이 0xffffffff (할당 없이 오류) |
   | `huge_sequence_count_is_an_error_not_a_panic` | 원소 수 0xffffffff |
   | `nan_bits_survive_round_trip` | NaN의 비트가 보존된다 |
4. `serialize.rs`에 trait 둘을 선언하고 구현을 모은다.
   `code/crates/mdds-cdr/src/serialize.rs`
   ```rust
   pub trait CdrSerialize {
       fn serialize(&self, w: &mut CdrWriter) -> Result<(), CdrError>;
   }

   pub trait CdrDeserialize: Sized {
       fn deserialize(r: &mut CdrReader<'_>) -> Result<Self, CdrError>;
   }
   ```
   원시 타입 11개 (`u8`, `i8`, `bool`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `f32`, `f64`)는 writer / reader의 메서드를 부르는 매크로로 만든다. Rust의 `char` (4바이트)와 `usize`에는 구현하지 않는다 (IDL의 char는 1바이트 `u8`이다). `str`과 `String`은 `write_string`, `&T`는 `T`로 넘기고, `Vec<T>`는 `write_sequence` / `read_sequence`, `[T; N]`은 길이 없이 원소만이다 (읽을 때는 `Vec`으로 모은 뒤 `try_into`). `[T]`에는 구현하지 않는다. 슬라이스가 sequence인지 array인지 타입만으로 알 수 없어서다. struct, enum, union은 사용자가 멤버를 선언 순서대로 나열해 구현한다. 테스트 모듈에 struct 둘 (`Small { a: u8, b: i32 }`, `Outer { x: u8, inner: Inner { y: u16 } }`)과 enum 하나 (`Color`, `i32`)를 구현해 둔다. 3.2절 첫 행의 테스트는 다음과 같다.
   ```rust
   #[test]
   fn i32_then_i64_pads_four_from_data_start() {
       let mut w = CdrWriter::new(Endianness::Little, XcdrVersion::V1);
       w.write(&1i32).unwrap();
       w.write(&2i64).unwrap();
       assert_eq!(w.position(), 16);
       assert_eq!(&w.as_bytes()[4..8], [0, 0, 0, 0]);
   }
   ```

   | 테스트 | 확인하는 것 |
   |---|---|
   | `struct_members_in_declaration_order_each_aligned` | `Small { 1, 2 }`는 `01 00 00 00 02 00 00 00` |
   | `nested_struct_has_no_own_alignment` | `Outer`는 `01 00 02 00`. 안의 u16은 2의 배수면 된다 |
   | `i32_then_i64_pads_four_from_data_start` | 위의 테스트 |
   | `array_has_no_length_prefix` | `[1u16, 2]`는 4바이트, 같은 값의 `Vec`은 8바이트 |
   | `vec_of_strings_round_trips`, `str_and_string_serialize_the_same` | 문자열 sequence, `&str`과 `String` |
   | `enum_is_i32_and_unknown_value_is_rejected` | 7은 `InvalidEnum { offset: 0, value: 7 }` |
5. `encapsulation.rs`에 식별자와 헤더, 페이로드 함수를 쓴다. `RepresentationId`는 `[u8; 2]`를 감싼 타입이고 3.4절의 값이 연관 상수다. `EncapsulationHeader::split`은 페이로드를 헤더와 본문으로 나누되 식별자를 가리지 않는다 (chapter9의 PL_CDR_LE도 이것으로 헤더를 읽는다). `data`는 options의 패딩 수만큼을 본문 끝에서 뗀다. `for_data_len`은 데이터 길이에서 패딩 수를 계산해 options에 싣는다. 지원 여부는 `from_payload`만 판단한다.
   `code/crates/mdds-cdr/src/encapsulation.rs`
   ```rust
   pub fn to_payload<T: CdrSerialize + ?Sized>(
       value: &T,
       endianness: Endianness,
   ) -> Result<Vec<u8>, CdrError> {
       let mut w = CdrWriter::new(endianness, XcdrVersion::V1);
       value.serialize(&mut w)?;
       let data = w.into_bytes();
       let padding = EncapsulationHeader::padding_for(data.len());
       let representation = match endianness {
           Endianness::Big => RepresentationId::CDR_BE,
           Endianness::Little => RepresentationId::CDR_LE,
       };
       let mut payload = Vec::with_capacity(EncapsulationHeader::LEN + data.len() + padding);
       payload.extend_from_slice(
           &EncapsulationHeader::for_data_len(representation, data.len()).to_bytes(),
       );
       payload.extend_from_slice(&data);
       payload.resize(payload.len() + padding, 0);
       Ok(payload)
   }
   ```
   ```rust
   pub fn from_payload<T: CdrDeserialize>(payload: &[u8]) -> Result<T, CdrError> {
       let (header, rest) = EncapsulationHeader::split(payload)?;
       let endianness = match header.representation {
           RepresentationId::CDR_BE => Endianness::Big,
           RepresentationId::CDR_LE => Endianness::Little,
           other => return Err(CdrError::UnsupportedRepresentation(other)),
       };
       let data = header.data(rest)?;
       let mut r = CdrReader::new(data, endianness, XcdrVersion::V1);
       T::deserialize(&mut r)
   }
   ```
   값 뒤에 바이트가 남아도 `from_payload`는 오류로 보지 않는다 (4.3절). 확인: `to_payload("BLUE", Little)`은 16바이트이고 앞 4바이트가 `00 01 00 03`이다 (Lab 6-1의 프레임 268과 같은 모양).

   | 테스트 | 확인하는 것 |
   |---|---|
   | `string_payload_is_padded_to_four_and_options_carry_the_count` | `BLUE`는 패딩 3과 options 3, u32는 패딩 0 |
   | `from_payload_strips_padding_and_reads_the_value`, `big_endian_payload_round_trips` | 패딩을 뗀 데이터 9바이트. BE 왕복 |
   | `trailing_bytes_after_the_value_are_not_an_error` | 값 뒤에 4바이트가 남아도 읽힌다 |
   | `header_shorter_than_four_bytes_is_unexpected_end` | 3바이트 페이로드 |
   | `pl_cdr_and_xcdr2_are_unsupported_here` | `00 03`, `00 09`, `c0 01`은 모두 `UnsupportedRepresentation` |
   | `padding_larger_than_data_is_invalid`, `upper_option_bits_are_ignored` | 패딩 3에 본문 1은 오류. options의 상위 비트는 무시 |
   | `representation_id_endianness` | 식별자가 알리는 엔디안 (XML은 없음) |
6. `my-code/`에서 검사를 실행한다.
   ```bash
   cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test -p mdds-cdr
   ```

#### 4.2.4 기대 결과

`cargo test -p mdds-cdr`의 단위 테스트가 모두 통과한다 (완성본은 34개. writer 10, reader 8, serialize 7, encapsulation 9). clippy 경고가 없다. 테스트 수가 같을 필요는 없고 표의 항목이 있는지만 본다.

#### 4.2.5 확인 질문

- `struct { octet a; uint32 b; }`와 `struct { uint32 b; octet a; }`의 값 하나를 `to_payload`로 만들면 각각 데이터는 몇 바이트이고, 끝 패딩은 몇 바이트이며, options는 얼마인가?
- 문자열 `AB`의 길이 필드 값은 얼마이고 왜 그런가?

#### 4.2.6 실패하면

- `align`에서 `n`이 0이면 나눗셈 패닉이다. 정렬은 항상 1 이상이고, `max_align`도 4 또는 8이다. 완성본은 `debug_assert!`로 막는다.
- 테스트의 `fn(&mut CdrWriter)` 배열처럼 복잡한 타입은 clippy가 `type_complexity`로 거부한다. `type Write = fn(&mut CdrWriter);`처럼 별칭을 둔다. `len % 2 == 0` 같은 식은 `is_multiple_of(2)`로 쓰라고 한다.
- `read_bytes`가 `&'a [u8]`를 돌려주려면 `take`도 `&'a [u8]`를 돌려주어야 한다. `&self`의 수명이 아니라 버퍼의 수명 `'a`다.
- `[T; N]`의 `try_into`가 실패하는 경우는 길이가 다를 때뿐이고 여기서는 생기지 않는다. `unwrap_or_else(|_| unreachable!())`로 적는다.
- `diff -r`에서 차이가 나면 먼저 `cargo fmt`를 실행한다. 형식 차이가 사라진 뒤 남는 것이 내용 차이다.

### 4.3 설계 메모

- **왜 serde를 쓰지 않는가.** serde로 CDR을 만들 수는 있다. Serializer는 값의 타입을 알므로 정렬도 구현할 수 있다. 쓰지 않는 이유는 셋이다. DDS-XTypes가 멤버에 붙이는 정보 (`@id`, `@key`, `@optional`, `@must_understand`, 확장성)가 serde의 데이터 모델에 없어 별도의 속성 체계가 필요하고, key 멤버만 직렬화하기 (chapter13)는 같은 타입을 두 번째 방식으로 쓰는 일이며, 바이트 하나하나를 직접 쓰는 것이 이 책의 목적이다. 비용은 serde 생태계 (파생 매크로, 다른 형식)를 재사용하지 못하는 것이다. XCDR1 final 타입만 쓰는 응용 (예: ROS 2 메시지)이라면 serde 위의 CDR 구현으로 충분하다.
- **왜 derive 매크로를 아직 만들지 않는가.** ShapeType의 구현은 멤버마다 한 줄이라 손으로 쓰는 편이 규칙을 그대로 보여 준다. 매크로를 만들려면 proc-macro crate와 `syn`, `quote` 의존이 생기고 독자가 Rust 매크로 작성법을 배워야 한다. 멤버가 많은 타입이 여럿 생기면 (chapter16의 타입 정보 타입들) 반복이 부담이 되는데, 그때 의존 없는 `macro_rules!`로 할지 chapter17의 IDL 컴파일러가 코드를 생성하게 할지 정한다.
- **trait에 넣지 않은 것.** key 멤버만 쓰기, 타입의 최대 직렬화 크기, 타입 이름은 `CdrSerialize`에 없다. chapter11 (타입 이름 매칭)과 chapter13 (key)이 위층의 trait로 더한다. `mdds-cdr`는 인코딩만 맡는다.
- **왜 값 뒤에 남은 바이트를 오류로 보지 않는가.** 끝 패딩이 있고 (options로 떼지만, 패딩 비트를 채우지 않는 구현이 있을 수 있다), XCDR1의 appendable 타입은 DHEADER가 없어 멤버를 덧붙인 새 버전의 샘플이 올 수 있다. 그런 샘플을 앞부분만 읽을 여지를 남긴다. 실제로 허용되는지는 chapter16의 타입 호환 규칙이 정한다. 대신 `CdrReader::remaining()`을 두어 테스트에서 0을 단언한다.

## 5. 벤더와 맞춰 보기

### 5.1 Lab 6-3: 캡처의 바이트를 골든 테스트로

#### 5.1.1 목표

ShapeType을 `CdrSerialize` / `CdrDeserialize`로 구현하고, Lab 6-1의 세 프레임과 Lab 4-4의 xcdr2 프레임을 골든 테스트 (벤더가 실제로 보낸 바이트를 정답으로 고정해 두고 비교하는 테스트)로 적는다.

#### 5.1.2 준비

Lab 6-2를 마친 `my-code/`에서 `crates/mdds-cdr/tests/shape_type.rs`를 새로 만든다. `tests/`의 파일은 crate의 공개 API만 쓰는 통합 테스트다. 첫 줄은 다음과 같다.
```rust
use mdds_cdr::{
    CdrDeserialize, CdrError, CdrReader, CdrSerialize, CdrWriter, EncapsulationHeader, Endianness,
    RepresentationId, XcdrVersion, from_payload, to_payload,
};
```

바이트는 Lab 6-1에서 읽은 것을 쓴다. 프레임마다 Lab 6-1 4단계의 tshark 명령 (프레임 번호와 캡처 파일만 바꾼다)으로 데이터를 출력하고, 앞에 캡슐화 헤더를 붙인다. 프레임 50과 32는 `0001 0000`, 프레임 268은 `0001 0003`이다. xcdr2의 프레임 41은 Lab 4-4의 바이트에 헤더 `0009 0000`을 붙인다. pcap을 직접 읽는 코드는 chapter7에 생기므로 여기서는 hex 리터럴로 둔다.

#### 5.1.3 절차

1. `ShapeType`을 선언하고 trait 둘을 구현한다. 멤버를 선언 순서대로 나열하는 것이 전부다. `string<128>`의 bound는 와이어에 없으므로 `String`이다.
   `code/crates/mdds-cdr/tests/shape_type.rs`
   ```rust
   #[derive(Debug, Clone, PartialEq, Eq)]
   struct ShapeType {
       color: String,
       x: i32,
       y: i32,
       shapesize: i32,
       additional_payload_size: Vec<u8>,
   }

   impl CdrSerialize for ShapeType {
       fn serialize(&self, w: &mut CdrWriter) -> Result<(), CdrError> {
           w.write(&self.color)?;
           w.write(&self.x)?;
           w.write(&self.y)?;
           w.write(&self.shapesize)?;
           w.write(&self.additional_payload_size)
       }
   }

   impl CdrDeserialize for ShapeType {
       fn deserialize(r: &mut CdrReader<'_>) -> Result<Self, CdrError> {
           Ok(ShapeType {
               color: r.read()?,
               x: r.read()?,
               y: r.read()?,
               shapesize: r.read()?,
               additional_payload_size: r.read()?,
           })
       }
   }
   ```
2. 헬퍼 둘을 쓴다. `blue(x, y)`는 색 `BLUE`, 크기 20, 빈 sequence인 ShapeType을 만든다. `hex`는 공백을 뺀 hex 문자열을 두 글자씩 바이트로 바꾼다.
   ```rust
   fn hex(s: &str) -> Vec<u8> {
       let digits: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
       assert!(digits.len().is_multiple_of(2), "hex 자릿수가 홀수다");
       digits
           .chunks(2)
           .map(|pair| u8::from_str_radix(core::str::from_utf8(pair).unwrap(), 16).unwrap())
           .collect()
   }
   ```
   프레임마다 상수를 두고 캡처 이름과 프레임 번호를 주석에 적는다. 프레임 50은 다음과 같다 (공백은 읽기 쉽게 넣은 것이다).
   ```rust
   // reliable-cyclone-pub.pcap 프레임 50 (Cyclone writer 0x00000202, seq 12), serializedData.
   // sub.log: Square BLUE 161 156 [20]
   const CYCLONE_FRAME_50: &str =
       "0001 0000  05000000 424c5545 00 000000  a1000000 9c000000 14000000 00000000";
   ```
3. 테스트 여섯 개를 쓴다.

   | 테스트 | 단언 |
   |---|---|
   | `cyclone_frame_50_serializes_to_the_capture_bytes` | `to_payload(&blue(161, 156), Little)`이 프레임 50의 32바이트와 같다 |
   | `fastdds_frame_32_deserializes_to_the_logged_sample` | 프레임 32가 `BLUE 194 66 20 []`로 읽히고, `remaining()`이 0이다 |
   | `dispose_frame_268_carries_only_the_key_with_three_padding_bytes` | 헤더의 `padding()`이 3, 데이터 9바이트, `read_string`이 `BLUE`. ShapeType으로 읽으면 `UnexpectedEnd { offset: 9, needed: 7, remaining: 0 }` |
   | `xcdr2_frame_41_is_not_supported_yet` | `UnsupportedRepresentation(RepresentationId::D_CDR2_LE)` |
   | `big_endian_bytes_follow_the_spec_figure` | 캡처에는 빅 엔디언이 없으므로 CORBA Figure 9.1로 손으로 만든 바이트와 같고, 되돌리면 같은 값 |
   | `every_truncated_prefix_is_an_error_not_a_panic` | 프레임 50의 모든 앞부분 (0 ~ 31바이트)이 패닉 없이 `Err` |

   첫 테스트는 다음과 같다.
   ```rust
   #[test]
   fn cyclone_frame_50_serializes_to_the_capture_bytes() {
       let payload = to_payload(&blue(161, 156), Endianness::Little).unwrap();
       assert_eq!(payload, hex(CYCLONE_FRAME_50));
       assert_eq!(payload.len(), 32);
   }
   ```
4. `my-code/`에서 `cargo test -p mdds-cdr`를 실행한다.

#### 5.1.4 기대 결과

여섯 테스트가 모두 통과한다. 특히 첫 테스트는 mdds가 만든 32바이트가 Cyclone이 보낸 것과 바이트 하나까지 같다는 뜻이고, 둘째는 Fast DDS가 보낸 것을 읽는다는 뜻이다. 셋째의 `UnexpectedEnd { offset: 9, needed: 7, remaining: 0 }`은 `color` 뒤 오프셋 9에서 `x`를 읽으려면 틈 3과 값 4로 7바이트가 필요한데 남은 것이 없다는 뜻이다. 오류의 `offset`이 Lab 6-1의 표와 맞는다. 넷째에서 0x0009가 `D_CDR2_LE`인 것은 Lab 4-4에서 보았다. 그 뒤의 DHEADER를 읽는 것은 chapter16이다.

#### 5.1.5 확인 질문

- ShapeType의 `shapesize`와 `additional_payload_size` 사이에 `double speed` 멤버가 더해진 타입으로 프레임 50과 같은 값을 보내면 데이터는 몇 바이트가 되는가? 틈은 어디에 몇 바이트 생기는가?

#### 5.1.6 실패하면

- 첫 테스트가 길이 32는 맞는데 바이트가 다르면 `x`, `y` 자리를 본다. 161과 156이 아니라 다른 샘플 (프레임 52는 164, 159)을 옮긴 것일 수 있다.
- `hex`에 공백이 아닌 글자가 섞이면 `from_str_radix`가 패닉하고, 자릿수가 홀수이면 `assert!`에 걸린다. 복사한 문자열에 `:`나 줄바꿈이 있는지, 바이트 하나를 빠뜨리지 않았는지 본다.
- 셋째 테스트의 `offset`이 9가 아니면 `color`를 읽은 뒤의 위치가 틀린 것이다. 길이 필드 4 + 5 = 9다. `needed`가 3이면 틈만 세고 값을 더하지 않은 것이다.

#### 5.1.7 이 chapter에서 답하지 않는 것

- pcap 파일에서 바로 `serializedData`를 꺼내 테스트하기 -> chapter7
- 프레임 268이 key만 싣는 이유와 key의 직렬화 규칙 -> chapter13

## 6. 두 벤더는 어떻게 하는가

두 벤더의 직렬화 코드를 고정한 커밋 (Cyclone DDS 11.0.1 `f2aaccb4`, Fast DDS 3.6.1과 그 직렬화 라이브러리 Fast-CDR 2.3.5)에서 읽은 결과다. 이 chapter에서 정한 것과 비교한다.

| 항목 | Cyclone DDS | Fast DDS | mdds |
|---|---|---|---|
| 정렬 기준점 | 캡슐화 헤더 바로 뒤. 받을 때 `dds_stream_normalize`가 데이터 시작을 0으로 둔다 | 캡슐화 헤더 바로 뒤. `Cdr::serialize_encapsulation()`과 `read_encapsulation()`이 끝에서 `reset_alignment()`로 기준점을 옮긴다 | writer / reader 버퍼의 0 (헤더 뒤) |
| options의 패딩 비트 | 쓴다. 4의 배수까지 0으로 채우고 그 수를 options에 적는다. 받을 때 그만큼 길이에서 뺀다 | 쓴다 (`Cdr::set_dds_cdr_options`). 패딩 바이트 자체는 RTPS 층이 DATA 서브메시지를 4바이트에 맞출 때 0으로 붙인다. 받을 때 끝을 그만큼 줄인다 | 쓴다. `to_payload`가 패딩을 붙이고 `from_payload`가 뗀다 |
| 모르는 식별자 | 허용 목록 (CDR, PL_CDR, CDR2, D_CDR2, PL_CDR2의 BE / LE) 밖이면 거부하고, 형식이 타입과 맞지 않아도 (final 타입에 PL_CDR) 거부한다. 받는 시점에 경고를 남기고 샘플을 버린다 | 첫 바이트가 0이 아니거나 형식이 목록 밖이면 `read_encapsulation()`이 예외를 던진다. 역직렬화는 응용이 read / take할 때 하고, 실패한 샘플은 지운다 | CDR_BE / CDR_LE만 받는다 (chapter7, 16에서 늘린다) |
| 타입을 알려 주는 방식 | idlc가 타입마다 opcode 배열을 만들고, 하나의 직렬화기 (`dds_cdrstream.c`)가 그것을 해석하며 바이트를 쓴다 | fastddsgen이 타입마다 C++ 함수 (`ShapeTypeCdrAux.ipp`의 `serialize` / `deserialize`)를 만든다 | 타입마다 손으로 쓴 trait 구현 (chapter17에서 생성) |
| 문자열의 NUL | 받을 때 길이 0과 NUL 없음을 오류로 본다 | NUL이 없어도 오류로 보지 않는다 | 길이 0과 NUL 없음은 오류 |
| 틈의 값 | 직렬화기가 0을 쓴다 | Fast-CDR은 틈을 건너뛰기만 한다. 버퍼를 0으로 초기화해 받으므로 캡처에서는 0으로 보인다 | 0을 쓴다 |

근거: Cyclone `src/core/cdr/src/dds_cdrstream.c`, `src/core/ddsc/src/dds_serdata_default.c`, Fast-CDR `src/cpp/Cdr.cpp`, `include/fastcdr/Cdr.h`, Fast DDS `src/cpp/rtps/messages/submessages/DataMsg.hpp`. 함수와 줄 번호는 [조사 노트](../../docs-for-agent/research/vendor-cdr-serialization.md)에 있다.

mdds에 적용하면 이렇다. 정렬 기준점과 options의 패딩 비트는 두 벤더와 mdds가 같으므로 3.2절과 3.4절의 읽기가 맞다. 문자열의 NUL 검사는 벤더마다 달라, mdds가 거부하는 바이트 (NUL 없는 문자열)를 Fast DDS는 받는다. 상호운용의 문제가 아니라 잘못된 바이트를 만났을 때의 동작 차이다. 틈을 0으로 쓰지 않아도 상호운용되는 것은 규격이 틈의 값을 정하지 않았기 때문이다. mdds도 읽을 때 틈의 값을 검사하지 않는다.

## 7. 스스로 확인하기

명령이나 수치를 외웠는지가 아니라 동작의 원리를 이해했는지 확인하는 질문이다. 먼저 스스로 답해 본 뒤 "해설"을 펼쳐 비교해 보라.

1. 동료가 묻는다. "Rust struct에 `#[repr(C)]`를 붙이고 메모리를 그대로 보내면 CDR 없이도 되지 않는가?"

   <details>
   <summary>해설</summary>

   `#[repr(C)]`는 멤버 순서와 C 컴파일러의 틈 규칙을 고정하지만, 세 가지가 남는다. `String`과 `Vec`은 메모리에 포인터와 길이로 있어 그대로 보낼 수 없다. 정수의 바이트 순서가 CPU마다 다르다. 그리고 받는 쪽이 다른 언어 (shape_main은 C와 C++)이면 "C 컴파일러의 틈 규칙"을 그대로 믿을 수 없다. CDR은 이 세 가지를 규칙으로 정한 것이다. 문자열과 sequence는 길이 뒤 원소로, 정수는 캡슐화 헤더가 알리는 엔디안으로, 틈은 크기의 배수 위치로. ShapeType처럼 고정 크기 멤버만 있다면 `repr(C)`의 배치와 CDR의 배치가 우연히 같을 수는 있어도, 그것은 규칙이 아니라 우연이다.

   </details>

2. 받는 쪽은 페이로드의 정수가 리틀 엔디언인지 빅 엔디언인지 어떻게 아는가? 식별자가 표에 없는 값이면 어떻게 하는가?

   <details>
   <summary>해설</summary>

   캡슐화 헤더 앞 2바이트의 식별자로 안다. `00 00`은 CDR_BE, `00 01`은 CDR_LE다 (RTPS 2.5 Sec. 10.2, 표 10.3). 데이터 자체에는 엔디안 표시가 없다. 표에 없는 값이면 어떤 규칙으로 썼는지 알 수 없으므로 읽지 않는 것이 맞다. mdds는 `UnsupportedRepresentation`으로 돌려주고, 두 벤더도 모르는 식별자의 샘플은 버린다 (6절).

   </details>

3. writer가 멤버 다섯 개짜리 ShapeType을 보내는데, reader의 타입에는 `additional_payload_size`가 없다 (멤버 넷). mdds의 `from_payload`로 그 바이트를 reader의 타입으로 읽으면 어떻게 되는가? 반대로 reader에 멤버가 하나 더 있으면?

   <details>
   <summary>해설</summary>

   reader의 멤버가 적은 쪽은 읽힌다. 멤버 넷을 읽고 나면 4바이트 (원소 수 0)가 남는데, `from_payload`는 남은 바이트를 오류로 보지 않는다 (4.3절). reader의 멤버가 더 많은 쪽은 `UnexpectedEnd`다. 다섯째 멤버를 읽을 바이트가 없다. XCDR1에서 appendable 타입은 final과 같은 바이트라 "어디까지가 이 샘플인가"를 바이트만으로 알 수 없고, 그래서 XCDR2는 appendable 타입 앞에 길이 (DHEADER)를 붙인다 (Lab 4-4, chapter16). 두 타입이 실제로 매칭되는지 (타입 호환)는 chapter16의 규칙이 정한다.

   </details>

4. 데이터가 9바이트인 페이로드에서 options는 왜 필요한가? 받는 쪽이 끝의 3바이트를 그냥 무시하면 되지 않는가?

   <details>
   <summary>해설</summary>

   받는 쪽은 본문의 길이 (12)만 알지 데이터가 9에서 끝났는지 12에서 끝났는지 모른다. 타입을 알면 9에서 끝난다고 계산할 수 있지만, 멤버가 더해진 새 버전의 타입이 보낸 바이트라면 끝의 바이트가 패딩인지 새 멤버인지 가릴 수 없다. 규격은 보내는 쪽이 options에 패딩 수를 적고 받는 쪽이 그것으로 끝을 정한다고만 적었고 (XTypes Sec. 7.6.3.1.2), 이 이유는 이 책의 설명이다. ShapeType처럼 데이터가 4의 배수이면 options는 0이다.

   </details>

5. Lab 4-4의 xcdr2 캡처 (`-x 2`)의 바이트는 프레임 50과 DHEADER 4바이트만 다르고 나머지 배치가 같다. 왜 같은가? 늘 같은가?

   <details>
   <summary>해설</summary>

   ShapeType은 appendable이다. XCDR1에서 appendable은 final과 같은 규칙으로 쓰고 (XTypes 규칙 (29)), XCDR2에서는 앞에 DHEADER를 붙인 뒤 같은 규칙으로 쓴다 (규칙 (30)). 멤버가 문자열, int32, 원시 원소의 sequence뿐이라 나머지가 같다. 늘 같지는 않다. XCDR2는 정렬 상한이 4라 int64나 float64 멤버가 있으면 틈이 달라진다. 그것이 chapter16의 내용이다.

   </details>

## 참고 자료

- [CORBA 3.3 Part 2 규격 (PDF)](https://www.omg.org/spec/CORBA/3.3/Interoperability/PDF), Sec. 9.3 CDR Transfer Syntax: 정렬 표 9.1, 원시 타입의 비트 배치 (Figure 9.1, 9.2), 구성 타입의 규칙 (3.2절, 3.5절)
- [DDS-XTypes 1.3 규격 (PDF)](https://www.omg.org/spec/DDS-XTypes/1.3/PDF), Sec. 7.4.1.1 PLAIN_CDR (Table 31, 32), Sec. 7.4.3.2 스트림 상태와 MAXALIGN, Sec. 7.4.3.5.3 직렬화 규칙 (1) ~ (30), Sec. 7.6.3.1.2 캡슐화 식별자 표 60과 options (3.4절, 3.5절)
- [DDSI-RTPS 2.5 규격 (PDF)](https://www.omg.org/spec/DDSI-RTPS/2.5/PDF), Sec. 10 Serialized Payload Representation: 캡슐화 헤더 4바이트와 정렬 기준점 (10.2), 식별자 표 10.3 (3.4절)
- [Cyclone DDS `f2aaccb4` `dds_cdrstream.c`](https://github.com/eclipse-cyclonedds/cyclonedds/blob/f2aaccb4/src/core/cdr/src/dds_cdrstream.c): opcode를 해석하는 직렬화기와 `dds_stream_normalize` (6절)
- [Fast-CDR `v2.3.5` `Cdr.cpp`](https://github.com/eProsima/Fast-CDR/blob/v2.3.5/src/cpp/Cdr.cpp): `serialize_encapsulation`, `set_dds_cdr_options`, `read_encapsulation` (6절)
- [Fast-DDS-Gen `v4.3.0` `TypesCdrAuxHeaderImpl.stg`](https://github.com/eProsima/Fast-DDS-Gen/blob/v4.3.0/src/main/java/com/eprosima/fastdds/idl/templates/TypesCdrAuxHeaderImpl.stg): 타입마다 생성되는 `serialize` / `deserialize`의 틀 (6절)
