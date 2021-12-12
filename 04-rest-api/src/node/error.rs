use std::error::Error;
pub type NodeResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub enum NodeError {
    NodeStopped,
    NodeRunning,
}
impl Error for NodeError {}

impl std::fmt::Display for NodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NodeStopped => write!(f, "node stopped"),
            Self::NodeRunning => write!(f, "node already running"),
        }
    }
}
