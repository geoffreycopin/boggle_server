use super::{Dict, ServerError};

use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufReader, prelude::*},
};

use unidecode::unidecode;

pub struct LocalDict {
    words: HashSet<String>
}

impl LocalDict {
    pub fn new() -> LocalDict {
        LocalDict::from_dictionary("dico_fr.txt")
    }

    pub fn from_dictionary(file: &str) -> LocalDict {
        let words = LocalDict::load_dictionary(file);
        LocalDict { words }
    }

    fn load_dictionary(file: &str) -> HashSet<String> {
        let f = File::open(file).expect(&format!("Cannot open file: {}", file));
        let reader = BufReader::new(f);
        reader.lines()
            .map(|l| unidecode(&l.expect(&format!("Error while reading dictionary: {}", file))))
            .collect()
    }
}

impl Dict for LocalDict {
    fn contains(&self, word: &str) -> bool {
        self.words.contains(&word.to_lowercase())
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn new_panics_on_invalid_dict_file() {
        let dict = LocalDict::from_dictionary("non_existing_dict.txt");
    }
}