//! Drive mapping, export coverage and extraction provenance contracts.

use std::collections::HashMap;
use systemprompt_mcp_knowledge_bank::config::KnowledgeBankConfig;
use systemprompt_mcp_knowledge_bank::drive::config::DriveSource;
use systemprompt_mcp_knowledge_bank::drive::google::export_format;
use systemprompt_mcp_knowledge_bank::drive::types::{DriveFile, assign_project};
use systemprompt_mcp_knowledge_bank::drive::{office, output};
use systemprompt_mcp_knowledge_bank::tools::{SearchKnowledgeInput, SyncKnowledgeInput};

fn source() -> DriveSource {
    DriveSource {
        drive_id: "drive".to_owned(),
        credential_secret: "google_drive_service_account".to_owned(),
        projects_folder: "Projects".to_owned(),
        shared_folder: "Shared".to_owned(),
    }
}
fn file(id: &str, name: &str, parent: &str) -> DriveFile {
    DriveFile {
        id: id.to_owned(),
        name: name.to_owned(),
        parents: vec![parent.to_owned()],
        ..Default::default()
    }
}

#[test]
fn project_assignment_survives_rename_and_tracks_move() {
    let mut tree: HashMap<_, _> = [
        file("projects", "Projects", "drive"),
        file("p1", "Launch", "projects"),
        file("sub", "Briefs", "p1"),
        file("shared", "Shared", "drive"),
    ]
    .into_iter()
    .map(|f| (f.id.clone(), f))
    .collect();
    let mut document = file("doc", "Brief.docx", "sub");
    let assignment = assign_project(&document, &tree, &source());
    assert_eq!(assignment.id, "p1");
    assert_eq!(assignment.path, "Projects/Launch/Briefs/Brief.docx");
    tree.get_mut("p1").unwrap().name = "Launch 2027".to_owned();
    assert_eq!(assign_project(&document, &tree, &source()).id, "p1");
    assert_eq!(
        assign_project(&document, &tree, &source()).name,
        "Launch 2027"
    );
    document.parents = vec!["shared".to_owned()];
    assert_eq!(assign_project(&document, &tree, &source()).id, "shared");
}

#[test]
fn root_files_missing_ancestors_and_cycles_are_unassigned() {
    let tree = HashMap::from([
        ("a".to_owned(), file("a", "Projects", "b")),
        ("b".to_owned(), file("b", "Cycle", "a")),
    ]);
    for parent in ["drive", "missing", "a"] {
        assert_eq!(
            assign_project(&file("doc", "brief.txt", parent), &tree, &source()).id,
            "unassigned"
        );
    }
}

#[test]
fn google_exports_preserve_multisheet_and_slide_formats() {
    assert_eq!(
        export_format("application/vnd.google-apps.spreadsheet")
            .unwrap()
            .1,
        "xlsx"
    );
    assert_eq!(
        export_format("application/vnd.google-apps.presentation")
            .unwrap()
            .1,
        "pptx"
    );
    assert_eq!(
        export_format("application/vnd.google-apps.document")
            .unwrap()
            .1,
        "docx"
    );
    assert!(export_format("application/pdf").is_none());
}

#[test]
fn bda_output_preserves_pages_timestamps_and_generated_provenance() {
    let body=output::normalize(br#"{"pages":[{"page_index":2,"text":"Printed launch date"}],"audio":{"segments":[{"start_timestamp_millis":1500,"end_timestamp_millis":3000,"text":"Launch is Friday","speaker_id":"speaker_1"}]},"image":{"summary":"A blue logo"}}"#).unwrap();
    assert!(body.contains("page_index=2"));
    assert!(body.contains("start_timestamp_millis=1500"));
    assert!(body.contains("Launch is Friday"));
    assert!(body.contains("Generated description\nA blue logo"));
    assert!(body.contains("Extracted content\nPrinted launch date"));
}

#[test]
fn slide_text_decodes_entities_and_keeps_paragraphs() {
    let text=office::xml_text("<p:sp><a:p><a:r><a:t>Research &amp; Growth</a:t></a:r></a:p><a:p><a:r><a:t>2027</a:t></a:r></a:p></p:sp>").unwrap();
    assert!(text.contains("Research & Growth"));
    assert!(text.contains('\n'));
    assert!(text.contains("2027"));
}

#[test]
fn manual_sync_and_project_search_defaults_remain_compatible() {
    let input: SearchKnowledgeInput = serde_json::from_str(r#"{"query":"launch"}"#).unwrap();
    assert!(input.include_shared);
    assert!(input.project_ids.is_empty());
    let sync: SyncKnowledgeInput =
        serde_json::from_str(r#"{"source_types":["systemprompt_marketing"],"force_resync":true}"#)
            .unwrap();
    assert!(sync.force_resync);
    assert!(!sync.rebuild);
}

#[test]
fn enabled_drive_requires_eu_extraction_and_service_configuration() {
    let result = KnowledgeBankConfig::parse(
        "google_drive: {enabled: true, extraction: {region: us-east-1}}",
    );
    assert!(result.is_err());
}

#[test]
fn unknown_binary_formats_are_explicitly_unsupported() {
    assert!(!systemprompt_mcp_knowledge_bank::drive::extract::supported(
        "psd"
    ));
    assert!(systemprompt_mcp_knowledge_bank::drive::extract::supported(
        "mp4"
    ));
    assert!(systemprompt_mcp_knowledge_bank::drive::extract::supported(
        "wav"
    ));
}

#[tokio::test]
async fn drive_listing_uses_shared_drive_flags_and_paginates_without_person_auth() {
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/files"))
        .and(header("authorization", "Bearer test-token"))
        .and(query_param("driveId", "drive"))
        .and(query_param("corpora", "drive"))
        .and(query_param("supportsAllDrives", "true"))
        .and(query_param("includeItemsFromAllDrives", "true"))
        .and(query_param("pageToken", "page-2"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"files":[{"id":"doc","name":"Brief"}]}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;
    let google = systemprompt_mcp_knowledge_bank::drive::google::GoogleDrive::with_access_token(
        server.uri(),
        "test-token".to_owned(),
    )
    .unwrap();
    let page = google.files("drive", Some("page-2")).await.unwrap();
    assert_eq!(page.files[0].id, "doc");
    assert!(page.next_page_token.is_none());
}

#[tokio::test]
async fn oversized_stream_is_rejected_even_without_content_length() {
    use wiremock::matchers::path;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    let server = MockServer::start().await;
    Mock::given(path("/files/doc"))
        .respond_with(ResponseTemplate::new(200).set_body_string("123456789"))
        .mount(&server)
        .await;
    let google = systemprompt_mcp_knowledge_bank::drive::google::GoogleDrive::with_access_token(
        server.uri(),
        "token".to_owned(),
    )
    .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let document = file("doc", "brief.txt", "drive");
    let error = google
        .download(&document, &temp.path().join("input.txt"), 4)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("limit"));
}

#[cfg(unix)]
#[tokio::test]
async fn aws_adapter_uses_persistable_invocations_and_reads_standard_output() {
    use std::os::unix::fs::PermissionsExt;
    use systemprompt_mcp_knowledge_bank::drive::aws::AwsExtraction;
    use systemprompt_mcp_knowledge_bank::drive::config::ExtractionSettings;
    let dir = tempfile::tempdir().unwrap();
    let executable = dir.path().join("aws");
    std::fs::write(&executable,r#"#!/usr/bin/env python3
import json,sys,pathlib
args=sys.argv[1:]
assert '--region' in args and args[args.index('--region')+1]=='eu-central-1'
if args[:2]==['bedrock-data-automation-runtime','invoke-data-automation-async']:
    assert '--client-token' in args
    print(json.dumps({'invocationArn':'arn:aws:bedrock:eu-central-1:123:data-automation-invocation/test'}))
elif args[:2]==['bedrock-data-automation-runtime','get-data-automation-status']:
    print(json.dumps({'status':'Success','outputConfiguration':{'s3Uri':'s3://bucket/knowledge-drive/output/test.pdf/output_metadata.json'}}))
elif args[:2]==['s3api','list-objects-v2']:
    print(json.dumps({'Contents':[{'Key':'knowledge-drive/output/test.pdf/standard_output/0/result.json'}]}))
elif args[:2]==['s3','cp'] and args[2].startswith('s3://'):
    pathlib.Path(args[3]).write_text(json.dumps({'pages':[{'page_index':1,'text':'Launch campaign evidence'}]}))
elif args[:2]!=['s3','cp']:
    sys.exit(2)
"#).unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    let aws = AwsExtraction {
        settings: ExtractionSettings {
            bucket: "bucket".to_owned(),
            project_arn: "project".to_owned(),
            profile_arn: "profile".to_owned(),
            aws_executable: executable.to_string_lossy().into_owned(),
            ..Default::default()
        },
    };
    let input = dir.path().join("input.pdf");
    std::fs::write(&input, b"mock binary").unwrap();
    let (arn, output) = aws.start(&input, "test.pdf").await.unwrap();
    assert!(arn.ends_with("/test"));
    assert_eq!(output, "s3://bucket/knowledge-drive/output/test.pdf/");
    let status = aws.status(&arn).await.unwrap();
    assert_eq!(status.status, "Success");
    let body = aws
        .result(&status.output_configuration.unwrap().s3_uri)
        .await
        .unwrap();
    assert!(body.contains("Launch campaign evidence"));
    assert!(body.contains("page_index=1"));
    assert!(
        aws.result("s3://another-bucket/knowledge-drive/output/")
            .await
            .is_err()
    );
}


#[test]
fn removal_of_a_parent_folder_removes_descendant_content() {
    let tree = HashMap::from([("sub".to_owned(), file("sub", "Briefs", "removed-project"))]);
    let document = file("doc", "brief.txt", "sub");
    let removed = std::collections::HashSet::from(["removed-project".to_owned()]);
    assert!(
        systemprompt_mcp_knowledge_bank::drive::types::has_removed_ancestor(
            &document, &tree, &removed
        )
    );
}

#[test]
fn slide_order_and_speaker_notes_follow_relationships_not_filenames() {
    use std::io::{Cursor, Write};
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let parts = [
        (
            "ppt/presentation.xml",
            r#"<p:presentation xmlns:p="p" xmlns:r="r"><p:sldIdLst><p:sldId id="1" r:id="second"/><p:sldId id="2" r:id="first"/></p:sldIdLst></p:presentation>"#,
        ),
        (
            "ppt/_rels/presentation.xml.rels",
            r#"<Relationships><Relationship Id="first" Target="slides/slide1.xml"/><Relationship Id="second" Target="slides/slide2.xml"/></Relationships>"#,
        ),
        (
            "ppt/slides/slide1.xml",
            r#"<a:p><a:r><a:t>Closing</a:t></a:r></a:p>"#,
        ),
        (
            "ppt/slides/slide2.xml",
            r#"<a:p><a:r><a:t>Opening</a:t></a:r></a:p>"#,
        ),
        (
            "ppt/slides/_rels/slide2.xml.rels",
            r#"<Relationships><Relationship Id="notes" Target="../notesSlides/notesSlide7.xml"/></Relationships>"#,
        ),
        (
            "ppt/notesSlides/notesSlide7.xml",
            r#"<a:p><a:r><a:t>Introduce the campaign</a:t></a:r></a:p>"#,
        ),
    ];
    for (name, body) in parts {
        zip.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(body.as_bytes()).unwrap();
    }
    let bytes = zip.finish().unwrap().into_inner();
    let text = office::presentation(&bytes).unwrap();
    assert!(text.contains("# Slide 1\nOpening"));
    assert!(text.contains("# Slide 2\nClosing"));
    assert!(text.contains("# Speaker notes for slide 1\nIntroduce the campaign"));
}
