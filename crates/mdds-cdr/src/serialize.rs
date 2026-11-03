//! 타입과 바이트를 잇는 trait. 원시 타입, 문자열, sequence, array의 구현.
//!
//! struct, enum, union은 사용자가 (chapter17부터는 IDL 컴파일러가) 멤버를 선언 순서대로
//! 나열해 구현한다. CORBA 3.3 Part 2 Sec. 9.3.2.2: struct의 멤버는 선언 순서대로, 각각 자기
//! 타입의 규칙으로 쓴다. 구성 타입 자체는 정렬이 없다 (Sec. 9.3.2.1).

use crate::{CdrError, CdrReader, CdrWriter};

/// 값을 CDR 바이트로 쓴다.
///
/// `Vec`에 쓰는 일은 실패하지 않지만, Rust 값이 CDR로 표현될 수 없는 경우 (안에 NUL이 있는
/// 문자열, u32를 넘는 길이)가 있어 `Result`다.
pub trait CdrSerialize {
    fn serialize(&self, w: &mut CdrWriter) -> Result<(), CdrError>;
}

/// CDR 바이트에서 값을 읽는다.
pub trait CdrDeserialize: Sized {
    fn deserialize(r: &mut CdrReader<'_>) -> Result<Self, CdrError>;
}

macro_rules! primitive {
    ($t:ty, $write:ident, $read:ident) => {
        impl CdrSerialize for $t {
            fn serialize(&self, w: &mut CdrWriter) -> Result<(), CdrError> {
                w.$write(*self);
                Ok(())
            }
        }

        impl CdrDeserialize for $t {
            fn deserialize(r: &mut CdrReader<'_>) -> Result<Self, CdrError> {
                r.$read()
            }
        }
    };
}

// XTypes 1.3 Sec. 7.4.1.1 Table 31. IDL char는 1바이트 (Char8)이고 해석하지 않으므로
// Rust `char` (4바이트)에는 구현하지 않는다. `usize`도 와이어 타입이 아니다.
primitive!(u8, write_u8, read_u8);
primitive!(i8, write_i8, read_i8);
primitive!(bool, write_bool, read_bool);
primitive!(u16, write_u16, read_u16);
primitive!(i16, write_i16, read_i16);
primitive!(u32, write_u32, read_u32);
primitive!(i32, write_i32, read_i32);
primitive!(u64, write_u64, read_u64);
primitive!(i64, write_i64, read_i64);
primitive!(f32, write_f32, read_f32);
primitive!(f64, write_f64, read_f64);

impl CdrSerialize for str {
    fn serialize(&self, w: &mut CdrWriter) -> Result<(), CdrError> {
        w.write_string(self)
    }
}

impl CdrSerialize for String {
    fn serialize(&self, w: &mut CdrWriter) -> Result<(), CdrError> {
        w.write_string(self)
    }
}

impl CdrDeserialize for String {
    fn deserialize(r: &mut CdrReader<'_>) -> Result<Self, CdrError> {
        r.read_string()
    }
}

impl<T: CdrSerialize + ?Sized> CdrSerialize for &T {
    fn serialize(&self, w: &mut CdrWriter) -> Result<(), CdrError> {
        (**self).serialize(w)
    }
}

/// sequence<T>: 원소 수 뒤에 원소들. `[T]`에는 구현하지 않는다 (array와 `&` 하나 차이로
/// 다른 바이트가 되므로). 슬라이스는 `CdrWriter::write_sequence`로 쓴다.
impl<T: CdrSerialize> CdrSerialize for Vec<T> {
    fn serialize(&self, w: &mut CdrWriter) -> Result<(), CdrError> {
        w.write_sequence(self)
    }
}

impl<T: CdrDeserialize> CdrDeserialize for Vec<T> {
    fn deserialize(r: &mut CdrReader<'_>) -> Result<Self, CdrError> {
        r.read_sequence()
    }
}

/// array: 길이 없이 원소만 (CORBA 3.3 Part 2 Sec. 9.3.2.4, XTypes 1.3 Sec. 7.4.3.5.3 규칙 (10)).
impl<T: CdrSerialize, const N: usize> CdrSerialize for [T; N] {
    fn serialize(&self, w: &mut CdrWriter) -> Result<(), CdrError> {
        for item in self {
            item.serialize(w)?;
        }
        Ok(())
    }
}

impl<T: CdrDeserialize, const N: usize> CdrDeserialize for [T; N] {
    fn deserialize(r: &mut CdrReader<'_>) -> Result<Self, CdrError> {
        let items = (0..N)
            .map(|_| T::deserialize(r))
            .collect::<Result<Vec<T>, CdrError>>()?;
        Ok(items
            .try_into()
            .unwrap_or_else(|_| unreachable!("원소 N개를 모았다")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Endianness, XcdrVersion};

    fn to_bytes<T: CdrSerialize>(v: &T) -> Vec<u8> {
        let mut w = CdrWriter::new(Endianness::Little, XcdrVersion::V1);
        v.serialize(&mut w).unwrap();
        w.into_bytes()
    }

    fn from_bytes<T: CdrDeserialize>(bytes: &[u8]) -> Result<T, CdrError> {
        let mut r = CdrReader::new(bytes, Endianness::Little, XcdrVersion::V1);
        let v = T::deserialize(&mut r)?;
        assert_eq!(r.remaining(), 0);
        Ok(v)
    }

    // struct { octet a; long b; }
    #[derive(Debug, PartialEq)]
    struct Small {
        a: u8,
        b: i32,
    }

    impl CdrSerialize for Small {
        fn serialize(&self, w: &mut CdrWriter) -> Result<(), CdrError> {
            w.write(&self.a)?;
            w.write(&self.b)
        }
    }

    impl CdrDeserialize for Small {
        fn deserialize(r: &mut CdrReader<'_>) -> Result<Self, CdrError> {
            Ok(Small {
                a: r.read()?,
                b: r.read()?,
            })
        }
    }

    // struct { octet x; Inner inner; }, struct Inner { unsigned short y; }
    struct Inner {
        y: u16,
    }

    impl CdrSerialize for Inner {
        fn serialize(&self, w: &mut CdrWriter) -> Result<(), CdrError> {
            w.write(&self.y)
        }
    }

    struct Outer {
        x: u8,
        inner: Inner,
    }

    impl CdrSerialize for Outer {
        fn serialize(&self, w: &mut CdrWriter) -> Result<(), CdrError> {
            w.write(&self.x)?;
            w.write(&self.inner)
        }
    }

    // enum Color { RED, GREEN, BLUE }: 기본 holder는 Int32 (XTypes 1.3 Sec. 7.4.1.1.3 Table 32)
    #[derive(Debug, PartialEq)]
    enum Color {
        Red,
        Green,
        Blue,
    }

    impl CdrSerialize for Color {
        fn serialize(&self, w: &mut CdrWriter) -> Result<(), CdrError> {
            let value: i32 = match self {
                Color::Red => 0,
                Color::Green => 1,
                Color::Blue => 2,
            };
            w.write_i32(value);
            Ok(())
        }
    }

    impl CdrDeserialize for Color {
        fn deserialize(r: &mut CdrReader<'_>) -> Result<Self, CdrError> {
            let offset = r.position();
            match r.read_i32()? {
                0 => Ok(Color::Red),
                1 => Ok(Color::Green),
                2 => Ok(Color::Blue),
                value => Err(CdrError::InvalidEnum { offset, value }),
            }
        }
    }

    #[test]
    fn struct_members_in_declaration_order_each_aligned() {
        let bytes = to_bytes(&Small { a: 1, b: 2 });
        assert_eq!(bytes, [1, 0, 0, 0, 2, 0, 0, 0]);
        assert_eq!(from_bytes::<Small>(&bytes).unwrap(), Small { a: 1, b: 2 });
    }

    #[test]
    fn nested_struct_has_no_own_alignment() {
        // CORBA 3.3 Part 2 Sec. 9.3.2.1: inner의 u16은 2의 배수 위치면 된다 (4가 아니다)
        let bytes = to_bytes(&Outer {
            x: 1,
            inner: Inner { y: 2 },
        });
        assert_eq!(bytes, [1, 0, 2, 0]);
    }

    #[test]
    fn i32_then_i64_pads_four_from_data_start() {
        // 정렬 기준이 데이터 시작 (헤더 뒤)이므로 i64는 8에 놓인다 (RTPS 2.5 Sec. 10.2)
        let mut w = CdrWriter::new(Endianness::Little, XcdrVersion::V1);
        w.write(&1i32).unwrap();
        w.write(&2i64).unwrap();
        assert_eq!(w.position(), 16);
        assert_eq!(&w.as_bytes()[4..8], [0, 0, 0, 0]);
    }

    #[test]
    fn array_has_no_length_prefix() {
        let bytes = to_bytes(&[1u16, 2]);
        assert_eq!(bytes, [1, 0, 2, 0]);
        assert_eq!(from_bytes::<[u16; 2]>(&bytes).unwrap(), [1, 2]);
        // 같은 값의 sequence는 원소 수가 앞에 붙는다
        assert_eq!(to_bytes(&vec![1u16, 2]), [2, 0, 0, 0, 1, 0, 2, 0]);
    }

    #[test]
    fn vec_of_strings_round_trips() {
        let v = vec!["A".to_string(), "BC".to_string()];
        let bytes = to_bytes(&v);
        assert_eq!(
            bytes,
            [
                2, 0, 0, 0, 2, 0, 0, 0, b'A', 0, 0, 0, 3, 0, 0, 0, b'B', b'C', 0
            ]
        );
        assert_eq!(from_bytes::<Vec<String>>(&bytes).unwrap(), v);
    }

    #[test]
    fn enum_is_i32_and_unknown_value_is_rejected() {
        assert_eq!(to_bytes(&Color::Blue), [2, 0, 0, 0]);
        assert_eq!(from_bytes::<Color>(&[1, 0, 0, 0]).unwrap(), Color::Green);
        assert_eq!(
            from_bytes::<Color>(&[7, 0, 0, 0]),
            Err(CdrError::InvalidEnum {
                offset: 0,
                value: 7
            })
        );
    }

    #[test]
    fn str_and_string_serialize_the_same() {
        assert_eq!(to_bytes(&"BLUE"), to_bytes(&"BLUE".to_string()));
    }
}
