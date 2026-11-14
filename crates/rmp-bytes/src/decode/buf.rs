use std::collections::VecDeque;
use std::ops::{Range, RangeBounds, RangeTo};

#[cfg(feature = "bytes")]
use bytes::{Bytes, BytesMut};

#[allow(clippy::len_without_is_empty)]
/// Byte storage with indexed access, including noncontiguous storage.
///
/// Reading or copying bytes leaves the storage unchanged.
pub trait SliceLike {
    fn len(&self) -> usize;

    /// Returns the byte at `idx`, or `None` if it is out of bounds.
    fn get(&self, idx: usize) -> Option<u8>;

    /// Copies `&self[idx..(idx + dst.len())]` to `dst`.
    ///
    /// May panic if `self.len() < idx + dst.len()`.
    fn copy_to(&self, dst: &mut [u8], idx: usize);
}

impl<T: SliceLike + ?Sized> SliceLike for &T {
    fn len(&self) -> usize {
        T::len(self)
    }

    fn get(&self, idx: usize) -> Option<u8> {
        T::get(self, idx)
    }

    fn copy_to(&self, dst: &mut [u8], idx: usize) {
        T::copy_to(self, dst, idx);
    }
}

impl SliceLike for [u8] {
    fn len(&self) -> usize {
        <[_]>::len(self)
    }

    fn get(&self, idx: usize) -> Option<u8> {
        <[_]>::get(self, idx).copied()
    }

    fn copy_to(&self, dst: &mut [u8], idx: usize) {
        dst.copy_from_slice(&self[idx..(idx + dst.len())]);
    }
}

impl SliceLike for Vec<u8> {
    fn len(&self) -> usize {
        <[_]>::len(self)
    }

    fn get(&self, idx: usize) -> Option<u8> {
        <[_] as SliceLike>::get(self, idx)
    }

    fn copy_to(&self, dst: &mut [u8], idx: usize) {
        <[_] as SliceLike>::copy_to(self, dst, idx);
    }
}

impl SliceLike for VecDeque<u8> {
    fn len(&self) -> usize {
        <VecDeque<_>>::len(self)
    }

    fn get(&self, idx: usize) -> Option<u8> {
        <VecDeque<_>>::get(self, idx).copied()
    }

    fn copy_to(&self, dst: &mut [u8], idx: usize) {
        let src = self.range(idx..(idx + dst.len()));
        let dst = dst.iter_mut();
        for (&src, dst) in src.zip(dst) {
            *dst = src;
        }
    }
}

#[allow(clippy::len_without_is_empty)]
/// A byte cursor that can return views of its remaining storage.
///
/// Reading and advancing consume bytes from the cursor. Slicing leaves the
/// cursor unchanged and returns another buffer, which may borrow or share storage.
/// The bytes need not be contiguous.
///
/// # Examples
///
/// Moving the cursor:
///
/// ```
/// use rmp_bytes::decode::Buf;
///
/// let mut buf = b"Hello, World!".as_slice();
/// let buf = &mut buf;
/// assert_eq!(Buf::len(buf), 13);
/// assert_eq!(buf.first_byte(), Some(b'H'));
/// assert_eq!(buf, b"ello, World!");
///
/// let mut copy_dst = [0u8; 4];
/// Buf::copy_to_slice(buf, &mut copy_dst);
/// assert_eq!(&copy_dst, b"ello");
/// assert_eq!(buf, b", World!");
///
/// Buf::advance(buf, 8);
/// assert_eq!(buf, b"");
/// assert_eq!(buf.first_byte(), None);
/// ```
///
/// Take a view without advancing the cursor:
///
/// ```
/// use rmp_bytes::decode::Buf;
///
/// let buf = b"Hello, World!".as_slice();
/// assert_eq!(Buf::slice(&buf, ..5), b"Hello");
/// assert_eq!(buf, b"Hello, World!");
/// ```
pub trait Buf {
    /// A view of a prefix of this buffer.
    type Slice: Buf;

    /// Returns a view of the first `range.end` remaining bytes.
    ///
    /// The range must fit in the buffer. Implementations may panic otherwise.
    fn slice(&self, range: RangeTo<usize>) -> Self::Slice;

    /// Total length in bytes.
    fn len(&self) -> usize;

    /// Consumes the first byte in the buffer. Returns `None` if it is empty.
    fn first_byte(&mut self) -> Option<u8>;

    /// Copies the first `dst.len()` bytes to `dst` and consumes them.
    ///
    /// The buffer must contain at least `dst.len()` bytes. Implementations may
    /// panic otherwise.
    fn copy_to_slice(&mut self, dst: &mut [u8]);

    /// Consumes `by` bytes.
    ///
    /// The buffer must contain at least `by` bytes. Implementations may panic
    /// otherwise.
    fn advance(&mut self, by: usize);
}

impl<T: Buf> Buf for &mut T {
    type Slice = T::Slice;

    fn slice(&self, range: RangeTo<usize>) -> Self::Slice {
        T::slice(self, range)
    }

    fn len(&self) -> usize {
        T::len(self)
    }

    fn first_byte(&mut self) -> Option<u8> {
        T::first_byte(self)
    }

    fn copy_to_slice(&mut self, dst: &mut [u8]) {
        T::copy_to_slice(self, dst)
    }

    fn advance(&mut self, by: usize) {
        T::advance(self, by)
    }
}

impl Buf for &[u8] {
    type Slice = Self;

    fn slice(&self, range: RangeTo<usize>) -> Self::Slice {
        &self[range]
    }

    fn len(&self) -> usize {
        <[_]>::len(self)
    }

    fn first_byte(&mut self) -> Option<u8> {
        if let Some(&byte) = self.first() {
            *self = &self[1..];
            Some(byte)
        } else {
            None
        }
    }

    fn copy_to_slice(&mut self, dst: &mut [u8]) {
        dst.copy_from_slice(&self[..dst.len()]);
        *self = &self[dst.len()..];
    }

    fn advance(&mut self, by: usize) {
        *self = &self[by..];
    }
}

/// Tracks a range within [`SliceLike`] storage.
///
/// `SliceCursor<&S>` implements [`Buf`]. Advancing changes only the range;
/// returned views retain a reference to the same storage.
///
/// # Examples
///
/// ```
/// use rmp_bytes::decode::buf::{Buf, SliceCursor};
///
/// let bytes = b"Hello, World!".to_vec();
/// let mut buf = SliceCursor::new(&bytes, ..);
/// assert_eq!(Buf::len(&buf), 13);
/// assert_eq!(buf.first_byte(), Some(b'H'));
///
/// let mut buf = SliceCursor::new(&bytes, 5..7);
/// assert_eq!(Buf::len(&buf), 2);
/// assert_eq!(buf.first_byte(), Some(b','));
/// ```
#[derive(Debug)]
pub struct SliceCursor<S> {
    pub slice: S,
    pub range: Range<usize>,
}

impl<S: SliceLike> SliceCursor<S> {
    /// Creates a cursor over `range`, clamping its end to the storage length.
    ///
    /// # Panics
    ///
    /// Panics if the start exceeds the clamped end, or an excluded start bound
    /// overflows `usize`.
    pub fn new(slice: S, range: impl RangeBounds<usize>) -> Self {
        use std::cmp::min;
        use std::ops::Bound;

        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n.checked_add(1).expect("range start overflow"),
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(&n) => min(n.saturating_add(1), slice.len()),
            Bound::Excluded(&n) => min(n, slice.len()),
            Bound::Unbounded => slice.len(),
        };

        assert!(start <= end);

        Self {
            slice,
            range: start..end,
        }
    }
}

impl<S: SliceLike + ?Sized> Buf for SliceCursor<&S> {
    type Slice = Self;

    fn slice(&self, range: RangeTo<usize>) -> Self::Slice {
        let start = self.range.start;
        let end = std::cmp::min(start + range.end, self.range.end);
        Self {
            slice: self.slice,
            range: start..end,
        }
    }

    fn len(&self) -> usize {
        self.range.len()
    }

    fn first_byte(&mut self) -> Option<u8> {
        if self.range.is_empty() {
            return None;
        }
        let ret = self.slice.get(self.range.start)?;
        self.advance(1);
        Some(ret)
    }

    fn copy_to_slice(&mut self, dst: &mut [u8]) {
        assert!(self.range.len() >= dst.len());
        self.slice.copy_to(dst, self.range.start);
        self.advance(dst.len());
    }

    fn advance(&mut self, by: usize) {
        assert!(by <= self.range.len());
        self.range.start += by;
    }
}

#[cfg(feature = "bytes")]
impl Buf for Bytes {
    type Slice = Self;

    fn slice(&self, range: RangeTo<usize>) -> Self::Slice {
        <Bytes>::slice(self, range)
    }

    fn len(&self) -> usize {
        <Bytes>::len(self)
    }

    fn first_byte(&mut self) -> Option<u8> {
        let &ret = self.first()?;
        Buf::advance(self, 1);
        Some(ret)
    }

    fn copy_to_slice(&mut self, dst: &mut [u8]) {
        dst.copy_from_slice(&self[..dst.len()]);
        Buf::advance(self, dst.len());
    }

    fn advance(&mut self, by: usize) {
        use bytes::Buf;
        Buf::advance(self, by);
    }
}

/// Storage that can lend a cursor for finding a complete value's boundary.
///
/// Use [`seek_to_eoo()`](crate::decode::seek_to_eoo()) on the returned cursor
/// without consuming the original storage. This also supports containers such as
/// `VecDeque<u8>` that do not implement [`Buf`] themselves.
///
/// # Examples
///
/// ```
/// use rmp_bytes::decode::{Seekable, seek_to_eoo};
/// use rmp_bytes::rmp::Marker;
///
/// use std::num::NonZeroUsize;
///
/// let s = "Hello, World!";
///
/// let mut bytes = [Marker::Str8.to_u8(), s.len() as u8].to_vec();
/// bytes.extend_from_slice(s.as_bytes());
///
/// let expected = NonZeroUsize::new(s.len() + 2).unwrap();
/// assert_eq!(seek_to_eoo(&mut bytes.seekable()), Ok(expected));
/// ```
pub trait Seekable {
    /// Returns a cursor over the current bytes, leaving this storage unchanged.
    fn seekable(&self) -> impl Buf + '_;
}

impl<T: Buf> Seekable for T {
    fn seekable(&self) -> impl Buf + '_ {
        self.slice(..self.len())
    }
}

impl Seekable for [u8] {
    fn seekable(&self) -> impl Buf + '_ {
        self
    }
}

impl Seekable for Vec<u8> {
    fn seekable(&self) -> impl Buf + '_ {
        &**self
    }
}

impl Seekable for VecDeque<u8> {
    fn seekable(&self) -> impl Buf + '_ {
        SliceCursor {
            slice: self,
            range: 0..self.len(),
        }
    }
}

#[cfg(feature = "bytes")]
impl Seekable for BytesMut {
    fn seekable(&self) -> impl Buf + '_ {
        &**self
    }
}
