use quicli::prelude::*;
use structopt::StructOpt;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::str::FromStr;

#[derive(Debug, StructOpt)]
struct Cli {
    file: String,
}

#[derive(Debug)]
struct Library {
    signup: usize,
    concurrency: usize,
    books: Vec<usize>,
}

fn main() {
    let Cli { file } = Cli::from_args();
    let mut lines = BufReader::new(File::open(file)
        .expect("Could not open file"))
        .lines()
        .map(|s| s.expect("Could not read string"));
    let (b, l, d) = {
        let items = lines.next()
            .expect("what?")
            .split(" ")
            .map(|s| usize::from_str_radix(s, 10).expect("Could not parse usize"))
            .collect::<Vec<_>>();
        (items[0], items[1], items[2])
    };
    let scores = lines.next().expect("what?")
        .split(" ")
        .take(b)
        .map(|s| usize::from_str_radix(s, 10).expect("Could not parse usize"))
        .collect::<Vec<_>>();
    let lines = lines.collect::<Vec<_>>();
    let libraries = lines.as_slice()
        .chunks(2)
        .take(l)
        .map(|s| {
            let (books_amount, signup, concurrency) = {
                let nums = s[0].split(" ").map(|s| usize::from_str_radix(s, 10).expect("Could not parse number")).collect::<Vec<_>>();
                (nums[0], nums[1], nums[2])
            };
            let books = s[1].split(" ").take(books_amount).map(usize::from_str).map(|s| s.unwrap()).collect::<Vec<_>>();
            Library {
                signup,
                concurrency,
                books
            }
        })
        .collect::<Vec<_>>();
    println!("{}, {}, {}", b, l, d);
    println!("{:?}", libraries);
    println!("{:?}", scores);
}
