use std::error::Error as StdError;
use std::fmt;
use std::result;

use ffi::{mps_res_t, MPS_RES_COMMIT_LIMIT, MPS_RES_IO, MPS_RES_LIMIT, MPS_RES_MEMORY, MPS_RES_OK,
          MPS_RES_PARAM, MPS_RES_RESOURCE, MPS_RES_UNIMPL};
use self::Error::*;

pub type Result<T> = result::Result<T, Error>;

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
    pub fn result(res: mps_res_t) -> Result<()> {
        match res as u32 {
            r if r == MPS_RES_OK as u32 => Ok(()),
            r if r == MPS_RES_COMMIT_LIMIT as u32 => Err(CommitLimit),
            r if r == MPS_RES_IO as u32 => Err(InputOutput),
            r if r == MPS_RES_MEMORY as u32 => Err(InsufficientMemory),
            r if r == MPS_RES_RESOURCE as u32 => Err(InsufficientResources),
            r if r == MPS_RES_LIMIT as u32 => Err(InternalLimit),
            r if r == MPS_RES_PARAM as u32 => Err(InvalidParam),
            r if r == MPS_RES_UNIMPL as u32 => Err(Unimplemented),
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
