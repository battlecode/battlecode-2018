use std::net::TcpStream;
use std::io::{Read, Write, ErrorKind};
use std::path::Path;
use std::convert::AsRef;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{to_writer, from_slice};
use failure::Error;
use std::mem;
use std::thread;
use std::time::Duration;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

pub struct Stream<S: Read + Write> {
    stream: S,
    // can't use BufReader because searching by lines is hard
    buf: Vec<u8>
}

impl<S: Read + Write> Stream<S> {
    pub(crate) fn new(stream: S) -> Stream<S> {
        Stream {
            stream,
            buf: vec![]
        }
    }

    pub(crate) fn read<T: DeserializeOwned>(&mut self) -> Result<T, Error> {
        let newline = self.buf.iter().position(|&b| b == b'\n');
        if let Some(idx) = newline {
            let t = from_slice(&self.buf[..idx]);
            self.buf.drain(..idx+1);
            return Ok(t?);
        }

        let mut buf: [u8; 256] = unsafe { mem::uninitialized() };

        loop {
            let len = self.stream.read(&mut buf[..]);
            match len {
                Ok(len) => {
                    if len == 0 {
                        thread::sleep(Duration::new(0, 1_000_000));
                        continue;
                    }

                    let newline = buf[..len].iter().position(|&b| b == b'\n');
                    if let Some(idx) = newline {
                        self.buf.extend(buf[..idx].iter());
                        let t = from_slice(&self.buf[..]);
                        self.buf.clear();
                        self.buf.extend(buf[idx+1..len].iter());
                        return Ok(t?);
                    } else {
                        // no data; continue
                        self.buf.extend(buf[..len].iter())
                    }
                },
                Err(e) => {
                    if e.kind() == ErrorKind::Interrupted {
                        continue;
                    } else {
                        Err(e)?
                    }
                }
            }
        }
    }

    pub(crate) fn write<T: Serialize>(&mut self, value: &T) -> Result<(), Error> {
        to_writer(&mut self.stream, value)?;
        self.stream.write_all(b"\n")?;
        Ok(())
    }
}

#[cfg(unix)]
pub enum Streams {
    UnixStream(Stream<UnixStream>),
    TcpStream(Stream<TcpStream>)
}
#[cfg(windows)]
pub enum Streams {
    // this is just here to keep the compiler happy, we never use it
    UnixStream(Stream<TcpStream>),
    TcpStream(Stream<TcpStream>)
}

impl Streams {
    #[cfg(unix)]
    pub(crate) fn new_unix<P: AsRef<Path>>(path: P) -> Result<Streams, Error> {
        let stream = UnixStream::connect(path)?;
        Ok(Streams::UnixStream(Stream::new(stream)))
    }
    #[cfg(windows)]
    pub(crate) fn new_unix<P: AsRef<Path>>(path: P) -> Result<Streams, Error> {
        bail!("unix streams not supported on windows");
    }

    pub(crate) fn new_tcp(port: u16) -> Result<Streams, Error> {
        let stream = TcpStream::connect(("localhost", port))?;

        Ok(Streams::TcpStream(Stream::new(stream)))
    }
    pub(crate) fn read<T: DeserializeOwned>(&mut self) -> Result<T, Error> {
        match *self {
            Streams::UnixStream(ref mut s) => s.read(),
            Streams::TcpStream(ref mut s) => s.read()
        }
    }
    pub(crate) fn write<T: Serialize>(&mut self, value: &T) -> Result<(), Error> {
        match *self {
            Streams::UnixStream(ref mut s) => s.write(value),
            Streams::TcpStream(ref mut s) => s.write(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixStream;

    #[test]
    fn write_read() {
        let (streama, streamb) = UnixStream::pair().unwrap();
        let mut a = Stream::new(streama);
        let mut b = Stream::new(streamb);

        a.write(&27u8).unwrap();
        assert_eq!(b.read::<u8>().unwrap(), 27u8);
        a.write(&("hello".to_string(), 35i32, "banana".to_string())).unwrap();
        assert_eq!(b.read::<(String, i32, String)>().unwrap(), ("hello".to_string(), 35i32, "banana".to_string()));
        a.write(&("hello".to_string(), 35i32, "banana".to_string())).unwrap();
        a.write(&("hello".to_string(), 35i32, "banana".to_string())).unwrap();
        a.write(&("hello".to_string(), 35i32, "banana".to_string())).unwrap();
        assert_eq!(b.read::<(String, i32, String)>().unwrap(), ("hello".to_string(), 35i32, "banana".to_string()));
        assert_eq!(b.read::<(String, i32, String)>().unwrap(), ("hello".to_string(), 35i32, "banana".to_string()));
        assert_eq!(b.read::<(String, i32, String)>().unwrap(), ("hello".to_string(), 35i32, "banana".to_string()));
        b.write(&35usize).unwrap();
        assert_eq!(a.read::<usize>().unwrap(), 35usize);
    }
}