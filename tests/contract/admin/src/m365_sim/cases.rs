//! The simulation's cases, in the order a person meets them: connect, read
//! mail, upload and attach, save a download as a document; then the crossings
//! between the two companies and the audit trail they leave.

use reqwest::StatusCode;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use systemprompt::identifiers::UserId;

use super::connect::{self, is_connected};
use super::stack::{DOCUMENTS, M365, Stack};
use super::{case, find, text_of};
use crate::docs_smoke::mcp::McpClient;

const OFFER_BYTES: usize = 100_000;
// The part-number placeholder of core's signed `parts_url` template.
const PART_NUMBER: &str = "{n}";
const ANA: &str = "ana@empresa-a.example";
const BRUNO: &str = "bruno@empresa-b.example";

pub(super) struct Run<'a> {
    stack: &'a Stack,
    pool: &'a sqlx::PgPool,
    ana: UserId,
    bruno: UserId,
    ana_web: String,
    bruno_web: String,
    ana_m365: McpClient,
    bruno_m365: McpClient,
    ana_docs: McpClient,
    tenant_a: String,
    tenant_b: String,
    upload_id: String,
    blob_ref: String,
    document_id: String,
}

// A PDF header, so the upload passes the store's content sniffing, then
// filler up to the size the case names.
fn offer() -> Vec<u8> {
    let mut bytes = b"%PDF-1.4\n%oferta de la empresa A\n".to_vec();
    bytes.resize(OFFER_BYTES, b'0');
    bytes
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        write!(out, "{b:02x}").expect("writing to a String cannot fail");
    }
    out
}

impl<'a> Run<'a> {
    pub(super) async fn new(
        stack: &'a Stack,
        pool: &'a sqlx::PgPool,
        ana: UserId,
        bruno: UserId,
    ) -> Self {
        let ana_caller = stack.sign_in(pool, &ana, M365).await;
        let bruno_caller = stack.sign_in(pool, &bruno, M365).await;
        let ana_docs = stack.sign_in(pool, &ana, DOCUMENTS).await;
        let tenant = |key: &str| {
            stack.sim.tenants[key]
                .as_str()
                .expect("a tenant")
                .to_owned()
        };
        Self {
            stack,
            pool,
            ana_web: ana_caller.bearer.clone(),
            bruno_web: bruno_caller.bearer.clone(),
            ana_m365: McpClient::connect(&stack.mcp_url(M365), ana_caller).await,
            bruno_m365: McpClient::connect(&stack.mcp_url(M365), bruno_caller).await,
            ana_docs: McpClient::connect(&stack.mcp_url(DOCUMENTS), ana_docs).await,
            ana,
            bruno,
            tenant_a: tenant("a"),
            tenant_b: tenant("b"),
            upload_id: String::new(),
            blob_ref: String::new(),
            document_id: String::new(),
        }
    }

    async fn sim(&self, path: &str) -> Value {
        let origin = self.stack.sim.graph.trim_end_matches("/v1.0");
        reqwest::get(format!("{origin}{path}"))
            .await
            .expect("read the simulator")
            .json()
            .await
            .expect("the simulator answers JSON")
    }

    pub(super) async fn connect_both(&self, n: &mut u32) {
        let back = connect::consent(self.stack, &self.ana_web, "carla@empresa-c.example").await;
        connect::callback(&self.ana_web, &back).await;
        let ana_row = connect::linked(self.pool, &self.ana).await;
        assert!(
            ana_row.as_ref().is_none_or(|row| row.status != "connected"),
            "a tenant outside microsoft_365_allowed_tenants was linked: {ana_row:?}"
        );
        case(
            n,
            "cruce: Ana inicia sesión en Entra con una cuenta de un tenant que la instalación no admite (empresa C): no se vincula",
        );

        let back = connect::consent(self.stack, &self.ana_web, "ana@empresa-a.example").await;
        connect::callback(&self.ana_web, &back).await;
        let ana_row = connect::linked(self.pool, &self.ana).await;
        assert!(
            is_connected(ana_row.as_ref(), &self.tenant_a),
            "Ana: {ana_row:?}"
        );
        assert_eq!(
            ana_row.and_then(|row| row.account).as_deref(),
            Some("ana@empresa-a.example")
        );
        case(
            n,
            "conectar: Ana (empresa A) vincula su cuenta de Microsoft 365 en el tenant A (Entra simulado, PKCE)",
        );

        let bruno_back =
            connect::consent(self.stack, &self.bruno_web, "bruno@empresa-b.example").await;
        let stolen = connect::callback(&self.ana_web, &bruno_back).await;
        assert!(
            stolen.is_client_error(),
            "Ana redeemed Bruno's consent: {stolen}"
        );
        let ana_row = connect::linked(self.pool, &self.ana).await;
        assert!(
            is_connected(ana_row.as_ref(), &self.tenant_a),
            "Ana after: {ana_row:?}"
        );
        let bruno_row = connect::linked(self.pool, &self.bruno).await;
        assert!(
            !is_connected(bruno_row.as_ref(), &self.tenant_b),
            "Bruno: {bruno_row:?}"
        );
        case(
            n,
            "cruce: Ana no puede completar el consentimiento de Bruno ni vincular así el tenant B (401, Ana sigue en A)",
        );

        let own = connect::callback(&self.bruno_web, &bruno_back).await;
        assert!(
            own.is_redirection() || own == StatusCode::OK,
            "Bruno's callback: {own}"
        );
        let bruno_row = connect::linked(self.pool, &self.bruno).await;
        assert!(
            is_connected(bruno_row.as_ref(), &self.tenant_b),
            "Bruno: {bruno_row:?}"
        );
        case(
            n,
            "conectar: Bruno (empresa B) vincula su cuenta en el tenant B con su propio consentimiento",
        );
    }

    pub(super) async fn read_mail(&mut self, n: &mut u32) {
        let ana = self
            .ana_m365
            .ok("d365_list_mail", json!({}))
            .await
            .payload
            .to_string();
        assert!(
            ana.contains("Pedido de la empresa A") && !ana.contains("empresa B"),
            "{ana}"
        );
        let log = self.sim("/_sim/log").await;
        let entries = log.as_array().expect("the simulator's log");
        assert!(!entries.is_empty(), "Graph was not called");
        assert!(
            entries
                .iter()
                .all(|entry| entry["account"] == json!(ANA)
                    && entry["tenant"] == json!(self.tenant_a)),
            "a Graph call for Ana carried another account's token: {log}"
        );
        case(
            n,
            "leer correo: Ana lista su buzón por el MCP; cada llamada a Graph lleva su token (cuenta y tenant A) y solo ve correo de A",
        );

        let bruno = self
            .bruno_m365
            .ok("d365_list_mail", json!({}))
            .await
            .payload
            .to_string();
        assert!(
            bruno.contains("Pedido de la empresa B") && !bruno.contains("empresa A"),
            "{bruno}"
        );
        let log = self.sim("/_sim/log").await;
        let last = log
            .as_array()
            .and_then(|entries| entries.last())
            .expect("Graph was called");
        assert_eq!(last["account"], json!(BRUNO), "{last}");
        case(
            n,
            "leer correo: Bruno lista su buzón con su propio token y solo ve correo del tenant B",
        );

        let foreign = self
            .ana_m365
            .outcome("d365_get_mail", json!({"message_id": "msg-b-1"}))
            .await
            .err()
            .expect("Ana read tenant B's message")
            .to_string();
        assert!(
            foreign.contains("404"),
            "refused for another reason: {foreign}"
        );
        let log = self.sim("/_sim/log").await;
        let asked = log
            .as_array()
            .and_then(|entries| {
                entries
                    .iter()
                    .rfind(|entry| entry["path"] == json!("/v1.0/me/messages/msg-b-1"))
            })
            .expect("the request reached Graph");
        assert_eq!(asked["account"], json!(ANA), "{asked}");
        assert_eq!(asked["tenant"], json!(self.tenant_a), "{asked}");
        case(
            n,
            "cruce: Ana pide por id un mensaje del tenant B; la petición llega a Graph con su token de A y Graph responde 404",
        );
    }

    pub(super) async fn upload_and_attach(&mut self, n: &mut u32) {
        let bytes = offer();
        let ticket = self
            .ana_docs
            .ok(
                "documents_upload",
                json!({"name": "oferta-a.pdf", "mime": "application/pdf",
                       "size": OFFER_BYTES, "sha256": hex(&Sha256::digest(&bytes))}),
            )
            .await
            .payload;
        self.upload_id = text_of(&ticket, "upload_id");
        let header = text_of(&ticket, "token_header");
        let token = text_of(&ticket, "upload_token");
        let http = reqwest::Client::new();
        let part = http
            .put(text_of(&ticket, "parts_url").replace(PART_NUMBER, "1"))
            .header(header.as_str(), token.as_str())
            .body(bytes.clone())
            .send()
            .await
            .expect("upload the part");
        assert!(part.status().is_success(), "part upload: {}", part.status());
        let done = http
            .post(text_of(&ticket, "complete_url"))
            .header(header.as_str(), token.as_str())
            .send()
            .await
            .expect("complete the upload");
        assert!(done.status().is_success(), "complete: {}", done.status());
        let done: Value = done.json().await.expect("the completion answers JSON");
        self.blob_ref = text_of(&done, "blob_ref");
        case(
            n,
            "pedir subida: Ana pide la subida (documents_upload), sube 100 KB al enlace firmado y la completa",
        );

        let draft = self
            .ana_m365
            .ok("d365_create_draft", json!({"subject": "Oferta", "body": "Adjunta.", "to": ["cliente@empresa-a.example"]}))
            .await
            .payload;
        let draft = text_of(&draft, "id");
        let attached = self
            .ana_m365
            .ok("d365_add_mail_attachment", json!({"message_id": draft, "filename": "oferta-a.pdf", "file": {"upload_id": self.upload_id}}))
            .await
            .payload;
        assert_eq!(
            find(&attached, "size").and_then(Value::as_u64),
            Some(OFFER_BYTES as u64),
            "{attached}"
        );
        case(
            n,
            "adjuntar: la subida de Ana llega como adjunto de 100 KB a su borrador en Graph (tenant A), sin base64 por el modelo",
        );

        self.ana_m365
            .ok("d365_send_draft", json!({"message_id": draft}))
            .await;
        self.ana_m365
            .ok(
                "d365_update_mail",
                json!({"message_id": "msg-a-1", "is_read": true}),
            )
            .await;
        self.ana_m365
            .ok("d365_upload_file", json!({"folder_path": "Ofertas", "filename": "oferta-a.pdf", "file": {"blob_ref": self.blob_ref}}))
            .await;
        let state = self.sim("/_sim/state").await;
        let box_a = &state[&self.tenant_a];
        assert_eq!(box_a[&draft]["isDraft"], json!(false), "{box_a}");
        assert_eq!(box_a["msg-a-1"]["isRead"], json!(true), "{box_a}");
        let drive = &state["drives"][&self.tenant_a][0];
        assert_eq!(
            drive["sha256"].as_str(),
            Some(hex(&Sha256::digest(&bytes)).as_str()),
            "{state}"
        );
        assert_eq!(state["drives"][&self.tenant_b], json!([]), "{state}");
        case(
            n,
            "escrituras: enviar el borrador (POST), marcar leído (PATCH) y subir a OneDrive (PUT) quedan en el tenant A",
        );
    }

    pub(super) async fn download_to_document(&mut self, n: &mut u32) {
        let saved = self
            .ana_m365
            .ok(
                "d365_get_mail_attachment",
                json!({"message_id": "msg-a-1", "attachment_id": "att-a-1"}),
            )
            .await
            .payload;
        self.document_id = text_of(&saved, "document_id");
        assert!(
            find(&saved, "download_url").is_some(),
            "no signed link: {saved}"
        );
        assert!(find(&saved, "contentBytes").is_none(), "{saved}");
        case(
            n,
            "descargar: el adjunto de A se guarda como documento de Ana con enlace firmado (toolset documents), sin bytes al modelo",
        );

        let draft = self
            .ana_m365
            .ok("d365_create_draft", json!({"subject": "Reenvío", "body": "Factura.", "to": ["cliente@empresa-a.example"]}))
            .await
            .payload;
        let draft = text_of(&draft, "id");
        self.ana_m365
            .ok("d365_add_mail_attachment", json!({"message_id": draft, "filename": "factura-a.pdf", "file": {"document_id": self.document_id}}))
            .await;
        case(
            n,
            "el documento guardado vuelve a servir de adjunto para su dueña (document_id)",
        );
    }

    pub(super) async fn references_stay_with_their_owner(&mut self, n: &mut u32) {
        let draft = self
            .bruno_m365
            .ok(
                "d365_create_draft",
                json!({"subject": "B", "body": "B", "to": ["cliente@empresa-b.example"]}),
            )
            .await
            .payload;
        let draft = text_of(&draft, "id");
        for (kind, value) in [
            ("upload_id", self.upload_id.clone()),
            ("blob_ref", self.blob_ref.clone()),
            ("document_id", self.document_id.clone()),
        ] {
            let refused = self
                .bruno_m365
                .outcome(
                    "d365_add_mail_attachment",
                    json!({"message_id": draft, "filename": "x.pdf", "file": {kind: value}}),
                )
                .await;
            assert!(refused.is_err(), "Bruno resolved Ana's {kind}");
        }
        let refused = self
            .bruno_m365
            .outcome("d365_upload_file", json!({"folder_path": "Robado", "filename": "x.pdf", "file": {"blob_ref": self.blob_ref}}))
            .await;
        assert!(refused.is_err(), "Bruno uploaded Ana's blob");
        let state = self.sim("/_sim/state").await;
        assert_eq!(
            state[&self.tenant_b][&draft]["attachments"],
            json!([]),
            "{state}"
        );
        assert_eq!(state["drives"][&self.tenant_b], json!([]), "{state}");
        case(
            n,
            "cruce: ninguna referencia de Ana (upload_id, blob_ref, document_id) se resuelve para Bruno; nada llega al tenant B",
        );
    }

    pub(super) async fn audit(&self, n: &mut u32) {
        let rows = |user: &UserId, action: &str| {
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM user_activity
                 WHERE category = 'mcp_access' AND entity_type = 'file' AND user_id = $1 AND action = $2",
            )
            .bind(user.as_str().to_owned())
            .bind(action.to_owned())
            .fetch_one(self.pool)
        };
        let used = rows(&self.ana, "used").await.expect("count Ana's uses");
        let imported = rows(&self.ana, "imported")
            .await
            .expect("count Ana's imports");
        let rejected = rows(&self.bruno, "rejected")
            .await
            .expect("count Bruno's refusals");
        assert!(
            used >= 3 && imported >= 1 && rejected >= 4,
            "used {used}, imported {imported}, rejected {rejected}"
        );
        case(
            n,
            "auditoría: cada uso, descarga y rechazo de fichero queda en user_activity a nombre de quien lo hizo",
        );
    }
}
