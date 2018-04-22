use super::{Dict, ServerError};

use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufReader, prelude::*},
};

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
            .map(|l| l.expect(&format!("Error while reading dictionary: {}", file)))
            .collect()
    }
}

impl Dict for LocalDict {
    fn contains(&self, word: &str) -> Result<(), ServerError> {
        if self.words.contains(word) {
            Ok(())
        } else {
            Err(ServerError::non_existing_word(word.to_string()))
        }
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

    #[test]
    fn contains() {
        let dict = LocalDict::new();
        match dict.contains("bombance") {
            Ok(()) => (),
            _ => panic!("This call should return Ok(())")
        }
    }

    #[test]
    fn doesnt_contain() {
        let dict = LocalDict::new();
        match dict.contains("preposterous") {
            Err(ServerError::NonExistingWord {..}) => (),
            _ => panic!("This call should return ServerError::NonExistingWord")
        }
    }
}