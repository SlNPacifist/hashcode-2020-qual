use structopt::StructOpt;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::str::FromStr;
use std::collections::{HashSet, HashMap};
use ordered_float::OrderedFloat;

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

#[derive(Debug)]
struct LibraryScanOrder {
    id: usize,
    books: Vec<usize>,
}

#[derive(Debug)]
struct Solution {
    libs: Vec<LibraryScanOrder>,
}

struct Problem {
    scores: Vec<usize>,
    libraries: Vec<Library>,
    days: usize,
}

fn solve_b(scores: &Vec<usize>, libraries: &Vec<Library>, _days: usize) -> Solution {
    let hs = &scores.iter().collect::<HashSet<_>>();
    assert_eq!(hs.len(), 1);
    assert!(&libraries.iter().all(|l| l.books.len() == 1000));
    assert!(&libraries.iter().all(|l| l.concurrency == 1));
    let acc = &libraries.iter().flat_map(|l| &l.books).fold(HashMap::new(), |mut acc, book| {
        *acc.entry(book).or_insert(0) += 1;
        acc
    });
    assert_eq!(*acc.values().max().unwrap(), 1);

    let mut order = (0..libraries.len()).collect::<Vec<_>>();
    order.sort_by_key(|&i| libraries[i].signup);
    Solution {
        libs: order.iter().map(|&i| LibraryScanOrder {
            id: i,
            books: libraries[i].books.clone(),
        }).collect::<Vec<_>>()
    }
}

// TODO validate
fn calc_score(problem: Problem, solution: Solution) -> usize {
    solution.libs
        .iter()
        .scan(0usize, |mut signup_start_day, lib| {
            if *signup_start_day >= problem.days {
                None
            } else {
                let lib_input = &problem.libraries[lib.id];
                let lib_start_day = *signup_start_day;
                *signup_start_day += lib_input.signup;
                Some((lib_start_day, lib_input.concurrency, lib.books.clone()))
            }
        })
        .flat_map(|(start_day, concurrency, books)| {
            let days_left = problem.days - start_day;
            let total_books_scanned = days_left * concurrency;
            books[0..total_books_scanned].to_vec()
        })
        .fold((0usize, HashSet::<usize>::new()), |(mut score, mut used_books), book| {
            if ! used_books.contains(&book) {
                score += problem.scores[book];
                used_books.insert(book);
            }
            (score, used_books)
        })
        .0
}

fn solve_c(scores: &Vec<usize>, libraries: &Vec<Library>, days: usize) -> Solution {
    assert!(&libraries.iter().all(|l| l.books.len() <= l.concurrency));
    let mut used_books: HashSet<usize> = HashSet::new();
    let mut used_libraries: HashSet<usize> = HashSet::new();
    let mut days_left = days;

    let calc_score = |lib: &Library, used_books: &HashSet<usize>| lib.books.iter()
        .filter(|b| !used_books.contains(b))
        .map(|&b| scores[b])
        .sum::<usize>() as f64 / (lib.signup as f64 + 1.0f64);

    while let Some((lib_id, lib)) = libraries.iter().enumerate()
            .filter(|(i, lib)| !used_libraries.contains(i) && lib.signup + 1 <= days_left)
            .max_by_key(|(_, lib)| OrderedFloat(calc_score(lib, &used_books))) {

        used_libraries.insert(lib_id);
        for &book in &lib.books {
            used_books.insert(book);
        }
        days_left -= lib.signup;
        println!("Days left {}", days_left);
        println!("Score added {}", calc_score(lib, &used_books));
    }

    println!("Used {} books", used_books.len());
    Solution {
        libs: used_libraries.iter().map(|&id| LibraryScanOrder {
            id,
            books: libraries[id].books.clone(),
        }).collect()
    }
}

fn main() {
    let Cli { file } = Cli::from_args();
    let mut lines = BufReader::new(File::open(&file)
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

    let solution = if file.starts_with("data/b") {
        solve_b(&scores, &libraries, d)
    } else if file.starts_with("data/c") {
        solve_c(&scores, &libraries, d)
    } else {
        panic!("How to solve this?");
    };

    println!("{}", solution.libs.len());
    for lib in solution.libs {
        println!("{} {}", lib.id, lib.books.len());
        println!("{}", lib.books.iter().map(usize::to_string).collect::<Vec<_>>().join(" "));
    }
}
