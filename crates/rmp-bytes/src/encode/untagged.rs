use super::Encode;
use super::primitive::{Sealed, State};

/// Encodes only the selected payload, without adding a branch tag or container.
///
/// `Left(7i64)` encodes the integer `7`; `Right("text")` encodes the string
/// `"text"`. Neither the variant name nor its position appears on the wire.
/// Nest this type to compose more alternatives; nesting does not add wire data.
///
/// ```
/// use rmp_bytes::encode::{Untagged, WriterSyncExt as _};
///
/// let value: Untagged<i64, &str> = Untagged::Right("text");
/// let mut bytes = Vec::new();
/// bytes.write_value(value).consume();
/// assert_eq!(bytes, b"\xa4text");
/// ```
///
/// This is an encoding composition, not a [`super::Primitive`]:
///
/// ```compile_fail
/// use rmp_bytes::encode::{Primitive, Untagged};
/// fn primitive<T: Primitive>() {}
/// primitive::<Untagged<i64, String>>();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Untagged<L, R> {
    /// Selects the first payload's encoding.
    Left(L),
    /// Selects the second payload's encoding.
    Right(R),
}

impl<L, R> Sealed for Untagged<L, R> {}
impl<L: Encode, R: Encode> Encode for Untagged<L, R> {
    type Encoder = Untagged<L::Encoder, R::Encoder>;

    fn into_encoder(self) -> Self::Encoder {
        match self {
            Self::Left(value) => Untagged::Left(value.into_encoder()),
            Self::Right(value) => Untagged::Right(value.into_encoder()),
        }
    }
}

impl<L, R> Sealed for &Untagged<L, R> {}
impl<'s, L, R> Encode for &'s Untagged<L, R>
where
    &'s L: Encode,
    &'s R: Encode,
{
    type Encoder = Untagged<<&'s L as Encode>::Encoder, <&'s R as Encode>::Encoder>;

    fn into_encoder(self) -> Self::Encoder {
        match self {
            Untagged::Left(value) => Untagged::Left(value.into_encoder()),
            Untagged::Right(value) => Untagged::Right(value.into_encoder()),
        }
    }
}

impl<L: State, R: State> State for Untagged<L, R> {
    type Item = Untagged<L::Item, R::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Left(state) => state.next().map(Untagged::Left),
            Self::Right(state) => state.next().map(Untagged::Right),
        }
    }
}

impl<L: AsRef<[u8]>, R: AsRef<[u8]>> AsRef<[u8]> for Untagged<L, R> {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Left(bytes) => bytes.as_ref(),
            Self::Right(bytes) => bytes.as_ref(),
        }
    }
}
