//! 바이트를 쓰는 쪽. 정렬과 엔디안만 안다.

use crate::{CdrError, CdrSerialize, Endianness, XcdrVersion};

/// 정렬과 엔디안을 아는 쓰기 커서.
///
/// 정렬의 기준은 이 버퍼의 0이다. 캡슐화 헤더 (4바이트)는 이 writer 밖에서 붙인다
/// ([`crate::encapsulation`]). 그래서 RTPS 2.5 Sec. 10.2의 "스트림은 options 뒤에서
/// 다시 시작한다"가 저절로 맞는다.
#[derive(Debug)]
pub struct CdrWriter {
    buf: Vec<u8>,
    endianness: Endianness,
    version: XcdrVersion,
}

macro_rules! write_int {
    ($name:ident, $t:ty) => {
        #[doc = concat!("`", stringify!($t), "` 하나를 자기 크기로 정렬해 쓴다.")]
        pub fn $name(&mut self, v: $t) {
            self.align(core::mem::size_of::<$t>());
            match self.endianness {
                Endianness::Big => self.buf.extend_from_slice(&v.to_be_bytes()),
                Endianness::Little => self.buf.extend_from_slice(&v.to_le_bytes()),
            }
        }
    };
}

impl CdrWriter {
    /// 빈 버퍼로 시작한다.
    pub fn new(endianness: Endianness, version: XcdrVersion) -> Self {
        CdrWriter {
            buf: Vec::new(),
            endianness,
            version,
        }
    }

    pub fn endianness(&self) -> Endianness {
        self.endianness
    }

    pub fn version(&self) -> XcdrVersion {
        self.version
    }

    /// 지금까지 쓴 바이트 수. 다음 값이 놓일 위치다.
    pub fn position(&self) -> usize {
        self.buf.len()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    /// 다음 값이 `n`의 배수 위치에 오도록 0을 채운다.
    ///
    /// CORBA 3.3 Part 2 Sec. 9.3.1.1: 크기 n인 원시 값은 n의 배수 위치에서 시작하고 틈은 최소다.
    /// XTypes 1.3 Sec. 7.4.3.2 Table 37: 실제 정렬은 min(n, maxalign)이다.
    /// 틈의 값은 규격이 정하지 않았다. mdds는 0을 쓴다.
    pub fn align(&mut self, n: usize) {
        debug_assert!(n > 0);
        let n = n.min(self.version.max_align());
        let pad = (n - self.buf.len() % n) % n;
        self.buf.resize(self.buf.len() + pad, 0);
    }

    /// 1바이트. 정렬이 필요 없다.
    pub fn write_u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn write_i8(&mut self, v: i8) {
        self.buf.push(v as u8);
    }

    /// CORBA 3.3 Part 2 Sec. 9.3.1.5: TRUE는 1, FALSE는 0인 1바이트.
    pub fn write_bool(&mut self, v: bool) {
        self.buf.push(u8::from(v));
    }

    write_int!(write_u16, u16);
    write_int!(write_i16, i16);
    write_int!(write_u32, u32);
    write_int!(write_i32, i32);
    write_int!(write_u64, u64);
    write_int!(write_i64, i64);

    /// IEEE 754 비트를 그대로 쓴다 (CORBA 3.3 Part 2 Sec. 9.3.1.3). NaN의 payload도 보존된다.
    pub fn write_f32(&mut self, v: f32) {
        self.write_u32(v.to_bits());
    }

    pub fn write_f64(&mut self, v: f64) {
        self.write_u64(v.to_bits());
    }

    /// 바이트를 정렬 없이 그대로 쓴다. octet 배열과 `sequence<octet>`의 원소에 쓴다.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    /// 문자열: u32 길이 (NUL 포함), UTF-8 바이트, NUL.
    ///
    /// CORBA 3.3 Part 2 Sec. 9.3.2.7, XTypes 1.3 Sec. 7.4.1.1.2와 Sec. 7.4.3.5.3 규칙 (3).
    /// 빈 문자열은 길이 1 (NUL만)이다. 안에 NUL이 있으면 CDR 문자열로 표현할 수 없다.
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

    /// sequence: u32 원소 수, 그 뒤 원소들 (각각 자기 정렬).
    ///
    /// CORBA 3.3 Part 2 Sec. 9.3.2.5, XTypes 1.3 Sec. 7.4.3.5.3 규칙 (13).
    pub fn write_sequence<T: CdrSerialize>(&mut self, items: &[T]) -> Result<(), CdrError> {
        let len = u32::try_from(items.len()).map_err(|_| CdrError::LengthOverflow)?;
        self.write_u32(len);
        for item in items {
            item.serialize(self)?;
        }
        Ok(())
    }

    /// `CdrSerialize`를 구현한 값 하나를 쓴다. struct의 멤버를 선언 순서대로 나열할 때 쓴다.
    pub fn write<T: CdrSerialize + ?Sized>(&mut self, value: &T) -> Result<(), CdrError> {
        value.serialize(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn le() -> CdrWriter {
        CdrWriter::new(Endianness::Little, XcdrVersion::V1)
    }

    #[test]
    fn u8_then_each_primitive_pads_to_its_size() {
        // CORBA 3.3 Part 2 Sec. 9.3.1.1 Table 9.1
        type Write = fn(&mut CdrWriter);
        let cases: [(Write, usize); 6] = [
            (|w| w.write_u16(1), 2),
            (|w| w.write_i16(1), 2),
            (|w| w.write_u32(1), 4),
            (|w| w.write_f32(1.0), 4),
            (|w| w.write_u64(1), 8),
            (|w| w.write_f64(1.0), 8),
        ];
        for (write, size) in cases {
            let mut w = le();
            w.write_u8(0xff);
            write(&mut w);
            // 1바이트 뒤 size - 1의 틈, 그 뒤 값
            assert_eq!(w.position(), 2 * size, "size {size}");
            assert!(w.as_bytes()[1..size].iter().all(|&b| b == 0), "size {size}");
        }
    }

    #[test]
    fn v2_caps_alignment_at_four() {
        // XTypes 1.3 Sec. 7.4.3.2 Table 37: MAXALIGN(VERSION2) = 4
        let mut w = CdrWriter::new(Endianness::Little, XcdrVersion::V2);
        w.write_u8(1);
        w.write_u64(2);
        assert_eq!(w.position(), 12);
        let mut w = le();
        w.write_u8(1);
        w.write_u64(2);
        assert_eq!(w.position(), 16);
    }

    #[test]
    fn little_and_big_endian_bytes() {
        // CORBA 3.3 Part 2 Sec. 9.3.1.2 Figure 9.1
        let mut w = le();
        w.write_i32(0x0102_0304);
        w.write_i32(-1);
        assert_eq!(
            w.as_bytes(),
            [0x04, 0x03, 0x02, 0x01, 0xff, 0xff, 0xff, 0xff]
        );
        let mut w = CdrWriter::new(Endianness::Big, XcdrVersion::V1);
        w.write_i32(0x0102_0304);
        w.write_i32(-1);
        assert_eq!(
            w.as_bytes(),
            [0x01, 0x02, 0x03, 0x04, 0xff, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn string_has_length_including_nul() {
        let mut w = le();
        w.write_string("BLUE").unwrap();
        assert_eq!(w.as_bytes(), [5, 0, 0, 0, b'B', b'L', b'U', b'E', 0]);
        let mut w = le();
        w.write_string("").unwrap();
        assert_eq!(w.as_bytes(), [1, 0, 0, 0, 0]);
    }

    #[test]
    fn string_with_interior_nul_is_rejected() {
        let mut w = le();
        assert_eq!(w.write_string("a\0b"), Err(CdrError::StringContainsNul));
    }

    #[test]
    fn sequence_has_count_then_elements() {
        let mut w = le();
        w.write_sequence(&[1u8, 2, 3]).unwrap();
        assert_eq!(w.as_bytes(), [3, 0, 0, 0, 1, 2, 3]);
        let mut w = le();
        w.write_sequence::<u8>(&[]).unwrap();
        assert_eq!(w.as_bytes(), [0, 0, 0, 0]);
    }

    #[test]
    fn sequence_elements_are_aligned_each() {
        // 원소 수 4바이트 뒤 i64는 8의 배수 위치로 간다
        let mut w = le();
        w.write_sequence(&[1i64]).unwrap();
        assert_eq!(w.position(), 16);
        assert_eq!(&w.as_bytes()[..8], [1, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn padding_bytes_are_zero_even_after_other_data() {
        let mut w = le();
        w.write_bytes(&[0xaa, 0xbb, 0xcc]);
        w.write_u32(1);
        assert_eq!(w.as_bytes(), [0xaa, 0xbb, 0xcc, 0, 1, 0, 0, 0]);
    }

    #[test]
    fn align_writes_nothing_when_already_aligned() {
        let mut w = le();
        w.write_u32(1);
        w.align(1);
        w.align(4);
        w.align(8);
        assert_eq!(w.position(), 8);
        w.write_u32(2);
        w.align(2);
        assert_eq!(w.position(), 12);
    }

    #[test]
    fn bool_is_one_byte() {
        let mut w = le();
        w.write_bool(true);
        w.write_bool(false);
        assert_eq!(w.as_bytes(), [1, 0]);
    }
}
