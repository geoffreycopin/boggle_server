

#[derive(Debug, Clone, Fail)]
pub enum ServerError {
    #[fail(display = "L'utilisateur {} existe déja.", username)]
    ExistingUser { username: String },

    #[fail(display = "L'utilisateur {} n'existe pas.", username)]
    NonExistingUser { username: String },

    #[fail(display = "Les coordonnées ({}, {}) sont invalides.", line, column)]
    InvalidCoordinates { line: char, column: usize},

    #[fail(display = "Requête invalide: {}", request)]
    BadRequest { request: String },

    #[fail(display = "Le mot {} n'existe pas.", word)]
    NonExistingWord { word: String },

    #[fail(display = "Le mot {} a déjà ete joué.", word)]
    AlreadyPlayed { word: String },

    #[fail(display = "La trajectoire {} est invalide.", trajectory)]
    BadTrajectory { trajectory: String},
}

impl ServerError {
    pub fn existing_user(username: String) -> ServerError {
        ServerError::ExistingUser { username }
    }

    pub fn non_existing_user(username: String) -> ServerError {
        ServerError::NonExistingUser { username }
    }

    pub fn invalid_coordinates(line: char, column: usize) -> ServerError {
        ServerError::InvalidCoordinates { line, column }
    }

    pub fn bad_request(request: String) -> ServerError {
        ServerError::BadRequest { request }
    }

    pub fn non_existing_word(word: String) -> ServerError {
        ServerError::NonExistingWord { word }
    }

    pub fn already_played(word: String) -> ServerError {
        ServerError::AlreadyPlayed { word }
    }

    pub fn bad_trajectory(trajectory: String) -> ServerError {
        ServerError::BadTrajectory { trajectory }
    }
}