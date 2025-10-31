use thiserror::Error;

#[derive(Error, Debug)]
pub enum Fit2SrtError {
    #[error("Can not merge: `{0}`")]
    MergeError(String),
    #[error("Parse Summary Error: {0}")]
    SummaryError(String),
}
