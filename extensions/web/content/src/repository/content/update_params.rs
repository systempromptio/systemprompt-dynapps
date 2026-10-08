//! `UpdateContentParams` and its builder: the required fields arrive as one
//! `UpdateContentSeed`, the optional image and links are layered on after.

use sqlx::types::Json;
use systemprompt::identifiers::ContentId;
use systemprompt_web_shared::models::ContentLinkMetadata;

#[derive(Debug, Clone)]
pub struct UpdateContentParams {
    pub id: ContentId,
    pub title: String,
    pub description: String,
    pub body: String,
    pub keywords: String,
    pub image: Option<String>,
    pub version_hash: String,
    pub links: Option<Json<Vec<ContentLinkMetadata>>>,
}

/// Required fields for [`UpdateContentParams`]; optional fields are layered
/// on via the `UpdateContentParamsBuilder::with_*` methods.
#[derive(Debug, Clone)]
pub struct UpdateContentSeed {
    pub id: ContentId,
    pub title: String,
    pub description: String,
    pub body: String,
    pub keywords: String,
    pub version_hash: String,
}

impl UpdateContentParams {
    #[must_use]
    pub fn builder(seed: UpdateContentSeed) -> UpdateContentParamsBuilder {
        UpdateContentParamsBuilder::new(seed)
    }
}

#[derive(Debug)]
pub struct UpdateContentParamsBuilder {
    id: ContentId,
    title: String,
    description: String,
    body: String,
    keywords: String,
    version_hash: String,
    image: Option<String>,
    links: Option<Json<Vec<ContentLinkMetadata>>>,
}

impl UpdateContentParamsBuilder {
    #[must_use]
    pub fn new(seed: UpdateContentSeed) -> Self {
        Self {
            id: seed.id,
            title: seed.title,
            description: seed.description,
            body: seed.body,
            keywords: seed.keywords,
            version_hash: seed.version_hash,
            image: None,
            links: None,
        }
    }

    #[must_use]
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    #[must_use]
    pub fn with_links(mut self, links: Vec<ContentLinkMetadata>) -> Self {
        self.links = Some(Json(links));
        self
    }

    #[must_use]
    pub fn build(self) -> UpdateContentParams {
        UpdateContentParams {
            id: self.id,
            title: self.title,
            description: self.description,
            body: self.body,
            keywords: self.keywords,
            version_hash: self.version_hash,
            image: self.image,
            links: self.links,
        }
    }
}
