//! Judge failures as the console sees them: a database fault keeps its
//! status, everything else is the server's own problem.

use super::AdminError;
use crate::evaluation::JudgeError;

impl From<JudgeError> for AdminError {
    fn from(value: JudgeError) -> Self {
        match value {
            JudgeError::Db(e) => Self::Database(e),
            JudgeError::UnknownCriterion(_) | JudgeError::Context(_) => Self::internal(value),
        }
    }
}
