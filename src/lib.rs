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
//! input-stream = "0.2.0"
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

#![recursion_limit = "1024"]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![deny(fat_ptr_transmutes,
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
        warnings)]

#[macro_use]
extern crate error_chain;

use std::io::{self, Read, BufRead};
use std::str::{self, FromStr};

/// errors sub-module made with [error-chain](https://crates.io/crates/error-chain)
pub mod errors {
    error_chain!{}
}

pub use errors::Result;
use errors::ResultExt;

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
        b' ' |
        b'\x09'...b'\x0d' => true,
        _ => false,
    }
}

fn act_while<T, F, G>(reader: &mut T, mut condition: F, mut act: G) -> io::Result<()>
    where T: BufRead,
          F: FnMut(&&u8) -> bool,
          G: FnMut(&[u8])
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
            reader: reader,
            byte_buffer: Vec::new(),
        }
    }

    /// Scan the underlying buffered reader for a value of a type that implements
    /// [`std::str::FromStr`](https://doc.rust-lang.org/std/str/trait.FromStr.html)
    /// returning a [`Result`](errors/type.Result.html).
    ///
    /// An example on how to use scan is at the [`crate documentation`](index.html).
    pub fn scan<F>(&mut self) -> Result<F>
        where F: FromStr,
              <F as FromStr>::Err: std::error::Error + Send + 'static
    {
        let &mut InputStream { ref mut reader, ref mut byte_buffer } = self;
        act_while(reader, |&&c| is_whitespace(c), |_| {}).chain_err(|| "IO Error")?;
        byte_buffer.clear();
        act_while(reader,
                  |&&c| !is_whitespace(c),
                  |slice| byte_buffer.extend_from_slice(slice)).chain_err(|| "IO Error")?;

        let slice = match byte_buffer.split_last() {
            Some((&b' ', slice)) => slice,
            _ => byte_buffer.as_slice(),
        };

        str::from_utf8(slice)
            .chain_err(|| "Input data not Utf-8")?
            .parse::<F>()
            .chain_err(|| "Could not parse string into requested type")
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
        assert_eq!(true,
                   (12.5 - stream.scan::<f32>().expect("12.5")).abs() < EPS);
        assert_eq!(true,
                   (-2.85 - stream.scan::<f32>().expect("-2.85")).abs() < EPS);
    }

    #[test]
    fn newlines() {
        let text = "12\nHello";
        let mut stream = InputStream::new(text.as_bytes());
        assert_eq!(12, stream.scan().expect("12"));
        assert_eq!("Hello", stream.scan::<String>().expect("Hello"));
    }
}
