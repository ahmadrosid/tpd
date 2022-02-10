use clap::{IntoApp, Parser, Subcommand};
use std::cmp::min;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::{Read, Write};

/// Typo detector for english words
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// File path to read
    file_path: Option<String>,

    /// Suggest fix for typo
    #[clap(long, short)]
    fix: bool,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add custom word to dictionary
    Add {
        /// Add english word
        #[clap(default_value_t = String::from(""))]
        word: String,

        /// Display my dictionary
        #[clap(long, short)]
        dictionary: bool,
    },
}

trait CharExt {
    fn accept(self) -> bool;
}

impl CharExt for char {
    fn accept(self) -> bool {
        match self {
            'a'..='z' | 'A'..='Z' | '\'' => true,
            _ => false,
        }
    }
}

fn main() {
    let args = Args::parse();
    match &args.command {
        Some(Commands::Add { word, dictionary }) => {
            match home::home_dir() {
                Some(mut path) => {
                    path.push(".config");
                    path.push("tpd");
                    path.push("dictionary");

                    let mut file = std::fs::OpenOptions::new()
                        .write(true)
                        .read(true)
                        .append(true)
                        .open(path)
                        .unwrap();

                    if *dictionary {
                        let mut data = String::new();
                        if let Ok(_) = file.read_to_string(&mut data) {
                            println!("{}", data);
                        } else {
                            println!("Unable to read dictionary");
                        }
                    } else {
                        if word.len() >= 1 {
                            write!(file, "{}\n", word).unwrap();
                            println!("Success store '{}' to dictionary!", word);
                        } else {
                            let mut app = Args::into_app();
                            app.print_help();
                        }
                    }
                }
                None => {}
            }

            std::process::exit(0);
        }
        None => {}
    }

    match args.file_path.as_deref() {
        Some(path) => detect_typo(path.to_string(), args.fix),
        _ => {
            let mut app = Args::into_app();
            app.print_help();
        }
    }
}

pub fn detect_typo(file_path: String, show_fix: bool) {
    match std::fs::read_to_string(file_path.clone()) {
        Ok(source) => {
            let val = include_str!("words_sort.txt").to_string();
            let mut words: Vec<&str> = val.split("\n").collect();

            match home::home_dir() {
                Some(mut path) => {
                    path.push(".config");
                    path.push("tpd");
                    path.push("dictionary");
                    match std::fs::read_to_string(path) {
                        Ok(file) => {
                            let dictionary = file.clone();
                            let dictionaries = dictionary.split("\n").collect::<Vec<_>>();
                            words.append(&mut dictionaries.to_vec());
                            words.sort();
                            scan_words(&*source, &words, show_fix, &file_path.clone());
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
                None => {}
            }

            scan_words(&*source, &words, show_fix, &file_path);
        }
        Err(_) => {
            println!("File not found!");
            std::process::exit(1);
        }
    }
}

pub fn scan_words(source: &str, words: &Vec<&str>, show_fix: bool, file_path: &str) {
    let mut line_number = 0;
    for line in source.lines().into_iter() {
        line_number += 1;
        let mut column = 0;
        for child in line.split(" ").into_iter() {
            let target = child.to_lowercase();
            column += child.len() + 1;
            if !target.chars().all(char::accept) {
                continue;
            }

            match words.binary_search(&&*target) {
                Ok(_) => {}
                Err(_) => {
                    if !show_fix {
                        println!("\"{}\" => {}:{}:{}", child, file_path, line_number, column);
                        continue;
                    }
    
                    match search_similar(&words.to_vec(), &target, 1) {
                        Some(result) => {
                            println!(
                                "\"{}\" => {}:{}:{} {}",
                                child, file_path, line_number, column, result
                            )
                        }
                        _ => match search_similar(&words.to_vec(), &target, 2) {
                            Some(result) => {
                                println!(
                                    "\"{}\" => {}:{}:{} {}",
                                    child, file_path, line_number, column, result
                                )
                            }
                            _ => println!("\"{}\" => {}:{}:{}", child, file_path, line_number, column),
                        },
                    }
                },
            }
        }
    }
}

pub fn search_similar(words: &Vec<&str>, target: &str, score: usize) -> Option<String> {
    let mut result: String = String::new();
    let distance = if score == 2 { 0 } else { 1 };
    words
        .to_vec()
        .iter()
        .filter(|item| {
            item.len() >= target.len() - distance && item.len() <= target.len() + distance
        })
        .for_each(|item| {
            if damerau_levenshtein(*item, &target) == score {
                result.push_str(*item);
                result.push(',')
            }
        });

    return if result.len() > 0 {
        Some(result.trim_end_matches(",").to_string())
    } else {
        None
    };
}

pub fn generic_damerau_levenshtein<Elem>(a_elems: &[Elem], b_elems: &[Elem]) -> usize
where
    Elem: Eq + Hash + Clone,
{
    let a_len = a_elems.len();
    let b_len = b_elems.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let width = a_len + 2;
    let mut distances = vec![0; (a_len + 2) * (b_len + 2)];
    let max_distance = a_len + b_len;
    distances[0] = max_distance;

    for i in 0..(a_len + 1) {
        distances[flat_index(i + 1, 0, width)] = max_distance;
        distances[flat_index(i + 1, 1, width)] = i;
    }

    for j in 0..(b_len + 1) {
        distances[flat_index(0, j + 1, width)] = max_distance;
        distances[flat_index(1, j + 1, width)] = j;
    }

    let mut elems: HashMap<Elem, usize> = HashMap::with_capacity(64);

    for i in 1..(a_len + 1) {
        let mut db = 0;

        for j in 1..(b_len + 1) {
            let k = match elems.get(&b_elems[j - 1]) {
                Some(&value) => value,
                None => 0,
            };

            let insertion_cost = distances[flat_index(i, j + 1, width)] + 1;
            let deletion_cost = distances[flat_index(i + 1, j, width)] + 1;
            let transposition_cost =
                distances[flat_index(k, db, width)] + (i - k - 1) + 1 + (j - db - 1);

            let mut substitution_cost = distances[flat_index(i, j, width)] + 1;
            if a_elems[i - 1] == b_elems[j - 1] {
                db = j;
                substitution_cost -= 1;
            }

            distances[flat_index(i + 1, j + 1, width)] = min(
                substitution_cost,
                min(insertion_cost, min(deletion_cost, transposition_cost)),
            );
        }

        elems.insert(a_elems[i - 1].clone(), i);
    }

    distances[flat_index(a_len + 1, b_len + 1, width)]
}

fn flat_index(i: usize, j: usize, width: usize) -> usize {
    j * width + i
}

pub fn damerau_levenshtein(a: &str, b: &str) -> usize {
    let (x, y): (Vec<_>, Vec<_>) = (a.chars().collect(), b.chars().collect());
    generic_damerau_levenshtein(x.as_slice(), y.as_slice())
}
