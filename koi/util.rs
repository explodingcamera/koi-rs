use std::ops::{Deref, DerefMut};

#[inline]
#[cold]
pub fn cold() {}

#[inline]
pub fn likely(b: bool) -> bool {
    if !b {
        cold()
    }
    b
}

#[inline]
pub fn unlikely(b: bool) -> bool {
    if b {
        cold()
    }
    b
}

pub struct Buffer<'a>(&'a [u8]);

impl<'a> Buffer<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self(buf)
    }

    pub fn read_one(self) -> (u8, Self) {
        (self.0[0], Self(&self.0[1..]))
    }

    pub fn read_many(self, n: usize) -> (&'a [u8], Self) {
        (&self.0[..n], Self(&self.0[n..]))
    }

    pub fn read_u32_le(self) -> (u32, Self) {
        let (bytes, buf) = self.read_many(4);
        let n = u32::from_le_bytes(bytes.try_into().expect("4 bytes"));
        (n, buf)
    }

    pub fn advance(self, n: usize) -> Self {
        Self(&self.0[n..])
    }
}

impl<'a> Deref for Buffer<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

pub struct BufferMut<'a>(&'a mut [u8]);

pub trait Writer: Sized {
    fn write_one(self, v: u8) -> Self;
    fn write_many(self, v: &[u8]) -> Self;
    fn capacity(&self) -> usize;
}

impl Deref for BufferMut<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl DerefMut for BufferMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<'a> BufferMut<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self(buffer)
    }

    #[inline]
    fn write_one(self, v: u8) -> Self {
        if let Some((first, tail)) = self.0.split_first_mut() {
            *first = v;
            Self(tail)
        } else {
            unreachable!()
        }
    }

    #[inline]
    fn write_many(self, v: &[u8]) -> Self {
        if v.len() <= self.0.len() {
            let (head, tail) = self.0.split_at_mut(v.len());
            head.copy_from_slice(v);
            Self(tail)
        } else {
            unreachable!()
        }
    }

    pub fn advance(self, n: usize) -> Self {
        if n <= self.0.len() {
            let (_, tail) = self.0.split_at_mut(n);
            Self(tail)
        } else {
            unreachable!()
        }
    }
}

impl<'a> Writer for BufferMut<'a> {
    #[inline]
    fn write_one(self, v: u8) -> Self {
        self.write_one(v)
    }

    #[inline]
    fn write_many(self, v: &[u8]) -> Self {
        self.write_many(v)
    }

    #[inline]
    fn capacity(&self) -> usize {
        self.0.len()
    }
}

// impl<'a> Write for Buffer<'a> {
//     #[inline]
//     fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
//         let len = std::cmp::min(buf.len(), self.0.len());
//         self.0[..len].copy_from_slice(&buf[..len]);
//         // self.0 = &mut self.0[len..];
//         *self = self.trim_start(len);
//         Ok(len)
//     }

//     #[inline]
//     fn flush(&mut self) -> std::io::Result<()> {
//         Ok(())
//     }
// }
