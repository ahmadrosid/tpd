
trait CharExt {
    fn accept(self) -> bool;
}

impl CharExt for char {
    fn accept(self) -> bool {
        match self {
            'a'..='z' | 'A'..='Z' | '\'' => true,
            _ => false
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

fn main() {
    let mut arg = std::env::args();
    if arg.len() == 1 {
        println!("Please provide file path!");
        std::process::exit(0x1)
    }
    arg.next();
    match std::fs::read_to_string(arg.next().unwrap()) {
        Ok(source) => {
            let val = include_str!("words_sort.txt").to_string();
            let words: Vec<&str> = val.split("\n").collect();
            let mut index = 0;
            let file = std::env::args().collect::<Vec<_>>();
            for line in source.lines().into_iter() {
                index += 1;
                for child in line.split(" ").into_iter() {
                    let target = child.to_lowercase();
                    if !target.chars().all(char::accept) {
                        continue;
                    }
                    if search(&words, &target, words.len()).is_none() {
                        println!("\"{}\" => {}:{}", child, file[1], index);
                    }
                }
            }
        }
        Err(_) => {
            println!("File not found!");
            std::process::exit(1);
        }
    }
}
