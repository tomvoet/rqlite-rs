use std::convert::Infallible;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueryBuilderError {
    /// An invalid amount of arguments were provided.
    #[error("Invalid Argument Count: Expected {0} arguments, got {1}")]
    InvalidArgumentCount(usize, usize),
    /// An invalid query operation was provided.
    #[error("Invalid Query Operation: {0}")]
    InvalidOperation(String),
    /// An invalid query was provided.
    #[error("Invalid Query: {0}")]
    InvalidQuery(String),
}

#[derive(Error, Debug)]
pub enum ClientBuilderError {
    /// No hosts were provided to the client builder.
    #[error("No hosts provided")]
    NoHostsProvided,
    /// An error occurred while building the reqwest client.
    #[error("Reqwest Error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}

#[derive(Error, Debug)]
pub enum RequestError {
    /// An error occurred while sending a request to the rqlite cluster.
    #[error("Reqwest Error: {body} - {status:?}")]
    ReqwestError {
        body: String,
        status: reqwest::StatusCode,
    },
    /// No available hosts to send the request to.
    #[error("No available hosts")]
    NoAvailableHosts,
    /// An unhandled error occured while switching to a different host.
    #[error("Switchover - Wrong Error: {0}")]
    SwitchoverWrongError(String),
    /// An invalid query was provided.
    #[error("Invalid Query: {0}")]
    InvalidQuery(#[from] QueryBuilderError),
    /// An error occurred while parsing the request body.
    #[error("Failed to parse request body: {0}")]
    FailedParseRequestBody(#[source] serde_json::Error),
    /// An error occurred while parsing the response body.
    #[error("Failed to parse response body: {0}")]
    FailedParseResponseBody(#[source] serde_json::Error),
    /// An error occurred while reading the response body.
    #[error("Failed to parse response body: {0}")]
    FailedReadingResponse(#[from] reqwest::Error),
    /// No Rows were returned from the query.
    #[error("No rows returned")]
    NoRowsReturned,
    /// The database returned an error.
    #[error("Database Error: {0}")]
    DatabaseError(String),
}

// This is a conversion from the `Infallible` type to the `RequestError` type.
// Weird hack, but it's necessary for the `?` operator to work in the `query!` macro.
impl From<Infallible> for RequestError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}
