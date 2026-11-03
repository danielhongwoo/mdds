//! 페이로드 맨 앞 4바이트 (캡슐화 헤더)와 끝 패딩.
//!
//! RTPS 2.5 Sec. 10.2: SerializedPayload는 representation_identifier 2바이트와
//! representation_options 2바이트로 시작하고, CDR 스트림은 그 뒤에서 다시 시작한다.
//! XTypes 1.3 Sec. 7.6.3.1.2: options 둘째 바이트의 하위 2비트는 끝 패딩 바이트 수다.

use crate::{
    CdrDeserialize, CdrError, CdrReader, CdrSerialize, CdrWriter, Endianness, XcdrVersion,
};

/// 캡슐화 식별자 (Wireshark의 encapsulation kind). 모르는 값도 담을 수 있게 newtype이다.
///
/// QoS의 `DataRepresentationId` (XCDR = 0, XCDR2 = 2. chapter5)와는 다른 번호 공간이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepresentationId(pub [u8; 2]);

impl RepresentationId {
    // RTPS 2.5 Sec. 10.5 Table 10.3, XTypes 1.3 Sec. 7.6.3.1.2 Table 60
    pub const CDR_BE: Self = Self([0x00, 0x00]);
    pub const CDR_LE: Self = Self([0x00, 0x01]);
    pub const PL_CDR_BE: Self = Self([0x00, 0x02]);
    pub const PL_CDR_LE: Self = Self([0x00, 0x03]);
    // XCDR2 (chapter16). 값은 XTypes 1.3 Table 60을 따른다. RTPS 2.5 Table 10.3은 0x10 ~ 0x15로
    // 다르지만 와이어 (xcdr2 캡처의 0x0009)는 Table 60이다.
    pub const CDR2_BE: Self = Self([0x00, 0x06]);
    pub const CDR2_LE: Self = Self([0x00, 0x07]);
    pub const D_CDR2_BE: Self = Self([0x00, 0x08]);
    pub const D_CDR2_LE: Self = Self([0x00, 0x09]);
    pub const PL_CDR2_BE: Self = Self([0x00, 0x0a]);
    pub const PL_CDR2_LE: Self = Self([0x00, 0x0b]);

    /// 이 식별자가 알리는 엔디안. 엔디안이 정해지지 않은 식별자 (XML, 벤더 정의, 모르는 값)는 `None`.
    pub fn endianness(self) -> Option<Endianness> {
        match self.0 {
            [0x00, 0x00 | 0x02 | 0x06 | 0x08 | 0x0a] => Some(Endianness::Big),
            [0x00, 0x01 | 0x03 | 0x07 | 0x09 | 0x0b] => Some(Endianness::Little),
            _ => None,
        }
    }
}

/// 캡슐화 헤더 4바이트.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncapsulationHeader {
    pub representation: RepresentationId,
    /// options 2바이트. 둘째 바이트의 하위 2비트만 뜻이 정해져 있다 (끝 패딩 수).
    pub options: [u8; 2],
}

impl EncapsulationHeader {
    pub const LEN: usize = 4;

    /// 길이가 `data_len`인 데이터를 4의 배수로 맞추는 끝 패딩 수 (0 ~ 3)를 options에 실은 헤더.
    pub fn for_data_len(representation: RepresentationId, data_len: usize) -> Self {
        EncapsulationHeader {
            representation,
            options: [0, Self::padding_for(data_len) as u8],
        }
    }

    /// 길이 `data_len`을 4의 배수로 맞추는 데 필요한 끝 패딩 수.
    pub fn padding_for(data_len: usize) -> usize {
        (4 - data_len % 4) % 4
    }

    /// options가 알리는 끝 패딩 바이트 수.
    pub fn padding(&self) -> usize {
        usize::from(self.options[1] & 0x03)
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        let [r0, r1] = self.representation.0;
        [r0, r1, self.options[0], self.options[1]]
    }

    /// 페이로드를 헤더와 나머지 (데이터 + 끝 패딩)로 나눈다. 식별자는 무엇이든 받는다.
    pub fn split(payload: &[u8]) -> Result<(Self, &[u8]), CdrError> {
        match payload {
            [r0, r1, o0, o1, rest @ ..] => Ok((
                EncapsulationHeader {
                    representation: RepresentationId([*r0, *r1]),
                    options: [*o0, *o1],
                },
                rest,
            )),
            _ => Err(CdrError::UnexpectedEnd {
                offset: 0,
                needed: Self::LEN,
                remaining: payload.len(),
            }),
        }
    }

    /// 나머지에서 끝 패딩을 뗀 데이터. 받는 쪽은 이 비트로 데이터의 끝을 정한다
    /// (XTypes 1.3 Sec. 7.6.3.1.2). 패딩 바이트의 값은 검사하지 않는다.
    pub fn data<'a>(&self, rest: &'a [u8]) -> Result<&'a [u8], CdrError> {
        let padding = self.padding();
        rest.len()
            .checked_sub(padding)
            .and_then(|end| rest.get(..end))
            .ok_or(CdrError::InvalidPadding {
                padding,
                len: rest.len(),
            })
    }
}

/// 값을 XCDR1 PLAIN_CDR 페이로드로 만든다: 헤더 4바이트, 데이터, 4의 배수까지 0 패딩.
///
/// XTypes 1.3 Sec. 7.4.1.1: 끝 패딩은 0이어야 한다. 패딩 수는 options에 싣는다.
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

/// XCDR1 PLAIN_CDR 페이로드 (CDR_BE / CDR_LE)에서 값을 읽는다.
///
/// 다른 식별자는 `UnsupportedRepresentation`이다. PL_CDR은 chapter7, 9에서, XCDR2는
/// chapter16에서 해석한다. 값 뒤에 바이트가 남아도 오류가 아니다.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_payload_is_padded_to_four_and_options_carry_the_count() {
        // "BLUE": 길이 4 + 5바이트 = 9, 패딩 3 -> 데이터 12, 헤더 포함 16
        let payload = to_payload("BLUE", Endianness::Little).unwrap();
        assert_eq!(
            payload,
            [0, 1, 0, 3, 5, 0, 0, 0, b'B', b'L', b'U', b'E', 0, 0, 0, 0]
        );
        // 데이터가 4의 배수면 패딩 0
        let payload = to_payload(&7u32, Endianness::Little).unwrap();
        assert_eq!(payload, [0, 1, 0, 0, 7, 0, 0, 0]);
    }

    #[test]
    fn from_payload_strips_padding_and_reads_the_value() {
        let payload = to_payload("BLUE", Endianness::Little).unwrap();
        assert_eq!(from_payload::<String>(&payload).unwrap(), "BLUE");
        let (header, rest) = EncapsulationHeader::split(&payload).unwrap();
        assert_eq!(header.padding(), 3);
        assert_eq!(header.data(rest).unwrap().len(), 9);
    }

    #[test]
    fn big_endian_payload_round_trips() {
        let payload = to_payload(&0x0102_0304u32, Endianness::Big).unwrap();
        assert_eq!(payload, [0, 0, 0, 0, 1, 2, 3, 4]);
        assert_eq!(from_payload::<u32>(&payload).unwrap(), 0x0102_0304);
    }

    #[test]
    fn header_shorter_than_four_bytes_is_unexpected_end() {
        assert_eq!(
            from_payload::<u32>(&[0, 1, 0]),
            Err(CdrError::UnexpectedEnd {
                offset: 0,
                needed: 4,
                remaining: 3
            })
        );
    }

    #[test]
    fn pl_cdr_and_xcdr2_are_unsupported_here() {
        assert_eq!(
            from_payload::<u32>(&[0, 3, 0, 0, 1, 0, 0, 0]),
            Err(CdrError::UnsupportedRepresentation(
                RepresentationId::PL_CDR_LE
            ))
        );
        assert_eq!(
            from_payload::<u32>(&[0, 9, 0, 0, 1, 0, 0, 0]),
            Err(CdrError::UnsupportedRepresentation(
                RepresentationId::D_CDR2_LE
            ))
        );
        // 모르는 값도 담긴다 (벤더 전용 범위 0xC0 ~ 0xFF. XTypes 1.3 Sec. 7.6.3.1.2)
        assert_eq!(
            from_payload::<u32>(&[0xc0, 0x01, 0, 0, 1, 0, 0, 0]),
            Err(CdrError::UnsupportedRepresentation(RepresentationId([
                0xc0, 0x01
            ])))
        );
    }

    #[test]
    fn trailing_bytes_after_the_value_are_not_an_error() {
        // XCDR1 appendable 타입에 멤버가 더해진 샘플을 옛 타입으로 읽을 여지를 남긴다
        assert_eq!(from_payload::<u8>(&[0, 1, 0, 0, 9, 0, 0, 0]).unwrap(), 9);
    }

    #[test]
    fn padding_larger_than_data_is_invalid() {
        assert_eq!(
            from_payload::<u8>(&[0, 1, 0, 3, 1]),
            Err(CdrError::InvalidPadding { padding: 3, len: 1 })
        );
    }

    #[test]
    fn upper_option_bits_are_ignored() {
        assert_eq!(from_payload::<u8>(&[0, 1, 0xff, 0xfc, 9]).unwrap(), 9);
    }

    #[test]
    fn representation_id_endianness() {
        assert_eq!(RepresentationId::CDR_BE.endianness(), Some(Endianness::Big));
        assert_eq!(
            RepresentationId::D_CDR2_LE.endianness(),
            Some(Endianness::Little)
        );
        // XML (0x0004)은 엔디안이 없다
        assert_eq!(RepresentationId([0x00, 0x04]).endianness(), None);
    }
}
