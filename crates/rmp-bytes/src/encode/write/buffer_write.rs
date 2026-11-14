use super::WriterAsync;
use super::WriterSync;

impl WriterSync for Vec<u8> {
    type Result = ();

    fn write_all(&mut self, buf: &[u8]) -> Self::Result {
        self.extend_from_slice(buf);
    }

    fn flush(&mut self) {}
}

impl WriterAsync for Vec<u8> {
    type Result = <Self as WriterSync>::Result;
    async fn write_all(&mut self, buf: &[u8]) -> Self::Result {
        <Self as WriterSync>::write_all(self, buf)
    }

    async fn flush(&mut self) {}
}

#[cfg(feature = "bytes")]
pub struct BufWriter<B> {
    inner: B,
}

#[cfg(feature = "bytes")]
impl<B> BufWriter<B> {
    pub fn new(wtr: B) -> Self {
        Self { inner: wtr }
    }

    pub fn get_ref(&self) -> &B {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut B {
        &mut self.inner
    }

    pub fn into_inner(self) -> B {
        self.inner
    }
}

#[cfg(feature = "bytes")]
impl<B: bytes::BufMut> WriterSync for BufWriter<B> {
    type Result = ();

    fn write_all(&mut self, buf: &[u8]) -> Self::Result {
        self.inner.put_slice(buf);
    }

    fn flush(&mut self) {}
}

#[cfg(feature = "bytes")]
impl<B: bytes::BufMut> WriterAsync for BufWriter<B> {
    type Result = <Self as WriterSync>::Result;
    async fn write_all(&mut self, buf: &[u8]) -> Self::Result {
        <Self as WriterSync>::write_all(self, buf)
    }

    async fn flush(&mut self) {}
}
