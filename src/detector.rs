use crate::dictionary::Dictionary;

use std::cmp::min;
use std::collections::HashMap;
use std::path::PathBuf;

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

pub fn detect_typo(file_path: PathBuf, show_suggest: bool) {
    match std::fs::read_to_string(&file_path) {
        Ok(source) => {
            let mut words = Dictionary::new();

            match home::home_dir() {
                Some(mut path) => {
                    path.push(".config");
                    path.push("tpd");
                    path.push("dictionary");
                    match std::fs::read_to_string(path) {
                        Ok(file) => {
                            for word in file.split('\n') {
                                words.add(word.to_string());
                            }
                            scan_words(&*source, &words, show_suggest, &file_path.display().to_string());
                            return;
                        }
                        _ => {}
                    }
                }
                None => {}
            }

            scan_words(&*source, &words, show_suggest, &file_path.display().to_string());
        }
        Err(_) => {
            println!("File not found!");
            std::process::exit(1);
        }
    }
}

pub fn scan_words(source: &str, words: &Dictionary, show_suggest: bool, file_path: &str) {
    let mut line_number = 0;
    for line in source.lines().into_iter() {
        line_number += 1;
        let mut column = 0;
        for child in line.split(" ").into_iter() {
            let mut target = child.to_lowercase();
            column += child.len() + 1;
            target = sanitize(target);
            if !target.chars().all(char::accept) || target == "" {
                continue;
            }

            if words.get(&&*target).is_none() {
                if !show_suggest {
                    println!("\x1b[0;31m{}\x1b[m => {}:{}:{}", child, file_path, line_number, column);
                    continue;
                }

                match search_similar(&words.to_vec(), &target, 1) {
                    Some(result) => {
                        println!(
                            "\x1b[0;31m{}\x1b[m => {}:{}:{} \x1b[0;32m{}\x1b",
                            child, file_path, line_number, column, result
                        )
                    }
                    _ => match search_similar(&words.to_vec(), &target, 2) {
                        Some(result) => {
                            println!(
                                "\x1b[0;31m{}\x1b[m => {}:{}:{} \x1b[0;32m{}\x1b",
                                child, file_path, line_number, column, result
                            )
                        }
                        _ => println!(
                            "\x1b[0;31m{}\x1b[m => {}:{}:{}",
                            child, file_path, line_number, column
                        ),
                    },
                }
            }
        }
    }
}

pub fn sanitize(word: String) -> String {
    if word.len() <= 1 {
        return word;
    }

    return word
        .replace("\"", "")
        .replace("'", "")
        .replace("!", "")
        .replace("?", "")
        .replace(".", "")
        .replace("“", "")
        .replace("”", "")
        .replace(",", "");
}

pub fn search_similar(words: &Vec<&String>, target: &str, score: usize) -> Option<String> {
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

pub fn damerau_levenshtein(word: &str, target: &str) -> usize
{
    let (word, target): (Vec<char>, Vec<char>) = (word.chars().collect(), target.chars().collect());
    let a_elems = word.as_slice();
    let b_elems = target.as_slice();
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

    let mut elems: HashMap<char, usize> = HashMap::with_capacity(64);

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
