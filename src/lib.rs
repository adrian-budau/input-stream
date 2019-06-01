//! A small utility library for input parsing in a manner akin to _C++_'s istream.
//!
//! It exposes a single struct [`InputStream`](struct.InputStream.html) which is wrapped around
//! any object that implements
//! [`std::io::BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html).
//!
//! It can parse any type which implements
//! [`std::str::FromStr`](https://doc.rust-lang.org/std/str/trait.FromStr.html).
//!
//! # Usage
//!
//! This crate is [on crates.io](https://crates.io/crates/input-stream) and can be used
//! by adding `input-stream` to the dependencies in your project's `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! input-stream = "0.3.0"
//! ```
//!
//! and this in your crate root:
//!
//! ```rust
//! extern crate input_stream;
//! ```
//!
//! # Examples:
//!
//! ```rust
//! use std::io;
//! use input_stream::InputStream;
//!
//! # let buf_reader = "test".as_bytes();
//! # use std::io::BufRead;
//! let mut input = InputStream::new(buf_reader);
//! let value = input.scan::<bool>();
//! match value {
//!     Ok(value) => println!("Successfully read boolean: {}", value),
//!     Err(err) => println!("Error reading value: {:?}", err)
//! }
//! ```
//!
//! ## Reading from standard input:
//!
//! ```rust,no_run
//! use std::io;
//! use input_stream::InputStream;
//!
//! let stdin = io::stdin();
//! let mut input = InputStream::new(stdin.lock());
//!
//! let integer: i32 = input.scan().expect("An integer");
//! let string: String = input.scan().expect("A string");
//!
//! println!("Read the number: {} and the string {}", integer, string);
//! ```
//!
//! ## or from a file
//!
//! ```rust,no_run
//! use std::io::{self, BufReader};
//! use std::fs::File;
//! use input_stream::InputStream;
//!
//! let mut input = InputStream::new(
//!     BufReader::new(File::open("name_of_file.txt").expect("a file named name_of_file.txt")));
//!
//! let value: f32 = input.scan().expect("A floating point number");
//!
//! println!("Read a float: {}", value);
//!

#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    variant_size_differences,
    clippy::all,
    warnings
)]

use std::fmt::{self, Debug, Display, Formatter};
use std::io::{self, BufRead, Read};
use std::str::{self, FromStr};

/// The type of errors this library can return.
#[derive(Debug)]
pub enum Error<E> {
    /// I/O Error
    Io(io::Error),
    /// Data is not valid utf8
    Utf8(str::Utf8Error),
    /// Could not parse given data type
    FromStr(E),
    /// Buffer limit exceeded
    BufferLimitExceeded,
}

/// A specialized [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html) for this
/// library's errors.
pub type Result<T, E = Error<<T as FromStr>::Err>> = std::result::Result<T, E>;

impl<E> From<io::Error> for Error<E> {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl<E> From<str::Utf8Error> for Error<E> {
    fn from(err: str::Utf8Error) -> Self {
        Error::Utf8(err)
    }
}

impl<E> Display for Error<E> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Error::Io(_) => write!(fmt, "I/O Error"),
            Error::Utf8(_) => write!(fmt, "Data is not valid utf8"),
            Error::FromStr(_) => write!(fmt, "Could not parse given data type"),
            Error::BufferLimitExceeded => write!(fmt, "Buffer limit exceeded"),
        }
    }
}

impl<E: Debug> std::error::Error for Error<E> {}

/// A wrapper for [`std::io::BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html).
///
/// To get an instance of this  struct use static method [`new`](struct.InputStream.html#method.new) on
/// `InputStream`.
#[derive(Debug)]
pub struct InputStream<T: BufRead> {
    reader: T,
    byte_buffer: Vec<u8>,
}

#[inline(always)]
fn is_whitespace(c: u8) -> bool {
    match c {
        b' ' | b'\x09'...b'\x0d' => true,
        _ => false,
    }
}

#[inline(always)]
fn act_while<T, F, G, E>(reader: &mut T, mut condition: F, mut act: G) -> Result<(), Error<E>>
where
    T: BufRead,
    F: FnMut(&&u8) -> bool,
    G: FnMut(&[u8]) -> Result<(), Error<E>>,
{
    loop {
        let (skipped, done) = match reader.fill_buf() {
            Ok(buf) => {
                let skipped = buf.iter().take_while(&mut condition).count();
                act(&buf[..skipped])?;
                (skipped, skipped < buf.len() || buf.is_empty())
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        };

        reader.consume(skipped);
        if done {
            break;
        }
    }
    Ok(())
}

impl<T: BufRead> InputStream<T> {
    /// Creates an instance of InputStream which wraps the given
    /// [`std::io::BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html).
    #[inline(always)]
    pub fn new(reader: T) -> InputStream<T> {
        InputStream {
            reader,
            byte_buffer: Vec::new(),
        }
    }

    /// Scan the underlying buffered reader for a value of a type that implements
    /// [`std::str::FromStr`](https://doc.rust-lang.org/std/str/trait.FromStr.html)
    /// returning a [`Result`](type.Result.html).
    ///
    /// An example on how to use scan is at the [`crate documentation`](index.html).
    pub fn scan<F: FromStr>(&mut self) -> Result<F> {
        self.inner_scan(None)
    }

    /// Scan the underlying buffer reader for a value of a type that implements
    /// [`std::str::FromStr`](https://doc.rust-lang.org/std/str/trait.FromStr.html)
    /// returning a [`Result`](type.Result.html).
    ///
    /// This is a refined version of [`scan`](struct.InputStream.html#method.scan) which allows
    /// limits to be placed on the maximum size of the internal buffer
    pub fn scan_with_limit<F: FromStr>(&mut self, limit: usize) -> Result<F> {
        self.inner_scan(Some(limit))
    }

    #[inline(always)]
    fn inner_scan<F: FromStr>(&mut self, limit: Option<usize>) -> Result<F> {
        let &mut InputStream {
            ref mut reader,
            ref mut byte_buffer,
        } = self;
        act_while(reader, |&&c| is_whitespace(c), |_| Ok(()))?;
        byte_buffer.clear();
        act_while(
            reader,
            |&&c| !is_whitespace(c),
            |slice| {
                if let Some(limit) = limit {
                    if byte_buffer.len() + slice.len() > limit {
                        return Err(Error::BufferLimitExceeded);
                    }
                }

                byte_buffer.extend_from_slice(slice);
                Ok(())
            },
        )?;

        let slice = match byte_buffer.split_last() {
            Some((&b' ', slice)) => slice,
            _ => byte_buffer.as_slice(),
        };

        str::from_utf8(slice)?.parse().map_err(Error::FromStr)
    }
}

impl<T: BufRead> Read for InputStream<T> {
    #[inline(always)]
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buffer)
    }
}

impl<T: BufRead> BufRead for InputStream<T> {
    #[inline(always)]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.reader.fill_buf()
    }

    #[inline(always)]
    fn consume(&mut self, amount: usize) {
        self.reader.consume(amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const EPS: f32 = 1e-6;

    #[test]
    fn simple_strings() {
        let text = "Howdy neighbour, how are you doing?";
        let mut stream = InputStream::new(text.as_bytes());

        let first: String = stream.scan().expect("First string");
        let second: String = stream.scan().expect("Second string");
        let third: String = stream.scan().expect("Third string");
        assert_eq!(first, "Howdy");
        assert_eq!(second, "neighbour,");
        assert_eq!(third, "how");
    }

    #[test]
    fn simple_numbers() {
        let text = "5 -7 12.5 -2.85";
        let mut stream = InputStream::new(text.as_bytes());
        assert_eq!(5, stream.scan().expect("5"));
        assert_eq!(-7, stream.scan().expect("-7"));
        assert_eq!(
            true,
            (12.5 - stream.scan::<f32>().expect("12.5")).abs() < EPS
        );
        assert_eq!(
            true,
            (-2.85 - stream.scan::<f32>().expect("-2.85")).abs() < EPS
        );
    }

    #[test]
    fn newlines() {
        let text = "12\nHello";
        let mut stream = InputStream::new(text.as_bytes());
        assert_eq!(12, stream.scan().expect("12"));
        assert_eq!("Hello", stream.scan::<String>().expect("Hello"));
    }

    #[test]
    fn test_non_utf8() {
        let text: [u8; 1] = [255];
        let mut stream = InputStream::new(&text[..]);
        assert_eq!(true, stream.scan::<i32>().is_err());
    }

    #[test]
    fn test_not_parsing() {
        let text = "hello";
        let mut stream = InputStream::new(text.as_bytes());
        assert_eq!(true, stream.scan::<i32>().is_err());
    }

    #[test]
    fn test_limit_buffer() {
        let text = "25 150 -250";
        let mut stream = InputStream::new(text.as_bytes());
        assert_eq!(25, stream.scan_with_limit(3).expect("25"));
        assert_eq!(150, stream.scan_with_limit(3).expect("150"));
        assert!(stream.scan_with_limit::<i32>(3).is_err());
    }
}
