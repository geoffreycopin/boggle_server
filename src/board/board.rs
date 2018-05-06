use super::*;

use rand::{self, Rng};

use std::iter::FromIterator;

pub struct Board {
    grid: [char; 16],
    scores: HashMap<String, u32>,
    player_words: HashMap<String, Vec<String>>,
    played: HashSet<String>,
    invalid_words: HashSet<String>,
    immediate_check: bool,
    turn: u64
}

impl Board {
    pub fn new(immediate_check: bool) -> Board {
        Board {
            grid: Board::generate_grid(),
            scores: HashMap::new(),
            player_words: HashMap::new(),
            played: HashSet::new(),
            invalid_words: HashSet::new(),
            immediate_check,
            turn: 0,
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

    pub fn reset(&mut self) {
        self.grid = Board::generate_grid();
        self.invalid_words.clear();
        self.scores.clear();
        self.player_words.clear();
        self.played.clear();
        self.turn = 1;
    }

    pub fn welcome_str(&self) -> String {
        let grid = String::from_iter(self.grid.iter());
        let scores = self.scores_str();
        format!("BIENVENUE/{}/{}*{}/\n", grid, self.turn, scores)
    }

    /// Renvoie une chaîne de charactères représentant le cube de boggle.
    pub fn grid_str(&self) -> String {
        String::from_iter(self.grid.iter())
    }

    /// Ajoute l'utilisateur `username` et lui aattribue un score de 0.
    /// Si l'utlisateur existait déjà, son score est écrasé.
    pub fn add_user(&mut self, username: &str) {
        self.scores.insert(username.to_string(), 0);
    }

    /// Retire l'utilisateur `username` de la liste des utilisateurs s'il existe.
    pub fn remove_user(&mut self, username: &str) {
        self.scores.remove(username);
    }

    pub fn scores_str(&self) -> String  {
        self.scores.keys()
            .map(|u| format!("{}*{}", u, self.user_score(u)))
            .collect::<Vec<String>>()
            .join("*")
    }

    fn words_str(&self) -> String {
        self.scores.keys()
            .map(|u| {
                match self.player_words.get(u) {
                    Some(words) => format!("{}*{}", u, words.join("*")),
                    None => u.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("*")
    }

    pub fn submit_word(&mut self, user: &str, word: &str, trajectory: &str)
        -> Result<(), ServerError>
    {
        let t = trajectory_of_string(trajectory)?;

        if self.word_of_trajectory(&t) != word {
            return Err(ServerError::no_match(trajectory, word))
        }

        if ! self.scores.contains_key(user) {
            return Err(ServerError::non_existing_user(user))
        }

        if self.played.contains(word) {
            if self.immediate_check {
                return Err(ServerError::already_played(word))
            } else {
                self.invalid_words.insert(word.to_string());
                return Ok(())
            }
        }

        self.player_words.entry(user.to_string()).or_insert(vec![])
            .push(word.to_string());
        self.played.insert(word.to_string());

        Ok(())
    }

    fn word_of_trajectory(&self, trajectory: &[(char, usize)]) -> String {
        trajectory.iter() .map(|&(line, col)| {
                let idx = (4 * index_of_letter(line)) + (col - 1);
                self.grid[idx]
            })
            .collect::<String>().to_lowercase()
    }

    pub fn new_turn(&mut self) {
        self.update_users_scores();
        self.grid = Board::generate_grid();
        self.player_words.clear();
        self.played.clear();
        self.invalid_words.clear();
        self.turn += 1;
    }

    pub fn turn_scores(&mut self) -> String {
        format!("BILANMOTS/{}/{}/\n", self.words_str(), self.scores_str())
    }

    fn update_users_scores(&mut self) {
        let mut scores = HashMap::new();
        for (user, score) in self.scores.iter() {
            let s = score + self.turn_score(user);
            scores.insert(user.to_string(), s);
        }
        self.scores = scores;
    }

    fn user_score(&self, user: &str) -> u32 {
        match self.scores.get(user) {
            Some(score) => score + self.turn_score(user),
            None => self.turn_score(user)
        }
    }

    fn turn_score(&self, user: &str) -> u32 {
        self.player_words.get(user).map_or(0, |words| {
            words.iter()
                .filter(|&w| ! self.invalid_words.contains(w))
                .map(|w| word_score(w))
                .sum()
        })
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn new() {
        let board = Board::new(true);
        board.grid.iter().enumerate()
            .for_each(|(idx, c)| assert!(DICES[idx].contains(c)));
    }

    #[test]
    fn word_of_trajectory() {
        let board = create_test_board();
        let trajectory = vec![('C', 2), ('B', 1), ('A', 2), ('A', 3), ('B', 2), ('C', 3), ('D', 2)];
        assert_eq!(board.word_of_trajectory(&trajectory), "trident");
    }

    #[test]
    fn submit_word() {
        let mut board = create_test_board();
        board.add_user("user1");
        board.immediate_check = false;
        board.submit_word("user1", "trident", "C2B1A2A3B2C3D2").unwrap();
        assert_eq!(board.user_score("user1"), 5);
    }

    #[test]
    fn submit_already_played_word() {
        let mut board = create_test_board();
        board.immediate_check = false;
        board.add_user("user1");
        board.add_user("user2");
        board.submit_word("user1", "trident", "C2B1A2A3B2C3D2").unwrap();
        board.submit_word("user1", "ile", "A2A1B2").unwrap();
        board.submit_word("user2", "trident", "C2B1A2A3B2C3D2").unwrap();
        assert_eq!(board.user_score("user1"), 1);
        assert_eq!(board.user_score("user2"), 0);
    }

    #[test]
    fn submit_word_im() {
        let mut board = create_test_board();
        board.add_user("user1");
        board.submit_word("user1", "trident", "C2B1A2A3B2C3D2").unwrap();
        let expected = vec!["trident".to_string()];
        assert_eq!(board.player_words.get("user1").unwrap(), &expected);
    }

    #[test]
    fn submit_word_im_adds_word_to_played() {
        let mut board = create_test_board();
        board.add_user("user1");
        board.submit_word("user1", "trident", "C2B1A2A3B2C3D2").unwrap();
        assert!(board.played.contains("trident"));
    }

    #[test]
    fn submit_words_im_error_no_match() {
        let mut board = create_test_board();
        match board.submit_word("user1", "ile", "C2B1A2A3B2C3D2") {
            Err(ServerError::NoMatch {..}) => (),
            _ => panic!("This call should return an error !")
        }
    }

    #[test]
    fn welcome_str() {
        let mut board = create_test_board();
        board.add_user("user1");
        board.submit_word("user1", "trident", "C2B1A2A3B2C3D2").unwrap();
        assert_eq!(board.welcome_str(), "BIENVENUE/LIDAREJULTNEATNG/1*user1*5/\n")
    }

    #[test]
    fn new_turn() {
        let mut board = create_test_board();
        let old_grid = board.grid;
        let old_turn = board.turn;

        board.new_turn();

        assert_eq!(board.turn, old_turn + 1);
        assert_eq!(board.player_words, HashMap::new());
        assert_eq!(board.played, HashSet::new());
        assert_eq!(board.invalid_words, HashSet::new());
        assert_ne!(board.grid, old_grid);
    }

    #[test]
    fn scores_updated_after_new_turn() {
        let mut board = create_test_board();
        board.add_user("user1");
        board.submit_word("user1", "trident", "C2B1A2A3B2C3D2").unwrap();
        assert_eq!(board.scores.get("user1").unwrap(), &(0 as u32));
        board.new_turn();
        assert_eq!(board.scores.get("user1").unwrap(), &(5 as u32));
    }

    #[test]
    fn reset() {
        let mut board = create_test_board();
        let old_grid = board.grid;

        board.reset();

        assert_eq!(board.turn, 1);
        assert_eq!(board.player_words, HashMap::new());
        assert_eq!(board.played, HashSet::new());
        assert_ne!(board.grid, old_grid);
    }

    pub fn create_test_board() -> Board {
        let mut board = Board::new(true);
        board.grid = ['L', 'I', 'D', 'A',
                     'R', 'E', 'J', 'U',
                     'L', 'T', 'N', 'E',
                     'A', 'T', 'N', 'G' ];
        board.turn = 1;
        board
    }
}
