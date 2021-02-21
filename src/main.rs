use structopt::StructOpt;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::str::FromStr;
use std::collections::{HashSet, HashMap};
use ordered_float::OrderedFloat;
use rayon::prelude::*;

#[derive(Debug, StructOpt)]
struct Cli {
    file: String,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
struct BookId(usize);

#[derive(Debug)]
struct Library {
    signup: usize,
    concurrency: usize,
    books: Vec<BookId>,
}

#[derive(Debug)]
struct LibraryScanOrder {
    id: usize,
    books: Vec<BookId>,
    books_left: Vec<BookId>,
    max_scanned_books: usize,
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

impl Solution {
    fn from_order(problem: &Problem, order: &Vec<usize>) -> Self {
        let Problem { libraries, days, .. } = problem;
        let mut days_left = *days;
        Solution {
            libs: order.iter().map(|&i| {
                let lib = &libraries[i];
                days_left -= lib.signup;
                LibraryScanOrder {
                    id: i,
                    books: lib.books.clone(),
                    books_left: vec!(),
                    max_scanned_books: lib.books.len().min(days_left * lib.concurrency)
                }
            }).collect::<Vec<_>>()
        }
    }
}

fn solve_b(problem: &Problem) -> Solution {
    let Problem { scores, libraries, .. } = problem;
    let hs = scores.iter().collect::<HashSet<_>>();
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
    Solution::from_order(problem, &order)
}

fn calc_score(problem: &Problem, solution: &Solution) -> usize {
    solution.libs
        .iter()
        .scan(0usize, |signup_start_day, lib| {
            if *signup_start_day >= problem.days {
                None
            } else {
                let lib_input = &problem.libraries[lib.id];
                *signup_start_day += lib_input.signup;
                Some((*signup_start_day, lib_input.concurrency, lib.books.clone(), &lib_input.books))
            }
        })
        .flat_map(|(start_day, concurrency, solution_books, input_books)| {
            assert!(solution_books.iter().all(|b| input_books.contains(b)));
            let days_left = problem.days.saturating_sub(start_day);
            let max_scanned_books = input_books.len().min(days_left * concurrency);
            let total_books_scanned = (max_scanned_books).min(solution_books.len());

            // println!("Slots left: {}", max_scanned_books - total_books_scanned);

            solution_books[0..total_books_scanned].to_vec()
        })
        .collect::<HashSet<_>>()
        .iter()
        .map(|&b| problem.scores[b.0])
        .sum()
}

fn solve_c(problem: &Problem) -> Solution {
    let Problem { scores, libraries, days } = problem;
    assert!(&libraries.iter().all(|l| l.books.len() <= l.concurrency));
    let mut used_books: HashSet<BookId> = HashSet::new();
    let mut used_libraries: HashSet<usize> = HashSet::new();
    let mut days_left = *days;
    let mut score_total = 0;

    let calc_score = |lib: &Library, used_books: &HashSet<BookId>| lib.books.iter()
        .filter(|b| !used_books.contains(b))
        .map(|&b| scores[b.0])
        .sum::<usize>();

    while let Some((lib_id, lib)) = libraries.iter().enumerate()
            .filter(|(i, lib)| !used_libraries.contains(i) && lib.signup + 1 <= days_left)
            .max_by_key(|(_, lib)| OrderedFloat(calc_score(lib, &used_books) as f64 / ((lib.signup + 1) as f64))) {

        println!("Days left {}", days_left);
        let score_added = calc_score(lib, &used_books);
        score_total += score_added;
        println!("Score total {}, added {}", score_total, score_added);

        used_libraries.insert(lib_id);
        for &book in &lib.books {
            used_books.insert(book);
        }
        days_left -= lib.signup;
    }

    println!("Used {} books", used_books.len());
    Solution::from_order(problem, &used_libraries.iter().copied().collect())
}

// TODO: do not break order of books
fn swap_book_usage(book: BookId, from: &mut Vec<BookId>, to: &mut Vec<BookId>) {
    from.remove(from.iter().position(|&b| b == book).expect("1"));
    to.push(book);
}

fn swap_book(solution: &mut Solution, book: BookId, from: usize, to: usize) {
    let lib_from = &mut solution.libs[from];
    swap_book_usage(book, &mut lib_from.books, &mut lib_from.books_left);

    let lib_to = &mut solution.libs[to];
    swap_book_usage(book, &mut lib_to.books_left, &mut lib_to.books);
}

fn optimize_solution(problem: &Problem, solution: &mut Solution) {
    let mut libs_by_book_used = HashMap::<BookId, usize>::new();
    let mut books_taken = HashSet::new();
    for (pos, lib) in solution.libs.iter().enumerate() {
        for &book in &lib.books {
            libs_by_book_used.insert(book, pos);
            books_taken.insert(book);
        }
    }

    while let Some((book_to_swap, current_lib_pos, lib_with_empty_slot_pos, book_to_take)) = solution
        .libs.iter().enumerate()
        // .filter(|(_, lib)| lib.max_scanned_books > lib.books.len())
        .flat_map(|(lib_with_empty_slot_pos, lib_with_empty_slot)| {
            let empty_slot_cost = if lib_with_empty_slot.max_scanned_books > lib_with_empty_slot.books.len() {
                0
            } else {
                problem.scores[lib_with_empty_slot.books.last().unwrap().0]
            };
            lib_with_empty_slot.books_left.iter()
                .filter_map(|&book_to_swap| {
                    libs_by_book_used.get(&book_to_swap)
                        .and_then(|&current_lib_pos| {
                            solution.libs[current_lib_pos].books_left.iter()
                                // books are ordered by their score, so take first one
                                .find(|book_to_take| !books_taken.contains(book_to_take))
                                .filter(|book_to_take| problem.scores[book_to_take.0] > empty_slot_cost)
                                .map(|&book_to_take| (book_to_swap, current_lib_pos, lib_with_empty_slot_pos, book_to_take))
                        })
                })
                .collect::<Vec<_>>()
        })
        .next() {

        let lib_with_empty_slot = &mut solution.libs[lib_with_empty_slot_pos];
        if lib_with_empty_slot.max_scanned_books == lib_with_empty_slot.books.len() {
            let last_book = *lib_with_empty_slot.books.last().unwrap();
            swap_book_usage(last_book, &mut lib_with_empty_slot.books, &mut lib_with_empty_slot.books_left);
            libs_by_book_used.remove(&last_book);
            books_taken.remove(&last_book);
        }

        swap_book(solution, book_to_swap, current_lib_pos, lib_with_empty_slot_pos);
        let current_lib = &mut solution.libs[current_lib_pos];
        libs_by_book_used.insert(book_to_swap, lib_with_empty_slot_pos);

        swap_book_usage(book_to_take, &mut current_lib.books_left, &mut current_lib.books);
        libs_by_book_used.insert(book_to_take, current_lib_pos);
        books_taken.insert(book_to_take);
        println!("Added {} score by using new book {:?}", problem.scores[book_to_take.0], book_to_take);
    }
}

// TODO buggy
fn solve_greedy(problem: &Problem) -> Solution {
    const use_norm: bool = true;
    // const use_norm: bool = false;

    let mut used_books: HashSet<BookId> = HashSet::new();
    let mut used_libraries: HashSet<usize> = HashSet::new();
    let mut solution_part: Vec<LibraryScanOrder> = vec![];
    let mut days_left = problem.days;
    let mut score_total = 0;

    let calc_for_lib = |lib: &Library, days_left: usize, used_books: &HashSet<BookId>| {
        let scanning_days_left = days_left.saturating_sub(lib.signup);
        let mut books_taken = 0;
        let books_max = lib.concurrency * scanning_days_left;

        lib.books.iter().copied()
            .partition::<Vec<_>, _>(|b| {
                if used_books.contains(b) || books_taken == books_max {
                    false
                } else {
                    books_taken += 1;
                    true
                }
            })
    };

    let calc_books_score = |books: &Vec<BookId>| {
        books
            .iter()
            .map(|&b| problem.scores[b.0])
            .sum::<usize>()
    };

    let calc_score = |lib: &Library, days_left: usize, used_books: &HashSet<BookId>| {
        calc_books_score(&calc_for_lib(lib, days_left, used_books).0)
    };

    while let Some((lib_id, lib)) = problem.libraries.par_iter().enumerate()
        .filter(|(i, lib)| !used_libraries.contains(i) && lib.signup + 1 <= days_left)
        .max_by_key(|(_, lib)| {
            if use_norm {
                OrderedFloat(calc_score(lib, days_left, &used_books) as f64 / (lib.signup as f64))
            } else {
                OrderedFloat(calc_score(lib, days_left, &used_books) as f64)
            }
        } ) {

        println!("Days left {}", days_left);
        let (books_for_lib, books_left) = calc_for_lib(lib, days_left, &used_books);
        let score_added = calc_books_score(&books_for_lib);
        score_total += score_added;
        println!("Score total {}, added {}", score_total, score_added);

        for &book in &books_for_lib {
            used_books.insert(book);
        }
        days_left -= lib.signup;

        used_libraries.insert(lib_id);
        solution_part.push(LibraryScanOrder {
            id: lib_id,
            books: books_for_lib,
            books_left,
            max_scanned_books: lib.books.len().min(days_left * lib.concurrency)
        });
    }

    println!("Used {} books", used_books.len());
    Solution {
        libs: solution_part,
    }
}

fn main() {
    let Cli { file } = Cli::from_args();
    let mut lines = BufReader::new(File::open(&file)
        .expect("Could not open file"))
        .lines()
        .map(|s| s.expect("Could not read string"));
    let (b, l, days) = {
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
            let mut books = s[1].split(" ")
                .take(books_amount)
                .map(usize::from_str)
                .map(|s| s.unwrap())
                .map(|b| BookId(b))
                .collect::<Vec<_>>();
            books.sort_by_key(|b| usize::max_value() - scores[b.0]);
            Library {
                signup,
                concurrency,
                books
            }
        })
        .collect::<Vec<_>>();

    let problem = Problem {
        scores,
        libraries,
        days,
    };

    let mut solution = if file.starts_with("data/b") {
        // solve_b(&problem)
        solve_greedy(&problem)
    } else if file.starts_with("data/c") {
        solve_c(&problem)
    } else {
        solve_greedy(&problem)
    };

    println!("Score {}", calc_score(&problem, &solution));

    optimize_solution(&problem, &mut solution);

    println!("Score {}", calc_score(&problem, &solution));
    // println!("{}", solution.libs.len());
    // for lib in solution.libs {
    //     println!("{} {}", lib.id, lib.books.len());
    //     println!("{}", lib.books.iter().map(usize::to_string).collect::<Vec<_>>().join(" "));
    // }
}
