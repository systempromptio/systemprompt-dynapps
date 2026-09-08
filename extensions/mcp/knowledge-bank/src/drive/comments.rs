//! Attributed Google Drive comments and replies.

use super::google::{GoogleDrive, pairs};
use crate::error::KnowledgeBankError;
use serde::Deserialize;

impl GoogleDrive {
    pub async fn comments(&self, id: &str) -> Result<String, KnowledgeBankError> {
        let mut page = String::new();
        let mut text = String::new();
        loop {
            let result: Comments = self.get(&format!("files/{}/comments", urlencoding::encode(id)), pairs(&[("fields", "nextPageToken,comments(content,deleted,author(displayName),replies(content,deleted,author(displayName)))"), ("pageSize", "100"), ("pageToken", &page)])).await?;
            for comment in result.comments {
                append_comment(&mut text, &comment);
                for reply in &comment.replies {
                    append_comment(&mut text, reply);
                }
            }
            match result.next_page_token {
                Some(next) => page = next,
                None => break,
            }
        }
        Ok(text)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Comments {
    #[serde(default)]
    comments: Vec<Comment>,
    next_page_token: Option<String>,
}
#[derive(Deserialize)]
struct Comment {
    #[serde(default)]
    content: String,
    #[serde(default)]
    deleted: bool,
    author: Option<Author>,
    #[serde(default)]
    replies: Vec<Self>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Author {
    display_name: Option<String>,
}
fn append_comment(text: &mut String, comment: &Comment) {
    if !comment.deleted {
        let author = comment
            .author
            .as_ref()
            .and_then(|a| a.display_name.as_deref())
            .unwrap_or("Unknown author");
        text.push_str(&format!("\nComment by {author}: {}\n", comment.content));
    }
}
