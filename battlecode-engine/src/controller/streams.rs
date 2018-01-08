use std::os::unix::net::UnixStream;
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

pub struct Streams {
    stream: UnixStream,
    // can't use BufReader because searching by lines is hard
    buf: Vec<u8>
}

impl Streams {
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> Result<Streams, Error> {
        let stream = UnixStream::connect(path)?;
        Ok(Streams {
            stream,
            buf: Vec::new()
        })
    }

    pub(crate) fn read<T: DeserializeOwned>(&mut self) -> Result<T, Error> {
        let newline = self.buf.iter().position(|&b| b == b'\n');
        if let Some(idx) = newline {
            println!("early newline: {}", idx);
            let t = from_slice(&self.buf[..idx]);
            self.buf.drain(..idx+1);
            return Ok(t?);
        }

        println!("reading");
        let mut buf: [u8; 256] = unsafe { mem::uninitialized() };

        loop {
            println!("loop self.buf.len(): {}", self.buf.len());
            let len = self.stream.read(&mut buf[..]);
            match len {
                Ok(len) => {
                    println!("read {}", len);
                    if len == 0 {
                        println!("empty, waiting");
                        thread::sleep(Duration::new(0, 1_000_000));
                        continue;
                    }

                    let newline = buf[..len].iter().position(|&b| b == b'\n');
                    if let Some(idx) = newline {
                        println!("newline {}", idx);
                        self.buf.extend(buf[..idx].iter());
                        let t = from_slice(&self.buf[..]);
                        self.buf.clear();
                        self.buf.extend(buf[idx+1..len].iter());
                        println!("self buf len: {}", self.buf.len());
                        println!("yielding value");
                        return Ok(t?);
                    } else {
                        println!("no newline");
                        // no data; continue
                        self.buf.extend(buf[..len].iter())
                    }
                },
                Err(e) => {
                    println!("err");
                    if e.kind() == ErrorKind::Interrupted {
                        println!("interrupted");
                        continue;
                    } else {
                        println!("some other goddamn thing");
                        Err(e)?
                    }
                }
            }
        }
    }

    pub(crate) fn write<T: Serialize>(&mut self, value: &T) -> Result<(), Error> {
        println!("writing");
        to_writer(&mut self.stream, value)?;
        self.stream.write_all(b"\n")?;
        println!("wrote");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixStream;

    #[test]
    fn write_read() {
        let (streama, streamb) = UnixStream::pair().unwrap();

        let mut a = Streams {
            stream: streama,
            buf: Vec::new()
        };

        let mut b = Streams {
            stream: streamb,
            buf: Vec::new()
        };

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