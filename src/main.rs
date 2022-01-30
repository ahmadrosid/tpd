use clap::{IntoApp, Parser, Subcommand};
use std::io::{Read, Write};

/// Typo detector for english words
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// File path to read
    file_path: Option<String>,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Store custom word to dictionary
    Words {
        /// English word
        #[clap(short, default_value_t = String::from(""))]
        word: String,

        /// Display my dictionary
        #[clap(short)]
        d: bool,
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
        Some(Commands::Words { word, d }) => {
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

                    if *d {
                        let mut data = String::new();
                        if let Ok(_) = file.read_to_string(&mut data) {
                            println!("{}", data);
                        } else {
                            println!("Unable to read dictionary");
                        }
                    } else {
                        if word.len() >= 1 {
                            write!(file, "{}\n", word);
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
        Some(path) => detect_typo(path.to_string()),
        _ => {
            let mut app = Args::into_app();
            app.print_help();
        }
    }
}

pub fn detect_typo(file_path: String) {
    match std::fs::read_to_string(file_path) {
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
                            scan_words(&*source, &words);
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
                None => {}
            }

            scan_words(&*source, &words);
        }
        Err(_) => {
            println!("File not found!");
            std::process::exit(1);
        }
    }
}

pub fn scan_words(source: &str, words: &Vec<&str>) {
    let mut line_number = 0;
    let file = std::env::args().collect::<Vec<_>>();
    for line in source.lines().into_iter() {
        line_number += 1;
        let mut column = 0;
        for child in line.split(" ").into_iter() {
            let target = child.to_lowercase();
            column += child.len() +  1;
            if !target.chars().all(char::accept) {
                continue;
            }
            if search(&words.to_vec(), &target, words.len()).is_none() {
                println!("\"{}\" => {}:{}:{}", child, file[1], line_number, column);
            }
        }
    }
}

pub fn search(a: &Vec<&str>, target_value: &str, len: usize) -> Option<usize> {
    let mut low: usize = 0;
    let mut high: usize = len - 1;

    while low <= high {
        let mid = ((high - low) / 2) + low;
        let mid_index = mid as usize;
        let val = a[mid_index];

        if val == target_value {
            return Some(mid_index);
        }

        if val < target_value {
            low = mid + 1;
        }

        if val > target_value {
            high = mid - 1;
        }
    }
    None
}
