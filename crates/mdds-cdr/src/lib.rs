//! CDR / XCDR 직렬화 (chapter6부터).
//!
//! 규격: CORBA 3.3 Part 2 Sec. 9.3 (CDR), DDS-XTypes 1.3 Sec. 7.4 (XCDR1 / XCDR2),
//! RTPS 2.5 Sec. 10 (페이로드 헤더).
//!
//! 이 crate는 바이트 버퍼만 다룬다. 소켓, 시계, 스레드를 쓰지 않는다.
//! - [`CdrWriter`] / [`CdrReader`]: 정렬과 엔디안을 아는 커서. 캡슐화 헤더는 모른다.
//! - [`CdrSerialize`] / [`CdrDeserialize`]: 타입을 바이트로, 바이트를 타입으로.
//! - [`encapsulation`]: 페이로드 맨 앞 4바이트 (식별자 2 + options 2)와 끝 패딩.

pub mod encapsulation;
pub mod reader;
pub mod serialize;
pub mod writer;

pub use encapsulation::{EncapsulationHeader, RepresentationId, from_payload, to_payload};
pub use reader::CdrReader;
pub use serialize::{CdrDeserialize, CdrSerialize};
pub use writer::CdrWriter;

/// 바이트 순서. 캡슐화 헤더의 식별자 (CDR_BE / CDR_LE)가 알린다 (RTPS 2.5 Sec. 10.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    Big,
    Little,
}

/// XCDR 인코딩 버전. 최대 정렬을 정한다.
///
/// XTypes 1.3 Sec. 7.4.3.2 Table 37: MAXALIGN(VERSION1) = 8, MAXALIGN(VERSION2) = 4.
/// chapter6은 V1만 쓴다. V2는 chapter13 (KeyHash)과 chapter16 (XCDR2)에서 쓴다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XcdrVersion {
    V1,
    V2,
}

impl XcdrVersion {
    /// 이 버전에서 정렬의 상한. 8바이트 값도 V2에서는 4로 정렬한다.
    pub const fn max_align(self) -> usize {
        match self {
            XcdrVersion::V1 => 8,
            XcdrVersion::V2 => 4,
        }
    }
}

/// 직렬화와 역직렬화의 오류.
///
/// `offset`은 그 reader가 읽던 버퍼 안의 위치다. `from_payload`에서는 캡슐화 헤더 4바이트
/// 뒤의 데이터 기준이라 Wireshark의 `serializedData` 리프의 바이트와 바로 대조할 수 있다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CdrError {
    /// 읽을 바이트가 모자란다 (정렬 틈을 건너뛸 때도 난다).
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        remaining: usize,
    },
    /// boolean은 0 또는 1이어야 한다 (CORBA 3.3 Part 2 Sec. 9.3.1.5). 그 밖의 값은 mdds가 거부한다.
    InvalidBool { offset: usize, value: u8 },
    /// 문자열이 UTF-8이 아니다 (XTypes 1.3 Sec. 7.4.1.1.2).
    InvalidUtf8 { offset: usize },
    /// 문자열 길이가 0이거나 마지막 바이트가 NUL이 아니다 (CORBA 3.3 Part 2 Sec. 9.3.2.7).
    MissingNul { offset: usize },
    /// enum의 값이 선언된 식별자 가운데 없다.
    InvalidEnum { offset: usize, value: i32 },
    /// Rust 문자열 안에 NUL이 있어 CDR 문자열로 표현할 수 없다.
    StringContainsNul,
    /// 길이나 원소 수가 u32를 넘는다.
    LengthOverflow,
    /// 캡슐화 식별자를 이 crate가 아직 해석하지 못한다.
    UnsupportedRepresentation(RepresentationId),
    /// options의 끝 패딩 수가 데이터 길이보다 크다.
    InvalidPadding { padding: usize, len: usize },
}

impl core::fmt::Display for CdrError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CdrError::UnexpectedEnd {
                offset,
                needed,
                remaining,
            } => write!(
                f,
                "unexpected end at offset {offset}: needed {needed} bytes, {remaining} remaining"
            ),
            CdrError::InvalidBool { offset, value } => {
                write!(f, "invalid boolean {value:#04x} at offset {offset}")
            }
            CdrError::InvalidUtf8 { offset } => {
                write!(f, "invalid UTF-8 string at offset {offset}")
            }
            CdrError::MissingNul { offset } => {
                write!(f, "string at offset {offset} is not NUL-terminated")
            }
            CdrError::InvalidEnum { offset, value } => {
                write!(f, "invalid enum value {value} at offset {offset}")
            }
            CdrError::StringContainsNul => write!(f, "string contains an interior NUL"),
            CdrError::LengthOverflow => write!(f, "length does not fit in u32"),
            CdrError::UnsupportedRepresentation(RepresentationId([hi, lo])) => {
                write!(
                    f,
                    "unsupported representation identifier 0x{hi:02x}{lo:02x}"
                )
            }
            CdrError::InvalidPadding { padding, len } => {
                write!(f, "padding {padding} exceeds data length {len}")
            }
        }
    }
}

impl std::error::Error for CdrError {}
