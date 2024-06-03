use std::borrow::Borrow;
use std::io;
/// A munger which XORs a key with some data
#[derive(Clone)]
pub struct Xorcism<'a> {
    // This field is just to suppress compiler complaints;
    // feel free to delete it at any point.
    // _phantom: std::marker::PhantomData<&'a u8>,
    key: &'a [u8],
    key_idx: usize,
}

impl<'a> Xorcism<'a> {
    /// Create a new Xorcism munger from a key
    ///
    /// Should accept anything which has a cheap conversion to a byte slice.
    pub fn new<Key: ?Sized + AsRef<[u8]>>(key: &'a Key) -> Xorcism<'a> {
        Xorcism {
            key: key.as_ref(),
            key_idx: 0,
        }
    }

    /// XOR each byte of the input buffer with a byte from the key.
    ///
    /// Note that this is stateful: repeated calls are likely to produce different results,
    /// even with identical inputs.
    pub fn munge_in_place(&mut self, data: &mut [u8]) {
        for c in data {
            *c = *c ^ self.key[self.key_idx];
            self.key_idx = (self.key_idx + 1) % (self.key.len());
        }
    }

    /// XOR each byte of the data with a byte from the key.
    ///
    /// Note that this is stateful: repeated calls are likely to produce different results,
    /// even with identical inputs.
    ///
    /// Should accept anything which has a cheap conversion to a byte iterator.
    /// Shouldn't matter whether the byte iterator's values are owned or borrowed.
    pub fn munge<Data: IntoIterator<Item = impl Borrow<u8>>>(
        &mut self,
        data: Data,
    ) -> impl Iterator<Item = u8> {
        let mut v: Vec<u8> = Vec::new();
        for d in data {
            v.push(d.borrow() ^ self.key[self.key_idx]);
            self.key_idx = (self.key_idx + 1) % (self.key.len());
        }
        v.into_iter()
    }
    pub fn reader<R : io::Read>(self, reader : R) -> Reader<'a,R>{
        Reader { xorr: self, reader}
    }
    pub fn writer<W : io::Write>(self, writer : W) -> Writer<'a,W>{
        Writer { xorr: self, writer}
    }
}
pub struct Reader<'a, R: io::Read> {
    xorr: Xorcism<'a>,
    reader: R,
}

impl<'a ,R: io::Read> io::Read for Reader<'a,R>{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = self.reader.read(buf)?;
        self.xorr.munge_in_place(&mut buf[..read]);
        Ok(read)
    }
}
pub struct Writer<'a, W: io::Write>{
    xorr : Xorcism<'a>,
    writer : W
}

impl<'a,W:io::Write> io::Write for Writer<'a,W>{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut buf = buf.to_vec();
        self.xorr.munge_in_place( &mut buf);
        self.writer.write(&buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}