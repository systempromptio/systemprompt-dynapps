//! Evaluation domain failures retain their HTTP classification.

use super::AdminError;

impl From<systemprompt::evaluation::EvaluationError> for AdminError {
    fn from(error: systemprompt::evaluation::EvaluationError) -> Self {
        use systemprompt::evaluation::EvaluationError;
        match error {
            EvaluationError::ResourceNotFound(message) => Self::NotFound(message),
            EvaluationError::ExperimentConflict(message) => Self::Conflict(message),
            EvaluationError::InvalidSpec(message) => Self::Unprocessable(message),
            EvaluationError::BudgetExhausted { .. } => Self::Conflict(error.to_string()),
            EvaluationError::RunNotFound(message) | EvaluationError::RubricNotFound(message) => {
                Self::NotFound(message)
            },
            other => Self::internal(other),
        }
    }
}
