use thiserror::Error;
#[derive(Error, Debug)]
pub enum ResourceIOError {
    #[error("InsufficientPermissions: cannot {0} resource")]
    InsufficientPermissions(String)
}

#[derive(Error, Debug)]
pub enum UserError {
    #[error("MismatchingCredential: cannot login")]
    MismatchingCredential
}