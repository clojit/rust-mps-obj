//! Error handling types

use std::error::Error as StdError;
use std::fmt;
use std::result;

use ffi::{mps_res_t, MPS_RES_COMMIT_LIMIT, MPS_RES_IO, MPS_RES_LIMIT, MPS_RES_MEMORY, MPS_RES_OK,
          MPS_RES_PARAM, MPS_RES_RESOURCE, MPS_RES_UNIMPL};
use self::Error::*;

/// The Rust equivalent of `mps_res_t`
pub type Result<T> = result::Result<T, Error>;

/// Errors returned by the Memory Pool System.
///
/// Please refer to the [error handling](https://www.ravenbrook.com/project/mps/master/manual/html/topic/error.html)
/// chapter in the Memory Pool System reference for more details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    CommitLimit,
    InputOutput,
    InsufficientMemory,
    InsufficientResources,
    InternalLimit,
    InvalidParam,
    Unimplemented,
    Other,
}

impl Error {
    /// Constructs a `Result` instance from a raw `mps_res_t` value.
    /// Occurences of `MPS_RES_OK` are converted to `Ok(())`.
    pub fn result(res: mps_res_t) -> Result<()> {
        match res as u32 {
            MPS_RES_OK => Ok(()),
            MPS_RES_COMMIT_LIMIT => Err(CommitLimit),
            MPS_RES_IO => Err(InputOutput),
            MPS_RES_MEMORY => Err(InsufficientMemory),
            MPS_RES_RESOURCE => Err(InsufficientResources),
            MPS_RES_LIMIT => Err(InternalLimit),
            MPS_RES_PARAM => Err(InvalidParam),
            MPS_RES_UNIMPL => Err(Unimplemented),
            _ => Err(Other),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            CommitLimit => "the arena’s commit limit would be exceeded.",
            InputOutput => "an input/output error occurred.",
            InsufficientMemory => "needed memory could not be obtained.",
            InsufficientResources => "a needed resource could not be obtained.",
            InternalLimit => "an internal limitation was exceeded.",
            InvalidParam => "an invalid parameter was passed.",
            Unimplemented => "operation is not implemented.",
            Other => "operation failed.",
        }
    }
}
