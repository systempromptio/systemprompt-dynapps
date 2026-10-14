//! Restoring the demo records to their seeded state before a demo run.
//!
//! The shell seeder only creates what is missing; a demo that has already
//! run with the destructive grant has deleted the dedup duplicates, archived
//! some and anonymised the GDPR requester. So every record here is upserted:
//! found by its demo key (archived rows included), rewritten to its seeded
//! values and reactivated, or created when a previous run deleted it. Posted
//! invoices are only ever created, never rewritten — legal retention.

use serde_json::{Value, json};
use systemprompt_web_admin_core::error::AdminResult;

use super::plan::{
    DEDUP_LEADS, DEDUP_PARTNERS, DEMO_TAGS, DemoLead, DemoPartner, GDPR_DRAFT_REF, GDPR_LEAD_NAME,
    GDPR_MESSAGES, GDPR_POSTED_REF, GDPR_REQUESTER, TAG_GDPR, TRIAGE_LEADS,
};
use super::rpc::OdooSession;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct SeedSummary {
    pub created: u32,
    pub restored: u32,
    pub gdpr_partner_id: i64,
    pub posted_invoice_id: i64,
}

#[derive(Debug, Clone, Copy)]
struct Tags {
    crm: [i64; 3],
    partner: [i64; 3],
}

impl Tags {
    fn index(tag: &str) -> usize {
        DEMO_TAGS.iter().position(|t| *t == tag).unwrap_or(0)
    }

    fn crm_of(&self, tag: &str) -> i64 {
        self.crm[Self::index(tag)]
    }

    fn partner_of(&self, tag: &str) -> i64 {
        self.partner[Self::index(tag)]
    }
}

pub(super) async fn reseed(odoo: &OdooSession) -> AdminResult<SeedSummary> {
    let mut summary = SeedSummary::default();
    let tags = resolve_tags(odoo).await?;
    for lead in TRIAGE_LEADS.iter().chain(DEDUP_LEADS.iter()) {
        upsert_lead(odoo, &tags, lead, &mut summary).await?;
    }
    for partner in &DEDUP_PARTNERS {
        let domain = json!([
            ["name", "=", partner.name],
            ["email", "=", partner.email],
            ["category_id", "in", [tags.partner_of(partner.tag)]],
        ]);
        upsert(
            odoo,
            "res.partner",
            domain,
            partner_values(&tags, partner),
            &mut summary,
        )
        .await?;
    }
    seed_gdpr(odoo, &tags, &mut summary).await?;
    tracing::info!(target: "odoo_demo", created = summary.created, restored = summary.restored,
        "Odoo demo records re-seeded");
    Ok(summary)
}

async fn resolve_tags(odoo: &OdooSession) -> AdminResult<Tags> {
    let mut tags = Tags {
        crm: [0; 3],
        partner: [0; 3],
    };
    for (i, name) in DEMO_TAGS.iter().enumerate() {
        tags.crm[i] = find_or_create(odoo, "crm.tag", name).await?;
        tags.partner[i] = find_or_create(odoo, "res.partner.category", name).await?;
    }
    Ok(tags)
}

async fn find_or_create(odoo: &OdooSession, model: &str, name: &str) -> AdminResult<i64> {
    match odoo.search_one(model, json!([["name", "=", name]])).await? {
        Some(id) => Ok(id),
        None => odoo.create(model, json!({"name": name})).await,
    }
}

async fn upsert(
    odoo: &OdooSession,
    model: &str,
    domain: Value,
    values: Value,
    summary: &mut SeedSummary,
) -> AdminResult<i64> {
    if let Some(id) = odoo.search_one(model, domain).await? {
        odoo.write(model, &[id], values).await?;
        summary.restored += 1;
        return Ok(id);
    }
    summary.created += 1;
    odoo.create(model, values).await
}

async fn upsert_lead(
    odoo: &OdooSession,
    tags: &Tags,
    lead: &DemoLead,
    summary: &mut SeedSummary,
) -> AdminResult<i64> {
    let tag = tags.crm_of(lead.tag);
    let domain = json!([["name", "=", lead.name], ["tag_ids", "in", [tag]]]);
    upsert(odoo, "crm.lead", domain, lead_values(tag, lead), summary).await
}

fn lead_values(tag: i64, lead: &DemoLead) -> Value {
    json!({
        "name": lead.name, "type": "lead", "active": true,
        "partner_name": lead.company, "contact_name": lead.contact,
        "email_from": lead.email, "phone": lead.phone, "city": lead.city,
        "description": lead.description, "tag_ids": [[6, 0, [tag]]],
    })
}

fn partner_values(tags: &Tags, partner: &DemoPartner) -> Value {
    json!({
        "name": partner.name, "email": partner.email, "phone": partner.phone,
        "city": partner.city, "is_company": partner.is_company, "active": true,
        "category_id": [[6, 0, [tags.partner_of(partner.tag)]]],
    })
}

async fn seed_gdpr(odoo: &OdooSession, tags: &Tags, summary: &mut SeedSummary) -> AdminResult<()> {
    let category = tags.partner_of(TAG_GDPR);
    let values = partner_values(tags, &GDPR_REQUESTER);
    let partner = upsert(
        odoo,
        "res.partner",
        json!([["category_id", "in", [category]]]),
        values,
        summary,
    )
    .await?;
    let strays: Vec<i64> = odoo
        .search_all(
            "res.partner",
            json!([["category_id", "in", [category]], ["id", "!=", partner]]),
        )
        .await?;
    odoo.write("res.partner", &strays, json!({"active": false}))
        .await?;
    let lead_domain = json!([
        ["name", "=", GDPR_LEAD_NAME],
        ["tag_ids", "in", [tags.crm_of(TAG_GDPR)]]
    ]);
    let lead = json!({
        "name": GDPR_LEAD_NAME, "type": "opportunity", "active": true, "partner_id": partner,
        "email_from": GDPR_REQUESTER.email, "phone": GDPR_REQUESTER.phone,
        "tag_ids": [[6, 0, [tags.crm_of(TAG_GDPR)]]],
    });
    upsert(odoo, "crm.lead", lead_domain, lead, summary).await?;
    let comments = json!([
        ["model", "=", "res.partner"],
        ["res_id", "=", partner],
        ["message_type", "=", "comment"]
    ]);
    if odoo.count("mail.message", comments).await? < 2 {
        for body in GDPR_MESSAGES {
            odoo.kw(
                "res.partner",
                "message_post",
                json!([[partner]]),
                json!({"body": body, "message_type": "comment"}),
            )
            .await?;
        }
    }
    invoice(odoo, partner, GDPR_DRAFT_REF, false, summary).await?;
    summary.posted_invoice_id = invoice(odoo, partner, GDPR_POSTED_REF, true, summary).await?;
    summary.gdpr_partner_id = partner;
    Ok(())
}

async fn invoice(
    odoo: &OdooSession,
    partner: i64,
    reference: &str,
    post: bool,
    summary: &mut SeedSummary,
) -> AdminResult<i64> {
    let domain = json!([["ref", "=", reference], ["partner_id", "=", partner]]);
    if let Some(id) = odoo.search_one("account.move", domain).await? {
        return Ok(id);
    }
    let line = json!({"name": "demo consulting day", "quantity": 1, "price_unit": 850});
    let id = odoo
        .create(
            "account.move",
            json!({
                "move_type": "out_invoice", "partner_id": partner, "ref": reference,
                "invoice_line_ids": [[0, 0, line]],
            }),
        )
        .await?;
    if post {
        odoo.kw("account.move", "action_post", json!([[id]]), json!({}))
            .await?;
    }
    summary.created += 1;
    Ok(id)
}
