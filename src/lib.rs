#![recursion_limit = "1024"]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#![feature(test)]
extern crate test;

#[macro_use]
extern crate error_chain;

use std::io::{self, Read, BufRead};
use std::str::{self, FromStr};

mod errors {
    error_chain! { }
}

use errors::*;

pub struct InputStream<T: BufRead> {
    reader: T,
    byte_buffer : Vec<u8>,
}

fn is_whitespace(c: u8) -> bool {
    match c {
        b' ' | b'\x09'...b'\x0d' => true,
        _ => false,
    }
}

fn act_while<T, F, G>(reader: &mut T, mut condition: F, mut act: G) -> io::Result<()> where
    T: BufRead,
    F: FnMut(&&u8) -> bool,
    G: FnMut(&[u8])
{
    loop {
        let (skipped, done) = match reader.fill_buf() {
            Ok(buf) => {
                let skipped = buf.iter().take_while(&mut condition).count();
                act(&buf[..skipped]);
                (skipped, skipped < buf.len() || buf.len() == 0)
            },
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };

        reader.consume(skipped);
        if done {
            break
        }
    }
    Ok(())
}

fn scan<T, F>(reader: &mut T, byte_buffer: &mut Vec<u8>) -> Result<F> where
    T: BufRead,
    F: FromStr,
    <F as FromStr>::Err: std::error::Error + Send + 'static
{
    act_while(reader, |&&c| is_whitespace(c), |_| {})
        .chain_err(|| "IO Error")?;
    byte_buffer.clear();
    act_while(reader, |&&c| !is_whitespace(c), |slice| byte_buffer.extend_from_slice(slice))
        .chain_err(|| "IO Error")?;

    let slice = match byte_buffer.split_last() {
        Some((&b' ', slice)) => slice,
        _ => byte_buffer.as_slice(),
    };

    str::from_utf8(slice)
        .chain_err(|| "Input data not Utf-8")?
        .parse::<F>()
        .chain_err(|| "Could not parse string into requested type")
}

impl<T: BufRead> InputStream<T> {

    pub fn new(reader: T) -> InputStream<T> {
        InputStream { reader : reader, byte_buffer: Vec::new() }
    }

    pub fn scan<F>(&mut self) -> Result<F> where
        F: FromStr,
        <F as FromStr>::Err: std::error::Error + Send + 'static
    {
        scan(&mut self.reader, &mut self.byte_buffer)
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
    use test::Bencher;
    use std::fs::File;
    use std::io::{BufReader, BufRead};

    #[test]
    fn simple_strings() {
        let text = "Howdy neighbour, how are you doing?";
        let mut stream = InputStream::new(text.as_bytes());

        let first: String = stream.scan().unwrap();
        let second: String = stream.scan().unwrap();
        let third: String = stream.scan().unwrap();
        assert_eq!(first, "Howdy");
        assert_eq!(second, "neighbour,");
        assert_eq!(third, "how");
    }

    #[test]
    fn simple_numbers() {
        let text = "5 -7 12.5 -2.85";
        let mut stream = InputStream::new(text.as_bytes());
        assert_eq!(5, stream.scan().unwrap());
        assert_eq!(-7, stream.scan().unwrap());
        assert_eq!(12.5, stream.scan().unwrap());
        assert_eq!(-2.85, stream.scan().unwrap());
    }

    #[test]
    fn newlines() {
        let mut stream = InputStream::new("12\nHello".as_bytes());
        assert_eq!(12, stream.scan().unwrap());
        assert_eq!("Hello", stream.scan::<String>().unwrap());
    }

    #[bench]
    fn numbers_bench(b: &mut Bencher) {
        b.iter(|| {
            let file = File::open("./fixtures/numbers").unwrap();
            let mut stream = InputStream::new(BufReader::new(file));

            let mut count = 0;
            let mut sum = 0;
            while let Ok(number) = stream.scan() {
                count += 1;
                sum ^= number;
            }

            assert_eq!(count, 250000);
            assert_eq!(sum, 1275235796);
        })
    }

    #[bench]
    fn default_bench(b: &mut Bencher) {
        b.iter(|| {
            let file = File::open("./fixtures/numbers").unwrap();
            let mut count = 0;
            let mut sum = 0;
            for line in BufReader::new(file).lines() {
                for number in line.unwrap().split_whitespace() {
                    let number = number.parse::<i32>().unwrap();
                    count += 1;
                    sum ^= number;
                }
            }
            assert_eq!(count, 250000);
            assert_eq!(sum, 1275235796);
        })
    }
}
