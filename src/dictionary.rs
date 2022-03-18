use std::collections::HashSet;

pub struct Dictionary {
    data: HashSet<String>
}

impl Dictionary {
    pub fn new() -> Self {
        let words = include_str!("words.txt");
        let mut dictionary: HashSet<String> = HashSet::new();
        for word in words.split('\n') {
            dictionary.insert(word.to_string());
        }

        Self {
            data: dictionary
        }
    }

    pub fn add(&mut self, word: String) {
        self.data.insert(word);
    }

    pub fn to_vec(&self) -> Vec<&String> {
        self.data.iter().collect()
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}