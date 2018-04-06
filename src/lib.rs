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

#![deny(missing_copy_implementations, missing_debug_implementations, missing_docs, trivial_casts,
        trivial_numeric_casts, unsafe_code, unused_extern_crates, unused_import_braces,
        unused_qualifications, unused_results, variant_size_differences, warnings)]
#![cfg_attr(feature = "cargo-clippy", deny(clippy))]
#[macro_use]
extern crate failure;

use failure::{Backtrace, Context, Fail, ResultExt};
use std::fmt::{self, Display, Formatter};
use std::io::{self, BufRead, Read};
use std::result;
use std::str::{self, FromStr};

/// The type of errors this library can return.
///
/// A kind can be obtained from an [`Error`](struct.Error.html)
/// by calling its [`kind`](struct.Error.html#method.kind) method.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    /// Could not read from the underlying buffered reader.
    #[fail(display = "IO error")]
    Io,
    /// Could not parse the byte buffer into valid Utf8.
    #[fail(display = "Input data is not utf8")]
    Utf8,
    /// Could not parse the utf8 string into the requested type.
    #[fail(display = "Could not parse string as type")]
    Parse,
}

/// The type of the errors returned by this library.
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Error {
    /// Returns the corresponding [`ErrorKind`](enum.ErrorKind.html) for this error.
    pub fn kind(&self) -> ErrorKind {
        *self.inner.get_context()
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}

/// A specialized [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html) for this
/// library's errors.
pub type Result<T> = result::Result<T, Error>;

/// A wrapper for [`std::io::BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html).
///
/// To get an instance of this  struct use static method [`new`](struct.InputStream.html#method.new) on
/// `InputStream`.
#[derive(Debug)]
pub struct InputStream<T: BufRead> {
    reader: T,
    byte_buffer: Vec<u8>,
}

fn is_whitespace(c: u8) -> bool {
    match c {
        b' ' | b'\x09'...b'\x0d' => true,
        _ => false,
    }
}

fn act_while<T, F, G>(reader: &mut T, mut condition: F, mut act: G) -> io::Result<()>
where
    T: BufRead,
    F: FnMut(&&u8) -> bool,
    G: FnMut(&[u8]),
{
    loop {
        let (skipped, done) = match reader.fill_buf() {
            Ok(buf) => {
                let skipped = buf.iter().take_while(&mut condition).count();
                act(&buf[..skipped]);
                (skipped, skipped < buf.len() || buf.is_empty())
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
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
    pub fn scan<F>(&mut self) -> Result<F>
    where
        F: FromStr,
        <F as FromStr>::Err: Fail,
    {
        let &mut InputStream {
            ref mut reader,
            ref mut byte_buffer,
        } = self;
        act_while(reader, |&&c| is_whitespace(c), |_| {}).context(ErrorKind::Io)?;
        byte_buffer.clear();
        act_while(
            reader,
            |&&c| !is_whitespace(c),
            |slice| byte_buffer.extend_from_slice(slice),
        ).context(ErrorKind::Io)?;

        let slice = match byte_buffer.split_last() {
            Some((&b' ', slice)) => slice,
            _ => byte_buffer.as_slice(),
        };

        Ok(str::from_utf8(slice)
            .context(ErrorKind::Utf8)?
            .parse::<F>()
            .context(ErrorKind::Parse)?)
    }
}

impl<T: BufRead> Read for InputStream<T> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buffer)
    }
}

impl<T: BufRead> BufRead for InputStream<T> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.reader.fill_buf()
    }

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
}
