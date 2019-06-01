#![feature(test)]
extern crate test;

use input_stream::InputStream;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::str::FromStr;
use test::Bencher;

const NUMBERS_GENERATED: usize = 100_000;

fn generate_numbers<T: ToString>(many: usize) -> String
where
    Standard: Distribution<T>,
{
    let mut rng = rand::thread_rng();
    (0..many)
        .map(|_| rng.gen::<T>().to_string())
        .fold("".to_string(), |mut str, number| {
            str += " ";
            str += &number;
            str
        })
}

fn count_numbers<T>(input: &str) -> usize
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    let mut stream = InputStream::new(input.as_bytes());

    let mut count = 0;
    while let Ok(_) = stream.scan::<T>() {
        count += 1;
    }
    count
}

macro_rules! num_bench {
    ($(($num: ty, $bench: ident)),* ) => {
        $(
            #[bench]
            fn $bench(b: &mut Bencher) {
                let numbers = generate_numbers::<$num>(NUMBERS_GENERATED);

                b.iter(|| {
                    let count = count_numbers::<$num>(&numbers);
                    assert_eq!(count, NUMBERS_GENERATED);
                });
            }
         )*
    }
}

num_bench! {
    (u8, u8_bench),
    (u16, u16_bench),
    (u32, u32_bench),
    (u64, u64_bench),
    (usize, usize_bench),
    (i8, i8_bench),
    (i16, i16_bench),
    (i32, i32_bench),
    (i64, i64_bench),
    (isize, isize_bench),
    (f32, f32_bench),
    (f64, f64_bench)
}
