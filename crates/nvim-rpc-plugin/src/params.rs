use crate::{Array, Bytes};

use nvim_rpc::api_types::{Decode, DecodeError};
use nvim_rpc::sys::types;

/// Decodes an incoming request or notification's argument array.
///
/// The result moves into a handler task, so it must be `Send + 'static`.
/// [`crate::Bin`] retains shared bytes without copying its payload. A custom
/// type can delegate field decoding to a tuple:
///
/// ```
/// use nvim_rpc_plugin::{Array, Bin, DecodeError, FromParams};
///
/// struct Add {
///     lhs: i32,
///     rhs: i32,
///     label: Bin,
/// }
///
/// impl FromParams for Add {
///     fn from_params(params: Array) -> Result<Self, DecodeError> {
///         let (lhs, rhs, label) = <(i32, i32, Bin)>::from_params(params)?;
///         Ok(Self { lhs, rhs, label })
///     }
/// }
/// ```
///
/// This conversion runs before the hook and receives only the arguments. Use
/// [`Array`] itself when argument layouts differ by method, then decode inside
/// the hook after matching the method name. `()` requires an empty argument
/// array; `Array` can be used to ignore arbitrary arguments.
/// [`crate::Bin`], [`Array`], [`crate::Map`], and [`crate::Object`] retain shared
/// message storage across receives. `String` validates UTF-8 and copies text
/// into an owned string.
///
/// A request decoding failure is sent as an error reply. A notification
/// decoding failure ends the connection. Decode inside the hook when you need
/// method-specific error handling.
pub trait FromParams: Sized + Send + 'static {
    fn from_params(params: Array) -> Result<Self, DecodeError>;
}

impl FromParams for Array {
    fn from_params(params: Array) -> Result<Self, DecodeError> {
        Ok(params)
    }
}

impl FromParams for () {
    fn from_params(params: Array) -> Result<Self, DecodeError> {
        match params.remaining_len() {
            0 => Ok(()),
            actual => Err(DecodeError::InvalidLength {
                expected: 0,
                actual,
            }),
        }
    }
}

macro_rules! _impl_tuple {
    ($n:expr; $($ty:ident),+) => {
        impl<$($ty: Decode<types::Object, Bytes> + Send + 'static),+> FromParams for ($($ty,)+) {
            fn from_params(mut params: Array) -> Result<Self, DecodeError> {
                let actual = params.remaining_len();
                if actual != $n {
                    return Err(DecodeError::InvalidLength { expected: $n, actual });
                }
                Ok(($(
                    <$ty as Decode<types::Object, Bytes>>::decode(
                        params.next().expect("parameter length was checked"),
                    )?,
                )+))
            }
        }
    };
}

_impl_tuple!(1; T0);
_impl_tuple!(2; T0, T1);
_impl_tuple!(3; T0, T1, T2);
_impl_tuple!(4; T0, T1, T2, T3);
_impl_tuple!(5; T0, T1, T2, T3, T4);
_impl_tuple!(6; T0, T1, T2, T3, T4, T5);
_impl_tuple!(7; T0, T1, T2, T3, T4, T5, T6);
_impl_tuple!(8; T0, T1, T2, T3, T4, T5, T6, T7);
_impl_tuple!(9; T0, T1, T2, T3, T4, T5, T6, T7, T8);
_impl_tuple!(10; T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
_impl_tuple!(11; T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
_impl_tuple!(12; T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
_impl_tuple!(13; T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
_impl_tuple!(14; T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
_impl_tuple!(15; T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::mp::decode::{Object, ObjectParser};

    fn params(bytes: &'static [u8]) -> Array {
        let Object::Array(array) = ObjectParser::new(Bytes::from_static(bytes))
            .next_object()
            .unwrap()
        else {
            panic!("expected an array");
        };
        array
    }

    #[test]
    fn empty_params_require_no_arguments() {
        assert_eq!(<()>::from_params(params(b"\x90")), Ok(()));
        assert_eq!(
            <()>::from_params(params(b"\x91\xc0")),
            Err(DecodeError::InvalidLength {
                expected: 0,
                actual: 1,
            })
        );
    }

    #[test]
    fn tuples_decode_each_argument_and_require_exact_arity() {
        assert_eq!(
            <(String, i64, bool)>::from_params(params(b"\x93\xa2hi\xff\xc3")),
            Ok(("hi".into(), -1, true))
        );
        for (bytes, actual) in [(&b"\x90"[..], 0), (&b"\x92\x01\x02"[..], 2)] {
            assert_eq!(
                <(u64,)>::from_params(params(bytes)),
                Err(DecodeError::InvalidLength {
                    expected: 1,
                    actual,
                })
            );
        }
        assert_eq!(
            <(bool,)>::from_params(params(b"\x91\x01")),
            Err(DecodeError::UnexpectedType("boolean"))
        );
    }

    #[test]
    fn raw_params_preserve_lazy_arguments() {
        let mut params = Array::from_params(params(b"\x92\x01\xc0")).unwrap();
        assert_eq!(params.remaining_len(), 2);
        assert!(matches!(params.next(), Some(Object::U8(1))));
        assert!(matches!(params.next(), Some(Object::Nil)));
    }
}
