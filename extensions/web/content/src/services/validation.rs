//! Editorial validation applied to content before it is published.

use systemprompt_web_shared::error::BlogError;
use systemprompt_web_shared::models::{ContentKind, ContentMetadata};

const MAX_TITLE_LENGTH: usize = 200;
const MAX_DESCRIPTION_LENGTH: usize = 500;
const MIN_BODY_LENGTH: usize = 100;

#[derive(Debug, Clone, Default, Copy)]
pub struct ValidationService;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl ValidationResult {
    #[must_use]
    pub const fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    #[must_use]
    pub const fn invalid(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    #[must_use]
    pub fn with_error(mut self, error: ValidationError) -> Self {
        self.is_valid = false;
        self.errors.push(error);
        self
    }
}

impl ValidationError {
    #[must_use]
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

impl ValidationService {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn validate_metadata(metadata: &ContentMetadata) -> ValidationResult {
        let errors: Vec<ValidationError> = [
            title_error(&metadata.title),
            slug_error(&metadata.slug),
            published_at_error(&metadata.published_at),
            kind_error(&metadata.kind),
            description_error(&metadata.description),
        ]
        .into_iter()
        .flatten()
        .collect();
        let warnings: Vec<String> = [
            (
                &metadata.description,
                "Description is empty - this may affect SEO",
            ),
            (&metadata.author, "Author is not specified"),
            (
                &metadata.keywords,
                "Keywords are empty - this may affect SEO",
            ),
        ]
        .into_iter()
        .filter(|(value, _)| value.trim().is_empty())
        .map(|(_, warning)| warning.to_owned())
        .collect();
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    #[must_use]
    pub fn validate_body(body: &str) -> ValidationResult {
        let mut result = ValidationResult::valid();

        if body.trim().is_empty() {
            result =
                result.with_error(ValidationError::new("body", "Content body cannot be empty"));
        } else if body.len() < MIN_BODY_LENGTH {
            result = result.with_warning(format!(
                "Content body is very short (< {MIN_BODY_LENGTH} characters)"
            ));
        }

        result
    }

    #[must_use]
    pub fn validate_url(url: &str, field_name: &str) -> ValidationResult {
        let mut result = ValidationResult::valid();

        if url.trim().is_empty() {
            return result;
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            result = result.with_error(ValidationError::new(
                field_name,
                "URL must start with http:// or https://",
            ));
        }

        result
    }

    pub fn validate_content(metadata: &ContentMetadata, body: &str) -> Result<(), BlogError> {
        let metadata_result = Self::validate_metadata(metadata);
        let body_result = Self::validate_body(body);

        let mut all_errors = metadata_result.errors;
        all_errors.extend(body_result.errors);

        if all_errors.is_empty() {
            Ok(())
        } else {
            let error_messages: Vec<String> = all_errors
                .iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect();
            Err(BlogError::Validation(error_messages.join("; ")))
        }
    }
}

fn title_error(title: &str) -> Option<ValidationError> {
    if title.trim().is_empty() {
        Some(ValidationError::new("title", "Title cannot be empty"))
    } else if title.len() > MAX_TITLE_LENGTH {
        Some(ValidationError::new(
            "title",
            format!("Title cannot exceed {MAX_TITLE_LENGTH} characters"),
        ))
    } else {
        None
    }
}

fn slug_error(slug: &str) -> Option<ValidationError> {
    if slug.trim().is_empty() {
        Some(ValidationError::new("slug", "Slug cannot be empty"))
    } else if is_valid_slug(slug) {
        None
    } else {
        Some(ValidationError::new(
            "slug",
            "Slug must contain only lowercase letters, numbers, and hyphens",
        ))
    }
}

fn published_at_error(published_at: &str) -> Option<ValidationError> {
    published_at
        .trim()
        .is_empty()
        .then(|| ValidationError::new("published_at", "Published date is required"))
}

fn kind_error(kind: &str) -> Option<ValidationError> {
    kind.parse::<ContentKind>().is_err().then(|| {
        ValidationError::new(
            "kind",
            format!(
                "Invalid content kind '{kind}'. Must be one of: blog, guide, tutorial, reference, docs-index, docs, docs-list, feature, playbook, legal"
            ),
        )
    })
}

fn description_error(description: &str) -> Option<ValidationError> {
    let oversized = !description.trim().is_empty() && description.len() > MAX_DESCRIPTION_LENGTH;
    oversized.then(|| {
        ValidationError::new(
            "description",
            format!("Description cannot exceed {MAX_DESCRIPTION_LENGTH} characters"),
        )
    })
}

fn is_valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && !slug.contains("--")
}
