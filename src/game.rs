use super::{
    errors::{ServerError, ServerError::*},
    dict::{Dict, LocalDict},
};

use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
};

use rand::{self, Rng};

const DICES: [[char; 6]; 16] = [
    ['E', 'T', 'U', 'K', 'N', 'O'],
    ['E', 'V', 'G', 'T', 'I', 'N'],
    ['D', 'E', 'C', 'A', 'M', 'P'],
    ['I', 'E', 'L', 'R', 'U', 'W'],
    ['E', 'H', 'I', 'F', 'S', 'E'],
    ['R', 'E', 'C', 'A', 'L', 'S'],
    ['E', 'N', 'T', 'D', 'O', 'S'],
    ['O', 'F', 'X', 'R', 'I', 'A'],
    ['N', 'A', 'V', 'E', 'D', 'Z'],
    ['E', 'I', 'O', 'A', 'T', 'A'],
    ['G', 'L', 'E', 'N', 'Y', 'U'],
    ['B', 'M', 'A', 'Q', 'J', 'O'],
    ['T', 'L', 'I', 'B', 'R', 'A'],
    ['S', 'P', 'U', 'L', 'T', 'E'],
    ['A', 'I', 'M', 'S', 'O', 'R'],
    ['E', 'N', 'H', 'R', 'I', 'S'],
];

pub struct Game {
    grid: [char; 16],
    player_words: HashMap<String, Vec<String>>,
    played: HashSet<String>,
    turn: u64,
    dict: Box<Dict>,
}

impl Game {
    pub fn new() -> Game {
        Game::with_dict(Box::new(LocalDict::new()))
    }

    pub fn with_dict(dict: Box<Dict>) -> Game {
        Game {
            grid: Game::generate_grid(),
            player_words: HashMap::new(),
            played: HashSet::new(),
            turn: 0,
            dict
        }
    }

    fn generate_grid() -> [char; 16] {
        let mut rng = rand::thread_rng();
        let mut result = ['A'; 16];

        DICES.iter()
            .map(|dice| rng.choose(dice).unwrap())
            .enumerate()
            .for_each(|(idx, &letter)| result[idx] = letter);

        result
    }

    pub fn grid_str(&self) -> String {
        String::from_iter(self.grid.iter())
    }

    pub fn submit_trajectory(&mut self) {
        // TODO: implement
        unimplemented!()
    }

    pub fn check_trajectory(trajectory: &str) -> Result<(), ServerError> {
        // TODO: implement
        unimplemented!()
    }

    pub fn new_turn(&mut self) {
        self.grid = Game::generate_grid();
        self.turn += 1;
    }

    pub fn update_users(&mut self, users: HashSet<&str>) {
        self.player_words.retain(|key, _| users.contains(&key.as_str()));
        self.played = self.compute_played()
    }

    fn compute_played(&self) -> HashSet<String> {
        HashSet::from_iter(
            self.player_words.values()
                .flat_map(|v| v)
                .map(|v| v.to_string())
        )
    }

    pub fn users_scores(&self, users: &[String]) -> Vec<(String, u32)> {
        users.iter()
            .map(|u| (u.to_owned(), self.user_score(u)))
            .collect()
    }

    fn user_score(&self, user: &str) -> u32 {
        self.player_words.get(user).map_or(0, |words| {
            words.iter()
                .map(|w| Game::word_score(w))
                .sum()
        })
    }

    fn word_score(word: &str) -> u32 {
        match word.len() {
            0...2 => 0,
            3...4 => 1,
            5 => 2,
            6 => 3,
            7 => 5,
            _ => 11
        }
    }

    fn letter_at(&self, line: char, column: usize) -> Result<char, ServerError> {
        let idx = Game::index_of_coordinates(line, column)?;
        Ok(self.grid[idx])
    }

    fn index_of_coordinates(line: char, column: usize) -> Result<usize, ServerError> {
        if line < 'A' || line > 'D' || column < 1 || column > 4 {
            Err(InvalidCoordinates { line, column })
        } else {
            Ok((column - 1) + (4 * Game::index_of_letter(line)))
        }
    }

    fn index_of_letter(letter: char) -> usize {
        match letter {
            'a' | 'A' => 0,
            'b' | 'B' => 1,
            'c' | 'C' => 2,
            'd' | 'D' => 3,
            _ => panic!("Invalid character index !")
        }
    }
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        let game = Game::new();
        game.grid.iter().enumerate()
            .for_each(|(idx, c)| assert!(DICES[idx].contains(c)));
    }

    #[test]
    fn index_of_coordinates() {
        assert_eq!(Game::index_of_coordinates('C', 2).unwrap(), 9);
        assert_eq!(Game::index_of_coordinates('A', 1).unwrap(), 0);
        assert_eq!(Game::index_of_coordinates('D', 4).unwrap(), 15);
    }

    #[test]
    fn invalid_coordinates_returns_error() {
        match Game::index_of_coordinates('E', 1) {
            Err(InvalidCoordinates {..}) => (),
            _ => panic!("This call sould return an error !")
        }

        match Game::index_of_coordinates('E', 0) {
            Err(InvalidCoordinates {..}) => (),
            _ => panic!("This call sould return an error !")
        }

        match Game::index_of_coordinates('E', 5) {
            Err(InvalidCoordinates {..}) => (),
            _ => panic!("This call sould return an error !")
        }
    }

    #[test]
    fn update_users() {
        let mut game = Game::new();
        game.player_words.insert("user1".to_string(), vec!["word1".to_string()]);
        game.player_words.insert("user2".to_string(), vec!["word2".to_string()]);
        game.player_words.insert("user3".to_string(), vec!["word3".to_string()]);

        let users = vec!["user1", "user3"];
        game.update_users(HashSet::from_iter(users));

        let expected: HashSet<&str> = HashSet::from_iter(vec!["user1", "user3"]);
        let actual: HashSet<&str> = game.player_words.keys()
            .map(|p| p.as_str())
            .collect();

        assert_eq!(expected, actual);
    }
}

