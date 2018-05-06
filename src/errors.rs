

#[derive(Debug, Clone, Fail)]
pub enum ServerError {
    #[fail(display = "L'utilisateur {} existe déja.", username)]
    ExistingUser { username: String },

    #[fail(display = "L'utilisateur {} n'existe pas.", username)]
    NonExistingUser { username: String },

    #[fail(display = "Les coordonnées ({}, {}) sont invalides.", line, column)]
    InvalidCoordinates { line: char, column: usize },

    #[fail(display = "Requête invalide: {}.", request)]
    BadRequest { request: String },

    #[fail(display = "Le mot {} n'existe pas.", word)]
    NonExistingWord { word: String },

    #[fail(display = "Le mot {} a déjà ete joué.", word)]
    AlreadyPlayed { word: String },

    #[fail(display = "La trajectoire {} est invalide.", trajectory)]
    BadTrajectory { trajectory: String },

    #[fail(display = "La trajectoire {} ne correspond pas au mot {}.", trajectory, word)]
    NoMatch { word: String, trajectory: String },

    #[fail(display = "La requête <{}> ne peut être soumise par un utilisateur non connecté.", request)]
    UnauthorizedRequest { request: String },

    #[fail(display = "Le message <{}> soumis par {} n'a pas pu être envoyé à {}: {}", message, sender, receiver, err)]
    InvalidChat { sender: String, receiver: String, message: String, err: Box<ServerError> },
}

impl ServerError {
    pub fn existing_user(username: &str) -> ServerError {
        ServerError::ExistingUser {
            username: username.to_string()
        }
    }

    pub fn non_existing_user(username: &str) -> ServerError {
        ServerError::NonExistingUser {
            username: username.to_string()
        }
    }

    pub fn invalid_coordinates(line: char, column: usize) -> ServerError {
        ServerError::InvalidCoordinates { line, column }
    }

    pub fn bad_request(request: &str) -> ServerError {
        ServerError::BadRequest {
            request: request.to_string()
        }
    }

    pub fn non_existing_word(word: &str) -> ServerError {
        ServerError::NonExistingWord {
            word: word.to_string()
        }
    }

    pub fn already_played(word: &str) -> ServerError {
        ServerError::AlreadyPlayed {
            word: word.to_string()
        }
    }

    pub fn bad_trajectory(trajectory: &str) -> ServerError {
        ServerError::BadTrajectory {
            trajectory: trajectory.to_string()
        }
    }

    pub fn no_match(trajectory: &str , word: &str) -> ServerError {
        ServerError::NoMatch {
            trajectory: trajectory.to_string(),
            word: word.to_string(),
        }
    }

    pub fn unauthorized_request(request: &str) -> ServerError {
        ServerError::UnauthorizedRequest {
            request: request.to_string()
        }
    }

    pub fn invalid_chat(sender: &str, receiver: &str, msg: &str, err: ServerError) -> ServerError {
        ServerError::InvalidChat {
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            message: msg.to_string(),
            err: Box::new(err)
        }
    }
}