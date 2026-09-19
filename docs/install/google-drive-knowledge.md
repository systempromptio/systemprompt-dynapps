# Google Drive → Astound knowledge bank

The `systemprompt_marketing` source reads the shared drive selected in configuration.
It is shipped disabled until Google and AWS access is configured. All implementation
lives in Astound; the existing chunker, Bedrock embeddings/reranking, pgvector index,
and governed MCP executor are reused.

## Service identities

1. Create a Google Cloud service account and enable the Google Drive API in its
   project. Ask the shared-drive Manager to add the service-account email as a
   **Viewer**. The personal Contributor invitation does not grant the service account
   access. Workspace sharing policy must permit this membership.
2. Store the service-account JSON in the profile secret named
   `google_drive_service_account`. Keep private keys out of YAML and Git. The client
   signs its own OAuth assertions with `drive.readonly`; there is no personal refresh
   token and an API key is insufficient.
3. Give the knowledge-bank process an AWS IAM role, workload identity, or standard AWS
   credential profile for S3 and Bedrock Data Automation. Its existing `bedrock_api_key`
   continues to authenticate embeddings/reranking but cannot authenticate BDA.
4. Create a private S3 bucket in `eu-central-1`, with encryption, public access blocked,
   and a lifecycle policy expiring `knowledge-drive/` objects after 30 days. Give the
   runtime identity `s3:ListBucket` on that bucket and `s3:GetObject`/`s3:PutObject` on
   its `knowledge-drive/*` objects. Add KMS permissions when using a customer KMS key.
5. Create a BDA project in Frankfurt, enabling document page/element text and tables,
   image text detection and image summaries, audio transcription with speaker labels,
   and video transcription/text detection/scene summaries. Choose JSON standard output.
   The runtime identity needs `bedrock:InvokeDataAutomationAsync` and
   `bedrock:GetDataAutomationStatus`, with access to the configured project, invocation
   resources and EU data-automation profile. Permit the EU destinations required by
   that profile; processing is EU-wide, not Frankfurt-only.

The container includes AWS CLI **2.36.18**, LibreOffice and fonts. On a native
installation, install the same tools and make `aws` and `libreoffice` available to
the MCP process. AWS CLI handles signing, role/session refresh and request retries;
Rust launches it directly without a shell. No AWS secret is included in arguments.

Reference: [Google service credentials](https://developers.google.com/workspace/guides/create-credentials),
[AWS BDA setup](https://docs.aws.amazon.com/bedrock/latest/userguide/bda-using-api.html),
[API-key limitations](https://docs.aws.amazon.com/bedrock/latest/userguide/api-keys-supported.html),
[EU routing](https://docs.aws.amazon.com/bedrock/latest/userguide/bda-cris.html).

## Configuration and first import

In `services/knowledge-bank/config.yaml`, set `google_workspace.enabled: true` and fill
`extraction.bucket`, `extraction.project_arn` and `extraction.profile_arn`:

```yaml
google_workspace:
  enabled: true
  sources:
    systemprompt_marketing:
      drive_id: YOUR_SHARED_DRIVE_ID
      credential_secret: google_drive_service_account
      projects_folder: Projects
      shared_folder: Shared
  extraction:
    region: eu-central-1
    bucket: YOUR_PRIVATE_BUCKET
    project_arn: arn:aws:bedrock:eu-central-1:ACCOUNT:data-automation-project/PROJECT
    profile_arn: arn:aws:bedrock:eu-central-1:ACCOUNT:data-automation-profile/eu.data-automation-v1
    version: drive-extraction-v1
```

Use `model.provider: bedrock` for production and preserve your configured embedding
model and dimensions. The development configuration defaults to a local model;
BDA extraction does not automatically change the retrieval model.

The knowledge-bank binary applies its additive `google_workspace.sql` schema to the
knowledge database at boot. The web scheduler opens the same database using
`knowledge_database_url`, falling back to the instance database. Restart the rebuilt
MCP and application to load configuration and register the scheduler job.

Request the initial import through MCP:

```json
{"source_types":["systemprompt_marketing"],"force_resync":true}
```

Tool: `sync_knowledge`. Requests return immediately. The worker checks the durable
queue every 30 seconds and processes up to 20 due files per pass. The daily
`knowledge_drive_sync` scheduler job runs at **03:00 UTC** under its configured
`owner: admin`; change the owner to the real administrator authorizing the job.

## Project organization

A Drive Content manager or Manager organizes the source material:

```text
Systemprompt Marketing/
  Projects/
    Product Launch/
      Briefs/
      Research/
    Website Refresh/
  Shared/
    Brand/
    Templates/
```

The sync service stays read-only. Folder IDs identify projects; renames update the
project display name, and moves update all descendants' assignments. Files outside
this layout appear under **Unassigned**. Shared files are included in project
searches by default. Project permissions are open to every signed-in `user` role;
existing Atlassian sources retain their personal grants.

Call `knowledge_drive_inventory` to prepare the manager's file-to-project mapping:

```json
{"source_type":"systemprompt_marketing","limit":100,"after":""}
```

Continue with the last `file_id` as `after` until the page is empty. Results contain
Drive URLs, filenames, MIME types, current paths, project IDs and processing errors.
No files are physically moved by this integration.

Call `search_knowledge` with:

```json
{"query":"What is the launch message?","source_types":["systemprompt_marketing"],"project_ids":["PROJECT_FOLDER_ID"],"include_shared":true}
```

Project constraints are applied inside both retrieval queries before ranking.
Omitting project IDs searches all projects.

## Monitoring, resync and recovery

`knowledge_sync_status` includes shared `drive_sources` grouped by project/status,
with request/completion counters and the last completed **discovery** time. Discovery
completion is not extraction completion: inspect the file states too.

- `pending` / `processing`: queued work or an asynchronous BDA job.
- `indexed`: extracted content has reached the index.
- `failed`: retryable processing failure; retried after 15 minutes, up to five failures per sync cycle.
- `unsupported`: conversion or operator action is required; never treated as indexed.
- `folder` / `shortcut`: structural inventory entries, not duplicate documents.
- `removed`: content was removed from search following confirmed upstream removal.

Force resync repeats a complete inventory and retries failures, retaining valid
extraction and embedding caches. `rebuild: true` additionally re-chunks documents;
unchanged embeddings are still cached. Increment `extraction.version` after changing
BDA output settings or conversion behavior to invalidate extracted outputs.

A complete inventory runs initially, on force resync, and at least weekly. Daily
runs otherwise use the Drive changes feed. Cursors advance only after discovered
work is durable. Incomplete listings and authentication errors cannot prune the
index. Changes and duplicate requests are replay-safe. A PostgreSQL advisory lock
allows one worker per drive across application replicas.

Original Drive links remain the citations. Extracted text contains sheet/cell,
slide, page and timestamp context; BDA summaries are labelled generated descriptions.
DOCX conversion page numbers refer to the rendered document, not a stable Word page.

## Coverage and acceptance

Docs export as DOCX, Sheets as XLSX, Slides as PPTX, and Drawings as PNG. Office
sheets/slides are parsed for cell values, formulas and notes, then rendered for visual
extraction. PDF/DOCX/images/audio/video use BDA. Plain text, Markdown, CSV, HTML,
JSON and XML are extracted directly. Accessible Drive comments/replies are included.

BDA has format, language, size and duration limits. Proprietary design files,
encrypted documents, unsupported Google-native files and files exceeding limits
remain explicit exceptions. A complete inventory is not a promise that every
arbitrary binary can be decoded. Verify representative native documents, a scanned
PDF, an image, audio and video; compare inventory totals, resolve exceptions, then
validate project-scoped answers against original files before declaring full coverage.
