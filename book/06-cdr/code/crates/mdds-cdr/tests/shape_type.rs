//! ShapeType을 손으로 구현하고, 캡처의 바이트와 맞춘다 (chapter6 Lab 6-3).
//!
//! shape.idl (dds-rtps 526dc56e):
//!   @appendable struct ShapeType {
//!     @key string<128> color;
//!     int32 x;
//!     int32 y;
//!     int32 shapesize;
//!     sequence<uint8> additional_payload_size;
//!   };
//! XCDR1에서 appendable은 final과 같은 바이트다 (XTypes 1.3 Sec. 7.4.3.5.3 규칙 (29)).
//! 골든 바이트는 Wireshark의 `serializedData` (헤더 4바이트 포함)에서 옮겼다.
//! pcap을 직접 읽는 테스트는 chapter7의 mdds-rtps에 둔다.

use mdds_cdr::{
    CdrDeserialize, CdrError, CdrReader, CdrSerialize, CdrWriter, EncapsulationHeader, Endianness,
    RepresentationId, XcdrVersion, from_payload, to_payload,
};

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
        // 멤버를 선언 순서대로 (CORBA 3.3 Part 2 Sec. 9.3.2.2). bound 128은 와이어에 없다.
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

fn blue(x: i32, y: i32) -> ShapeType {
    ShapeType {
        color: "BLUE".to_string(),
        x,
        y,
        shapesize: 20,
        additional_payload_size: Vec::new(),
    }
}

/// hex 문자열 (공백 무시)을 바이트로.
fn hex(s: &str) -> Vec<u8> {
    let digits: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    assert!(digits.len().is_multiple_of(2), "hex 자릿수가 홀수다");
    digits
        .chunks(2)
        .map(|pair| u8::from_str_radix(core::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

// reliable-cyclone-pub.pcap 프레임 50 (Cyclone writer 0x00000202, seq 12), serializedData.
// sub.log: Square BLUE 161 156 [20]
const CYCLONE_FRAME_50: &str =
    "0001 0000  05000000 424c5545 00 000000  a1000000 9c000000 14000000 00000000";

// reliable-fastdds-pub.pcap 프레임 32 (Fast DDS writer 0x00000102의 첫 DATA), serializedData.
// sub.log: Square BLUE 194 066 [20]
const FASTDDS_FRAME_32: &str =
    "0001 0000  05000000 424c5545 00 000000  c2000000 42000000 14000000 00000000";

// dispose-on-exit.pcap 프레임 268, serializedKey. key 멤버 (color)만 9바이트, 끝 패딩 3.
const DISPOSE_FRAME_268: &str = "0001 0003  05000000 424c5545 00 000000";

// xcdr2.pcap 프레임 41, serializedData. D_CDR2_LE (0x0009)와 DHEADER (chapter16).
const XCDR2_FRAME_41: &str =
    "0009 0000  1c000000 05000000 424c5545 00 000000  0f000000 0c000000 14000000 00000000";

#[test]
fn cyclone_frame_50_serializes_to_the_capture_bytes() {
    let payload = to_payload(&blue(161, 156), Endianness::Little).unwrap();
    assert_eq!(payload, hex(CYCLONE_FRAME_50));
    assert_eq!(payload.len(), 32);
}

#[test]
fn fastdds_frame_32_deserializes_to_the_logged_sample() {
    let payload = hex(FASTDDS_FRAME_32);
    assert_eq!(from_payload::<ShapeType>(&payload).unwrap(), blue(194, 66));
    // 남는 바이트가 없다
    let (header, rest) = EncapsulationHeader::split(&payload).unwrap();
    assert_eq!(header.padding(), 0);
    let mut r = CdrReader::new(
        header.data(rest).unwrap(),
        Endianness::Little,
        XcdrVersion::V1,
    );
    ShapeType::deserialize(&mut r).unwrap();
    assert_eq!(r.remaining(), 0);
}

#[test]
fn dispose_frame_268_carries_only_the_key_with_three_padding_bytes() {
    let payload = hex(DISPOSE_FRAME_268);
    let (header, rest) = EncapsulationHeader::split(&payload).unwrap();
    assert_eq!(header.representation, RepresentationId::CDR_LE);
    assert_eq!(header.padding(), 3);
    let data = header.data(rest).unwrap();
    assert_eq!(data.len(), 9);
    // key 멤버는 color 하나다. 형식은 chapter13에서 다룬다.
    let mut r = CdrReader::new(data, Endianness::Little, XcdrVersion::V1);
    assert_eq!(r.read_string().unwrap(), "BLUE");
    assert_eq!(r.remaining(), 0);
    // 전체 샘플로 읽으면 x에서 바이트가 모자란다 (틈 3 + int32 4)
    assert_eq!(
        from_payload::<ShapeType>(&payload),
        Err(CdrError::UnexpectedEnd {
            offset: 9,
            needed: 7,
            remaining: 0
        })
    );
}

#[test]
fn xcdr2_frame_41_is_not_supported_yet() {
    assert_eq!(
        from_payload::<ShapeType>(&hex(XCDR2_FRAME_41)),
        Err(CdrError::UnsupportedRepresentation(
            RepresentationId::D_CDR2_LE
        ))
    );
}

#[test]
fn big_endian_bytes_follow_the_spec_figure() {
    // 캡처에는 빅 엔디안이 없다. CORBA 3.3 Part 2 Sec. 9.3.1.2 Figure 9.1로 손으로 만든 값.
    let payload = to_payload(&blue(161, 156), Endianness::Big).unwrap();
    assert_eq!(
        payload,
        hex("0000 0000  00000005 424c5545 00 000000  000000a1 0000009c 00000014 00000000")
    );
    assert_eq!(from_payload::<ShapeType>(&payload).unwrap(), blue(161, 156));
}

#[test]
fn every_truncated_prefix_is_an_error_not_a_panic() {
    let payload = hex(CYCLONE_FRAME_50);
    for len in 0..payload.len() {
        assert!(
            from_payload::<ShapeType>(&payload[..len]).is_err(),
            "prefix {len}"
        );
    }
}
