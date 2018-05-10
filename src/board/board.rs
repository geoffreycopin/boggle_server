use super::*;

use std::iter::FromIterator;

pub struct Board {
    /// Grille de jeu
    grid: [char; 16],
    /// Grilles de test fournie au lancement du serveur
    grids: Vec<String>,
    /// HashMap associant chaque joueur à son score.
    scores: HashMap<String, u32>,
    /// HashMap associant chaque joueur aux mots qu'il a soumis.
    player_words: HashMap<String, Vec<String>>,
    /// Set contennant tous les mots joués lors de ce tours.
    played: HashSet<String>,
    /// Set contenant tous les mots joués au moins deux fois lors de ce tours.
    invalid_words: HashSet<String>,
    /// Si true, la verrification immédiate est activée.
    immediate: bool,
    /// Numéro du tours en cours.
    turn: u64
}

impl Board {
    pub fn new(immediate: bool, grids: Vec<String>) -> Board {
        Board {
            grid: ['A'; 16],
            grids,
            scores: HashMap::new(),
            player_words: HashMap::new(),
            played: HashSet::new(),
            invalid_words: HashSet::new(),
            immediate,
            turn: 0,
        }
    }


    fn update_grid(&mut self) {
        let grid = match self.next_grid() {
            Some(grid) => grid,
            None => generate_random_grid(),
        };
        self.grid = grid;
    }

    fn next_grid(&mut self) -> Option<[char; 16]> {
        if self.grids.is_empty() {
            None
        } else {
            let grid = self.grids.remove(0);
            self.grids.push(grid.clone());
            grid_of_string(&grid)
        }
    }

    /// "Mise à zéro" du plateau de jeu après une tour.
    pub fn reset(&mut self) {
        self.update_grid();
        self.invalid_words.clear();
        self.scores.values_mut().for_each(|v| *v = 0);
        self.player_words.clear();
        self.played.clear();
        self.turn = 1;
    }

    /// Renvoie une chaîne de caractères contenant le message de bienvenue.
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

    /// Renvoie une chaîne de caractères représentant les scores des joueurs.
    pub fn scores_str(&self) -> String  {
        self.scores.keys()
            .map(|u| format!("{}*{}", u, self.user_score(u)))
            .collect::<Vec<String>>()
            .join("*")
    }

    /// Renvoie une chaîne de caractères représentant les mot joués par les joueurs.
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

    /// Soummision du mot `word`, de trajectoire `trajectory` par l'utilisateur `user`.
    pub fn submit_word(&mut self, user: &str, word: &str, trajectory: &str)
        -> Result<bool, ServerError>
    {
        let t = trajectory_of_string(trajectory)?;

        if self.word_of_trajectory(&t) != word {
            return Err(ServerError::no_match(trajectory, word))
        }

        if ! self.scores.contains_key(user) {
            return Err(ServerError::non_existing_user(user))
        }

        if self.played.contains(word) {
            if self.immediate {
                return Err(ServerError::already_played(word, true))
            } else {
                self.invalid_words.insert(word.to_string());
                return Err(ServerError::already_played(word, false))
            }
        }

        self.player_words.entry(user.to_string()).or_insert(vec![])
            .push(word.to_string());
        self.played.insert(word.to_string());

        Ok(self.immediate)
    }

    /// Renvoie le mot correspondant à la trajectoire `trajectory`.
    fn word_of_trajectory(&self, trajectory: &[(char, usize)]) -> String {
        trajectory.iter() .map(|&(line, col)| {
                let idx = (4 * index_of_letter(line)) + (col - 1);
                self.grid[idx]
            })
            .collect::<String>().to_lowercase()
    }

    /// Mise à jour du plateau de jeu après un tour.
    pub fn new_turn(&mut self) {
        self.update_users_scores();
        self.update_grid();
        self.player_words.clear();
        self.played.clear();
        self.invalid_words.clear();
        self.turn += 1;
    }

    pub fn turn_scores(&mut self) -> String {
        format!("BILANMOTS/{}/{}/\n", self.words_str(), self.scores_str())
    }

    /// Ajoute les points du tour courant au score de chaque joueur.
    fn update_users_scores(&mut self) {
        let mut scores = HashMap::new();
        for (user, score) in self.scores.iter() {
            let s = score + self.turn_score(user);
            scores.insert(user.to_string(), s);
        }
        self.scores = scores;
    }

    /// Renvoie le score du joueur `user`.
    fn user_score(&self, user: &str) -> u32 {
        match self.scores.get(user) {
            Some(score) => score + self.turn_score(user),
            None => self.turn_score(user)
        }
    }

    /// Renvoie le nombre de points gagnés par le joueur `user` lors du tour courant.
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
    fn update_grid() {
        let mut board = Board::new(true, vec!["BBBBBBBBBBBBBBBB".to_string(),
                                              "CCCCCCCCCCCCCCCC".to_string()]);
        board.update_grid();
        assert_eq!(['B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B'], board.grid);
        board.update_grid();
        assert_eq!(['C', 'C', 'C', 'C', 'C', 'C', 'C', 'C', 'C', 'C', 'C', 'C', 'C', 'C', 'C', 'C'], board.grid);
        board.update_grid();
        assert_eq!(['B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B'], board.grid);
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
        board.immediate = false;
        board.submit_word("user1", "trident", "C2B1A2A3B2C3D2").unwrap();
        assert_eq!(board.user_score("user1"), 5);
    }

    #[test]
    fn submit_already_played_word() {
        let mut board = create_test_board();
        board.immediate = false;
        board.add_user("user1");
        board.add_user("user2");
        board.submit_word("user1", "trident", "C2B1A2A3B2C3D2").unwrap();
        board.submit_word("user1", "ile", "A2A1B2").unwrap();
        match board.submit_word("user2", "trident", "C2B1A2A3B2C3D2") {
            Err(ServerError::AlreadyPlayed {..}) => (),
            _ => panic!()
        };
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
        let mut board = Board::new(true, vec![]);
        board.grid = ['L', 'I', 'D', 'A',
                     'R', 'E', 'J', 'U',
                     'L', 'T', 'N', 'E',
                     'A', 'T', 'N', 'G' ];
        board.turn = 1;
        board
    }
}
