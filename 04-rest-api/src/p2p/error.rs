use std::error::Error;
pub type P2PResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub enum P2PError {
    ServerRunning,
}
impl Error for P2PError {}

impl std::fmt::Display for P2PError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ServerRunning => write!(f, "server already running"),
        }
    }
}
