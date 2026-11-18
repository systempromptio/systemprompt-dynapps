//! The `OneNote` tools against a simulated Graph: notebooks and sections with
//! the Belgian `$select`, a page's HTML handed back as text, and a new page
//! posted as an HTML document whose title is escaped.

use serde_json::json;
use systemprompt_mcp_microsoft_365::tools::onenote::{
    create_notebook, create_page, create_section, escape_html, get_page_content,
    list_notebook_sections, list_notebooks, list_section_pages, list_sections,
};

use crate::m365_pim_support::{args, assert_domain_registered, graph, is_invalid};
use crate::m365_support::{Reply, Sim, collection};

const FIELDS: &str = "id,displayName,createdDateTime,lastModifiedDateTime";

#[test]
fn m365_rest_onenote_all_eight_belgian_tools_are_registered_as_pinned() {
    assert_domain_registered("onenote", 8);
}

#[tokio::test]
async fn m365_rest_onenote_notebooks_sections_and_pages_are_listed() {
    let sim = Sim::start(|_, _| Reply::json(200, &collection(&[json!({"id": "n"})], None))).await;
    let client = graph(&sim);
    list_notebooks(&client, args(json!({"skip": 2})))
        .await
        .unwrap();
    list_notebook_sections(&client, args(json!({"notebook_id": "1-nb!1"})))
        .await
        .unwrap();
    list_sections(&client, args(json!({}))).await.unwrap();
    let pages = list_section_pages(&client, args(json!({"section_id": "1-s", "top": 0})))
        .await
        .unwrap();
    let bad = list_notebook_sections(&client, args(json!({"notebook_id": "a/b"})))
        .await
        .unwrap_err();

    let calls = sim.calls();
    assert_eq!(calls.len(), 4);
    assert_eq!(calls[0].path, "/v1.0/me/onenote/notebooks");
    assert_eq!(
        (calls[0].q("$select"), calls[0].q("$skip")),
        (Some(FIELDS), Some("2"))
    );
    assert_eq!(calls[1].path, "/v1.0/me/onenote/notebooks/1-nb!1/sections");
    assert_eq!(calls[1].q("$select"), Some(FIELDS));
    assert_eq!(calls[2].path, "/v1.0/me/onenote/sections");
    assert_eq!(calls[3].path, "/v1.0/me/onenote/sections/1-s/pages");
    assert_eq!(calls[3].q("$top"), Some("1"), "top is clamped to 1-50");
    assert!(calls[3].q("$select").is_none());
    assert_eq!(pages.data, vec![json!({"id": "n"})]);
    assert!(is_invalid(&bad));
}

#[tokio::test]
async fn m365_rest_onenote_page_content_comes_back_as_its_html() {
    let html = "<html><body><p>Café &amp; notes</p></body></html>";
    let sim = Sim::start(move |_, _| {
        Reply::bytes(200, html.as_bytes()).with_header("content-type", "text/html")
    })
    .await;
    let page = get_page_content(&graph(&sim), args(json!({"page_id": "1-p!2"})))
        .await
        .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.path, "/v1.0/me/onenote/pages/1-p!2/content");
    assert_eq!(
        serde_json::to_value(&page).unwrap(),
        json!({"data": {"html": html, "page_id": "1-p!2"}, "message": "success"})
    );
}

#[tokio::test]
async fn m365_rest_onenote_notebook_section_and_page_are_created() {
    let sim = Sim::start(|_, _| Reply::json(201, &json!({"id": "new"}))).await;
    let client = graph(&sim);
    create_notebook(&client, args(json!({"display_name": "Work"})))
        .await
        .unwrap();
    create_section(
        &client,
        args(json!({"notebook_id": "1-nb", "display_name": "Q1"})),
    )
    .await
    .unwrap();
    let page = create_page(
        &client,
        args(json!({
            "section_id": "1-s", "title": "R&D <plan> \"v2\" 'x'", "html_content": "<p>Hi</p>"
        })),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/me/onenote/notebooks");
    assert_eq!(calls[0].json(), json!({"displayName": "Work"}));
    assert_eq!(calls[1].path, "/v1.0/me/onenote/notebooks/1-nb/sections");
    assert_eq!(calls[1].json(), json!({"displayName": "Q1"}));
    assert_eq!(calls[2].method, "POST");
    assert_eq!(calls[2].path, "/v1.0/me/onenote/sections/1-s/pages");
    assert_eq!(calls[2].header("content-type"), Some("text/html"));
    assert_eq!(
        String::from_utf8(calls[2].body.clone()).unwrap(),
        "<html><head><title>R&amp;D &lt;plan&gt; &quot;v2&quot; &#x27;x&#x27;</title></head>\
         <body><p>Hi</p></body></html>"
    );
    assert_eq!(page.data["id"], "new");
    assert_eq!(escape_html("a&b"), "a&amp;b");
}
