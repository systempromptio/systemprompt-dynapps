//! Eligibility, the context a chunk is embedded with, and the synopsis
//! prompt.

use dynbrain::ids::{ArticleId, SpaceId};
use dynbrain::ingest::context::{
    ChunkContext, ContextStrategy, SYNOPSIS_DOC_CHARS, clean_synopsis, document_context,
    embedding_text, is_code_chunk, prefix, synopsis_prompt, text_hash,
};
use dynbrain::ingest::eligibility::{Placement, RagRule, RuleScope, eligible};
use dynbrain::model::RagMode;

fn placement(space: i64, lineage: &[i64]) -> Placement {
    Placement {
        space: SpaceId(space),
        lineage: lineage.iter().map(|id| ArticleId(*id)).collect(),
    }
}

fn rule(scope: RuleScope, include: bool) -> RagRule {
    RagRule { scope, include }
}

#[test]
fn nothing_is_indexed_unless_something_says_so() {
    let here = placement(1, &[10, 5]);
    assert!(!eligible(RagMode::Default, &[], &here));
    assert!(!eligible(
        RagMode::Default,
        &[rule(RuleScope::Space(SpaceId(2)), true)],
        &here
    ));
}

#[test]
fn the_mode_of_an_article_beats_every_rule() {
    let here = placement(1, &[10]);
    assert!(eligible(
        RagMode::Always,
        &[rule(RuleScope::All, false)],
        &here
    ));
    assert!(!eligible(
        RagMode::Never,
        &[rule(RuleScope::All, true)],
        &here
    ));
}

#[test]
fn the_first_rule_that_matches_decides() {
    let here = placement(1, &[10, 5]);
    let article_out = rule(RuleScope::Article(ArticleId(5)), false);
    let space_in = rule(RuleScope::Space(SpaceId(1)), true);
    assert!(!eligible(RagMode::Default, &[article_out, space_in], &here));
    assert!(eligible(RagMode::Default, &[space_in, article_out], &here));
    assert!(eligible(
        RagMode::Default,
        &[
            rule(RuleScope::Article(ArticleId(99)), false),
            rule(RuleScope::All, true)
        ],
        &here
    ));
}

#[test]
fn an_article_rule_reaches_the_article_and_everything_below_it() {
    let include = [rule(RuleScope::Article(ArticleId(5)), true)];
    assert!(eligible(RagMode::Default, &include, &placement(1, &[5])));
    assert!(eligible(
        RagMode::Default,
        &include,
        &placement(1, &[10, 7, 5])
    ));
    assert!(!eligible(
        RagMode::Default,
        &include,
        &placement(1, &[6, 4])
    ));
}

#[test]
fn code_is_a_chunk_that_is_mostly_fenced() {
    assert!(is_code_chunk(
        "Run it. ```sh\ncargo test --all --features x\n``` ok"
    ));
    assert!(!is_code_chunk("Plain prose with no fence at all."));
    assert!(!is_code_chunk(
        "```\nonly an opening fence and a lot of prose after it, and more prose"
    ));
    let prose_heavy = format!("{} ```x``` {}", "word ".repeat(60), "word ".repeat(60));
    assert!(!is_code_chunk(&prose_heavy));
}

#[test]
fn the_prefix_follows_the_strategy_and_never_reaches_a_code_chunk() {
    let document = document_context("Breach policy", "What to do when data leaks");
    assert_eq!(document, "Breach policy\nWhat to do when data leaks");
    let with_path = ChunkContext {
        content: "The deadline is 72 hours.",
        heading_path: "Security > Deadlines",
        synopsis: Some("Notification deadline"),
    };
    let without_path = ChunkContext {
        heading_path: "",
        ..with_path
    };
    let expect = |strategy, chunk: &ChunkContext<'_>, want: &str| {
        assert_eq!(prefix(strategy, &document, chunk), want, "{strategy:?}");
    };
    expect(
        ContextStrategy::Auto,
        &with_path,
        &format!("{document}\nSecurity > Deadlines"),
    );
    expect(
        ContextStrategy::Auto,
        &without_path,
        &format!("{document}\nNotification deadline"),
    );
    expect(
        ContextStrategy::Heading,
        &with_path,
        &format!("{document}\nSecurity > Deadlines"),
    );
    expect(
        ContextStrategy::Llm,
        &with_path,
        &format!("{document}\nNotification deadline"),
    );
    expect(
        ContextStrategy::HeadingLlm,
        &with_path,
        &format!("{document}\nSecurity > Deadlines\nNotification deadline"),
    );
    expect(ContextStrategy::Document, &with_path, &document);
    expect(ContextStrategy::Off, &with_path, "");
    let code = ChunkContext {
        content: "```sh\ncargo test --all\n```",
        ..with_path
    };
    expect(ContextStrategy::Auto, &code, "");
}

#[test]
fn only_the_embedding_text_carries_the_prefix() {
    assert_eq!(embedding_text("Title\nPath", "Body."), "Title\nPath\nBody.");
    assert_eq!(embedding_text("", "Body."), "Body.");
    assert_eq!(text_hash("a"), text_hash("a"));
    assert_ne!(text_hash("a"), text_hash("b"));
}

#[test]
fn a_synopsis_is_needed_where_the_strategy_has_nothing_else() {
    assert!(ContextStrategy::Auto.needs_synopsis("", true));
    assert!(!ContextStrategy::Auto.needs_synopsis("A > B", true));
    assert!(ContextStrategy::Llm.needs_synopsis("A > B", true));
    assert!(ContextStrategy::HeadingLlm.needs_synopsis("A", true));
    assert!(!ContextStrategy::Heading.needs_synopsis("", true));
    assert!(!ContextStrategy::Auto.needs_synopsis("", false));
    assert_eq!(
        ContextStrategy::parse("heading_llm"),
        Some(ContextStrategy::HeadingLlm)
    );
    assert_eq!(ContextStrategy::parse("nope"), None);
}

#[test]
fn the_synopsis_prompt_names_the_language_and_cuts_a_long_document() {
    let spanish = synopsis_prompt("es", "doc", "excerpt");
    assert!(
        spanish.system.contains("written in Spanish (es)."),
        "{}",
        spanish.system
    );
    assert!(spanish.system.contains("at most 30 words"));
    assert_eq!(
        spanish.user,
        "<document>\ndoc\n</document>\n\n<excerpt>\nexcerpt\n</excerpt>"
    );
    assert!(
        synopsis_prompt("", "d", "e")
            .system
            .contains("the SAME language as the document")
    );
    assert!(
        synopsis_prompt("xx", "d", "e")
            .system
            .contains("written in xx.")
    );
    let long = "é".repeat(SYNOPSIS_DOC_CHARS + 10);
    let cut = synopsis_prompt("en", &long, "e").user;
    assert!(cut.contains(&format!(
        "{}\n[...truncated...]\n</document>",
        "é".repeat(SYNOPSIS_DOC_CHARS)
    )));
}

#[test]
fn a_synopsis_is_one_clean_line() {
    assert_eq!(
        clean_synopsis("  \"A  sentence\n about it.\"  "),
        "A sentence about it."
    );
    assert_eq!(clean_synopsis("'quoted'"), "quoted");
    assert_eq!(clean_synopsis("   "), "");
}
