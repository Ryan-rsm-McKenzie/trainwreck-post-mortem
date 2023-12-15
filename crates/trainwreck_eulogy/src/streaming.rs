use std::{
    io::{self, Read, Write},
    mem,
};

pub trait BinaryStreamable {
    type Item;
    fn from_stream<R: Read>(stream: &mut R) -> io::Result<Self::Item>;
    fn to_stream<W: Write>(stream: &mut W, item: &Self::Item) -> io::Result<()>;
}

macro_rules! make_binary_streamable {
    ($t:ty) => {
        impl BinaryStreamable for $t {
            type Item = $t;

            fn from_stream<R: Read>(stream: &mut R) -> io::Result<Self::Item> {
                let mut bytes = [0u8; mem::size_of::<Self::Item>()];
                stream.read_exact(&mut bytes)?;
                Ok(Self::from_ne_bytes(bytes))
            }

            fn to_stream<W: Write>(stream: &mut W, item: &Self::Item) -> io::Result<()> {
                let mut bytes = item.to_ne_bytes();
                stream.write_all(&mut bytes)
            }
        }
    };
}

make_binary_streamable!(u8);
make_binary_streamable!(u16);
make_binary_streamable!(u32);
make_binary_streamable!(u64);

make_binary_streamable!(i8);
make_binary_streamable!(i16);
make_binary_streamable!(i32);
make_binary_streamable!(i64);

pub struct Source<'a, R>
where
    R: Read,
{
    stream: &'a mut R,
}

impl<'a, R> Source<'a, R>
where
    R: Read,
{
    pub fn new(stream: &'a mut R) -> Self {
        Self { stream }
    }

    pub fn read<T>(&mut self) -> io::Result<T>
    where
        T: BinaryStreamable<Item = T>,
    {
        T::from_stream(self.stream)
    }
}

pub struct Sink<'a, W>
where
    W: Write,
{
    stream: &'a mut W,
}

impl<'a, W> Sink<'a, W>
where
    W: Write,
{
    pub fn new(stream: &'a mut W) -> Self {
        Self { stream }
    }

    pub fn write<T>(&mut self, item: &T) -> io::Result<()>
    where
        T: BinaryStreamable<Item = T>,
    {
        T::to_stream(self.stream, item)
    }
}
