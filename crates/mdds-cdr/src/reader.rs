//! 바이트를 읽는 쪽. 길이 필드를 믿지 않고, 어떤 입력에도 패닉하지 않는다.

use crate::{CdrDeserialize, CdrError, Endianness, XcdrVersion};

/// 정렬과 엔디안을 아는 읽기 커서.
///
/// 정렬의 기준은 이 슬라이스의 0이다. 캡슐화 헤더를 뗀 데이터, 또는 ParameterList의
/// 파라미터 값 하나 (chapter7)처럼 "여기서부터 다시 센다"는 범위마다 새 reader를 만든다.
#[derive(Debug)]
pub struct CdrReader<'a> {
    buf: &'a [u8],
    pos: usize,
    endianness: Endianness,
    version: XcdrVersion,
}

macro_rules! read_int {
    ($name:ident, $t:ty) => {
        #[doc = concat!("`", stringify!($t), "` 하나를 자기 크기로 정렬해 읽는다.")]
        pub fn $name(&mut self) -> Result<$t, CdrError> {
            const N: usize = core::mem::size_of::<$t>();
            let bytes: [u8; N] = self
                .take_aligned(N)?
                .try_into()
                .expect("take_aligned는 요청한 길이를 돌려준다");
            Ok(match self.endianness {
                Endianness::Big => <$t>::from_be_bytes(bytes),
                Endianness::Little => <$t>::from_le_bytes(bytes),
            })
        }
    };
}

impl<'a> CdrReader<'a> {
    pub fn new(buf: &'a [u8], endianness: Endianness, version: XcdrVersion) -> Self {
        CdrReader {
            buf,
            pos: 0,
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

    /// 다음에 읽을 위치.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// 아직 읽지 않은 바이트 수.
    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    /// `n`의 배수 위치까지 건너뛸 틈의 크기.
    fn gap(&self, n: usize) -> usize {
        debug_assert!(n > 0);
        let n = n.min(self.version.max_align());
        (n - self.pos % n) % n
    }

    /// 다음 값이 `n`의 배수 위치에서 시작하도록 틈을 건너뛴다. 틈의 값은 검사하지 않는다.
    pub fn align(&mut self, n: usize) -> Result<(), CdrError> {
        let pad = self.gap(n);
        self.take(pad).map(|_| ())
    }

    /// 크기 `n`인 값을 자기 크기로 정렬해 떼어 돌려준다. 틈과 값을 합쳐 미리 검사하므로
    /// 모자라면 `needed`가 틈 + 값이고, 읽기 위치는 움직이지 않는다.
    fn take_aligned(&mut self, n: usize) -> Result<&'a [u8], CdrError> {
        let pad = self.gap(n);
        if self.remaining() < pad + n {
            return Err(CdrError::UnexpectedEnd {
                offset: self.pos,
                needed: pad + n,
                remaining: self.remaining(),
            });
        }
        self.take(pad)?;
        self.take(n)
    }

    /// `n`바이트를 떼어 돌려준다. 모자라면 `UnexpectedEnd`.
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

    pub fn read_u8(&mut self) -> Result<u8, CdrError> {
        Ok(self.take(1)?[0])
    }

    pub fn read_i8(&mut self) -> Result<i8, CdrError> {
        Ok(self.read_u8()? as i8)
    }

    /// 0은 false, 1은 true. 그 밖의 값은 `InvalidBool` (규격은 받는 쪽 동작을 정하지 않았다).
    pub fn read_bool(&mut self) -> Result<bool, CdrError> {
        let offset = self.pos;
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(CdrError::InvalidBool { offset, value }),
        }
    }

    read_int!(read_u16, u16);
    read_int!(read_i16, i16);
    read_int!(read_u32, u32);
    read_int!(read_i32, i32);
    read_int!(read_u64, u64);
    read_int!(read_i64, i64);

    pub fn read_f32(&mut self) -> Result<f32, CdrError> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    pub fn read_f64(&mut self) -> Result<f64, CdrError> {
        Ok(f64::from_bits(self.read_u64()?))
    }

    /// `n`바이트를 정렬 없이 그대로 읽는다.
    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], CdrError> {
        self.take(n)
    }

    /// 문자열: u32 길이 (NUL 포함), UTF-8 바이트, NUL.
    ///
    /// 길이가 0이거나 마지막 바이트가 NUL이 아니면 `MissingNul`. 길이가 남은 바이트보다
    /// 크면 할당 없이 `UnexpectedEnd`. 안쪽의 NUL은 검사하지 않는다.
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

    /// sequence: u32 원소 수, 그 뒤 원소들.
    ///
    /// 원소 수는 와이어에서 온 값이라 믿지 않는다. 미리 할당하는 메모리는 남은 바이트 수를
    /// 넘지 않고, 원소를 읽다 모자라면 `UnexpectedEnd`로 끝난다.
    pub fn read_sequence<T: CdrDeserialize>(&mut self) -> Result<Vec<T>, CdrError> {
        let len = self.read_u32()? as usize;
        let fit = self.remaining() / core::mem::size_of::<T>().max(1);
        let mut items = Vec::with_capacity(len.min(fit));
        for _ in 0..len {
            items.push(T::deserialize(self)?);
        }
        Ok(items)
    }

    /// `CdrDeserialize`를 구현한 값 하나를 읽는다.
    pub fn read<T: CdrDeserialize>(&mut self) -> Result<T, CdrError> {
        T::deserialize(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CdrWriter;

    fn le(buf: &[u8]) -> CdrReader<'_> {
        CdrReader::new(buf, Endianness::Little, XcdrVersion::V1)
    }

    #[test]
    fn reads_back_what_writer_wrote_in_both_endiannesses() {
        for endianness in [Endianness::Little, Endianness::Big] {
            let mut w = CdrWriter::new(endianness, XcdrVersion::V1);
            w.write_u8(1);
            w.write_i8(-2);
            w.write_bool(true);
            w.write_u16(3);
            w.write_i16(-4);
            w.write_u32(5);
            w.write_i32(-6);
            w.write_u64(7);
            w.write_i64(-8);
            w.write_f32(9.5);
            w.write_f64(-10.25);
            w.write_string("BLUE").unwrap();
            w.write_string("").unwrap();
            w.write_sequence(&[11u8, 12]).unwrap();
            w.write_sequence(&[0x0102u16, 0x0304]).unwrap();
            w.write_sequence::<u32>(&[]).unwrap();
            let bytes = w.into_bytes();
            let mut r = CdrReader::new(&bytes, endianness, XcdrVersion::V1);
            assert_eq!(r.read_u8().unwrap(), 1);
            assert_eq!(r.read_i8().unwrap(), -2);
            assert!(r.read_bool().unwrap());
            assert_eq!(r.read_u16().unwrap(), 3);
            assert_eq!(r.read_i16().unwrap(), -4);
            assert_eq!(r.read_u32().unwrap(), 5);
            assert_eq!(r.read_i32().unwrap(), -6);
            assert_eq!(r.read_u64().unwrap(), 7);
            assert_eq!(r.read_i64().unwrap(), -8);
            assert_eq!(r.read_f32().unwrap(), 9.5);
            assert_eq!(r.read_f64().unwrap(), -10.25);
            assert_eq!(r.read_string().unwrap(), "BLUE");
            assert_eq!(r.read_string().unwrap(), "");
            assert_eq!(r.read_sequence::<u8>().unwrap(), [11, 12]);
            assert_eq!(r.read_sequence::<u16>().unwrap(), [0x0102, 0x0304]);
            assert_eq!(r.read_sequence::<u32>().unwrap(), []);
            assert_eq!(r.remaining(), 0);
        }
    }

    #[test]
    fn v2_reads_u64_at_offset_four() {
        // XTypes 1.3 Sec. 7.4.3.2 Table 37: V2의 정렬 상한은 4
        let bytes = [1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0];
        let mut r = CdrReader::new(&bytes, Endianness::Little, XcdrVersion::V2);
        assert_eq!(r.read_u8().unwrap(), 1);
        assert_eq!(r.read_u64().unwrap(), 2);
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn unexpected_end_reports_offset_needed_and_remaining() {
        let mut r = le(&[1, 2]);
        assert_eq!(
            r.read_u32(),
            Err(CdrError::UnexpectedEnd {
                offset: 0,
                needed: 4,
                remaining: 2
            })
        );
        // 오류가 나면 읽기 위치는 그대로다
        assert_eq!(r.position(), 0);
        // 정렬 틈 3과 값 4를 합쳐 7바이트가 모자라다고 알린다
        let mut r = le(&[1]);
        r.read_u8().unwrap();
        assert_eq!(
            r.read_u32(),
            Err(CdrError::UnexpectedEnd {
                offset: 1,
                needed: 7,
                remaining: 0
            })
        );
        assert_eq!(r.position(), 1);
    }

    #[test]
    fn alignment_gap_is_skipped_but_not_checked() {
        let mut r = le(&[1, 0xff, 0xff, 0xff, 2, 0, 0, 0]);
        assert_eq!(r.read_u8().unwrap(), 1);
        assert_eq!(r.read_u32().unwrap(), 2);
    }

    #[test]
    fn bool_other_than_zero_or_one_is_invalid() {
        let mut r = le(&[0, 2]);
        assert!(!r.read_bool().unwrap());
        assert_eq!(
            r.read_bool(),
            Err(CdrError::InvalidBool {
                offset: 1,
                value: 2
            })
        );
    }

    #[test]
    fn string_errors() {
        // 길이 0: 빈 문자열도 NUL 하나는 있어야 한다
        assert_eq!(
            le(&[0, 0, 0, 0]).read_string(),
            Err(CdrError::MissingNul { offset: 0 })
        );
        // 마지막 바이트가 NUL이 아니다
        assert_eq!(
            le(&[2, 0, 0, 0, b'A', b'B']).read_string(),
            Err(CdrError::MissingNul { offset: 0 })
        );
        // UTF-8이 아니다
        assert_eq!(
            le(&[2, 0, 0, 0, 0xff, 0]).read_string(),
            Err(CdrError::InvalidUtf8 { offset: 0 })
        );
        // 길이가 남은 바이트보다 크다 (할당하지 않고 바로 오류)
        assert_eq!(
            le(&[0xff, 0xff, 0xff, 0xff, 0]).read_string(),
            Err(CdrError::UnexpectedEnd {
                offset: 4,
                needed: 0xffff_ffff,
                remaining: 1
            })
        );
    }

    #[test]
    fn huge_sequence_count_is_an_error_not_a_panic() {
        let mut r = le(&[0xff, 0xff, 0xff, 0xff, 1, 2]);
        assert_eq!(
            r.read_sequence::<u32>(),
            Err(CdrError::UnexpectedEnd {
                offset: 4,
                needed: 4,
                remaining: 2
            })
        );
    }

    #[test]
    fn nan_bits_survive_round_trip() {
        let nan = f64::from_bits(0x7ff8_0000_dead_beef);
        let mut w = CdrWriter::new(Endianness::Little, XcdrVersion::V1);
        w.write_f64(nan);
        let bytes = w.into_bytes();
        let got = le(&bytes).read_f64().unwrap();
        assert_eq!(got.to_bits(), nan.to_bits());
    }
}
