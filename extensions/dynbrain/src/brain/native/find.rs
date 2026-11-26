//! `search` and `answer`: the retriever's passages as `docs_search` and
//! `docs_answer` with `synthesize: false` answer them.
//!
//! `where` must name properties some space declares. `lang`, when given,
//! keeps the search to the spaces in that language and is the preferred
//! variant; otherwise the preferred variant is the language the query reads
//! as, among the corpus's. Paths hide the ancestors the reader cannot read.
//! No model is called here: `answer` assembles each source's text (whole, or
//! its best chunks in reading order) and numbers the blocks.

use std::collections::{BTreeSet, HashMap};

use systemprompt::identifiers::UserId;

use super::NativeBrain;
use super::shape::path_of;
use crate::assets::markdown::index_text;
use crate::brain::Reply;
use crate::brain::args::{AnswerArgs, SearchArgs};
use crate::brain::fault::BrainFault;
use crate::brain::found::{Answer, AnswerMode, Citation, SearchHit, SearchResults};
use crate::ids::{ArticleId, SpaceId};
use crate::model::Space;
use crate::search::confidence::{round_to, verdict};
use crate::search::context::{ContextSource, blocks, document_text};
use crate::search::filter::PropertyFilter;
use crate::search::lang::detect;
use crate::search::query::QueryVector;
use crate::search::service::{Passage, RetrievalRequest, SearchScope};

const SNIPPET_CHARS: usize = 300;

impl NativeBrain {
    pub(super) async fn find(&self, user: &UserId, args: &SearchArgs) -> Reply<SearchResults> {
        let query = args.query.trim();
        let spaces = self.repo.list_spaces().await?;
        let filter = PropertyFilter::parse(args.filter.as_ref(), &declared(&spaces))?;
        let scope = scope_of(user, args.space, args.lang.as_deref(), filter);
        let prefer = prefer_lang(query, scope.lang.as_deref(), &spaces);
        let request = RetrievalRequest {
            query,
            scope: &scope,
            limit: bounded(args.limit, 10, 50),
            prefer_lang: prefer.as_deref(),
        };
        let found = self.search.retrieve(&request).await?;
        let mut paths = self.paths_of(user, &spaces, &found.passages).await?;
        let results: Vec<SearchHit> = found
            .passages
            .into_iter()
            .map(|passage| SearchHit {
                path: paths.remove(&passage.article).unwrap_or_else(String::new),
                article_id: passage.article,
                space: passage.space,
                snippet: snippet(&passage.content),
                title: passage.title,
                description: passage.description,
                lang: passage.lang,
                score: passage.score.map(|score| round_to(score, 5)),
            })
            .collect();
        let top = results.first().and_then(|hit| hit.score);
        let threshold = self.search.settings().weak_match_threshold;
        Ok(SearchResults {
            query: query.to_owned(),
            mode: found.mode,
            total: results.len(),
            verdict: verdict(top, found.trace.scoring, threshold),
            results,
        })
    }

    pub(super) async fn assemble(&self, user: &UserId, args: &AnswerArgs) -> Reply<Answer> {
        let question = args.question.trim();
        let spaces = self.repo.list_spaces().await?;
        let scope = scope_of(
            user,
            args.space,
            args.lang.as_deref(),
            PropertyFilter::default(),
        );
        let prefer = prefer_lang(question, scope.lang.as_deref(), &spaces);
        let request = RetrievalRequest {
            query: question,
            scope: &scope,
            limit: bounded(args.max_sources, 8, 20),
            prefer_lang: prefer.as_deref(),
        };
        let found = self.search.retrieve(&request).await?;
        let paths = self.paths_of(user, &spaces, &found.passages).await?;
        let mut sources = Vec::new();
        let mut citations = Vec::new();
        for passage in &found.passages {
            let path = paths
                .get(&passage.article)
                .cloned()
                .unwrap_or_else(String::new);
            sources.push(ContextSource {
                title: passage.title.clone(),
                path: path.clone(),
                lang: passage.lang.clone(),
                text: self.source_text(passage, found.vector.as_ref()).await?,
            });
            citations.push(Citation {
                path,
                title: passage.title.clone(),
                lang: passage.lang.clone(),
                snippet: snippet(&passage.content),
            });
        }
        let top = found.passages.first().and_then(|passage| passage.score);
        let threshold = self.search.settings().weak_match_threshold;
        Ok(Answer {
            context: blocks(&sources),
            citations,
            mode: AnswerMode::Context,
            verdict: verdict(top, found.trace.scoring, threshold),
        })
    }

    async fn source_text(
        &self,
        passage: &Passage,
        vector: Option<&QueryVector>,
    ) -> Result<String, BrainFault> {
        let markdown = self
            .repo
            .find_variant(passage.article, &passage.lang)
            .await?
            .map_or_else(String::new, |variant| index_text(&variant.body));
        let ranked = match vector {
            Some(vector) => {
                self.repo
                    .list_context_chunks(passage.article, &passage.lang, vector)
                    .await?
            },
            None => Vec::new(),
        };
        let quota = self.search.settings().context_quota_chars;
        Ok(document_text(&markdown, quota, &ranked))
    }

    async fn paths_of(
        &self,
        user: &UserId,
        spaces: &[Space],
        passages: &[Passage],
    ) -> Result<HashMap<ArticleId, String>, BrainFault> {
        let touched: BTreeSet<SpaceId> = passages.iter().map(|passage| passage.space).collect();
        let mut paths = HashMap::new();
        for space in spaces.iter().filter(|space| touched.contains(&space.id)) {
            let all = self.repo.list_articles(space.id).await?;
            let seen = self.seen(space.id, user).await?;
            for article in all
                .iter()
                .filter(|article| passages.iter().any(|passage| passage.article == article.id))
            {
                paths.insert(article.id, path_of(space, article, &all, &seen));
            }
        }
        Ok(paths)
    }
}

fn scope_of(
    user: &UserId,
    space: Option<SpaceId>,
    lang: Option<&str>,
    filter: PropertyFilter,
) -> SearchScope {
    SearchScope {
        reader: user.clone(),
        space,
        lang: lang
            .map(str::trim)
            .filter(|lang| !lang.is_empty())
            .map(ToOwned::to_owned),
        filter,
    }
}

fn declared(spaces: &[Space]) -> BTreeSet<String> {
    spaces
        .iter()
        .flat_map(|space| space.properties.names())
        .collect()
}

fn prefer_lang(text: &str, explicit: Option<&str>, spaces: &[Space]) -> Option<String> {
    if let Some(lang) = explicit {
        return Some(lang.to_owned());
    }
    let mut corpus: Vec<String> = Vec::new();
    for lang in spaces.iter().flat_map(|space| space.langs.iter()) {
        if !corpus.contains(lang) {
            corpus.push(lang.clone());
        }
    }
    detect(text, &corpus)
}

fn bounded(asked: Option<u32>, default: u32, max: u32) -> usize {
    asked.filter(|count| *count > 0).unwrap_or(default).min(max) as usize
}

fn snippet(text: &str) -> String {
    text.chars().take(SNIPPET_CHARS).collect()
}
