use crate::streaming::{Sink, Source};
use std::{
    ffi::OsString,
    io::{self, Read, Write},
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::PathBuf,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("protocol read an invalid version from the stream")]
    InvalidVersion(u16),

    #[error("crash log path is too long to be written to the stream")]
    CrashLogPathTooLong,
}

pub type Result<T> = ::core::result::Result<T, Error>;

// A brief overview of the message format:
// struct {
//   version: u16 <-- 1
//   crash_log_path_len: u32
//   crash_log_path: [u16; crash_log_path_len] <-- potentially ill-formed utf-16 encoded path to the crash log
// };
pub struct Eulogy {
    pub crash_log_path: PathBuf,
}

impl Eulogy {
    pub fn read_from<R: Read>(stream: &mut R) -> Result<Self> {
        let mut source = Source::new(stream);
        let version: u16 = source.read()?;
        if version < 1 {
            return Err(Error::InvalidVersion(version));
        }

        let crash_log_path_len: u32 = source.read()?;
        let mut crash_log_path = Vec::with_capacity(crash_log_path_len as usize);
        for _ in 0..crash_log_path_len {
            crash_log_path.push(source.read()?);
        }

        let crash_log_path = OsString::from_wide(&crash_log_path[..]);
        let crash_log_path = PathBuf::from(crash_log_path);
        Ok(Self { crash_log_path })
    }

    pub fn write_to<W: Write>(&self, stream: &mut W) -> Result<()> {
        let mut sink = Sink::new(stream);
        sink.write(&1u16)?;

        let crash_log_path = self
            .crash_log_path
            .as_os_str()
            .encode_wide()
            .collect::<Vec<_>>();
        let crash_log_path_len =
            u32::try_from(crash_log_path.len()).map_err(|_| Error::CrashLogPathTooLong)?;
        sink.write(&crash_log_path_len)?;
        for x in &crash_log_path {
            sink.write(x)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context as _;
    use std::io::Cursor;

    #[test]
    fn test_roundtrip() -> anyhow::Result<()> {
        let mut stream = Cursor::new(Vec::new());
        let old_eulogy = Eulogy {
            crash_log_path: PathBuf::from("hello_world/and/all_who/inhabit.it"),
        };

        old_eulogy
            .write_to(&mut stream)
            .context("Failed to write eulogy to stream")?;
        let old_pos = stream.position();

        stream.set_position(0);
        let new_eulogy =
            Eulogy::read_from(&mut stream).context("Failed to read eulogy from stream")?;
        let new_pos = stream.position();

        assert_eq!(old_eulogy.crash_log_path, new_eulogy.crash_log_path);
        assert_eq!(old_pos, new_pos);

        Ok(())
    }
}
