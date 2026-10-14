//! The demo records, as data: what `demo/agent-farm/odoo-seed.sh` creates,
//! restated so the console can restore it without a shell.
//!
//! Every record is tagged with one of [`DEMO_TAGS`]; the seeder resolves the
//! tag name to its id and never touches a record outside those tags. The
//! plan is pure so the invariant can be checked without an Odoo.

pub const TAG_TRIAGE: &str = "demo-triage";
pub const TAG_DEDUP: &str = "demo-dedup";
pub const TAG_GDPR: &str = "demo-gdpr";
pub const DEMO_TAGS: [&str; 3] = [TAG_TRIAGE, TAG_DEDUP, TAG_GDPR];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DemoLead {
    pub tag: &'static str,
    pub name: &'static str,
    pub company: &'static str,
    pub contact: &'static str,
    pub email: &'static str,
    pub phone: &'static str,
    pub city: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DemoPartner {
    pub tag: &'static str,
    pub name: &'static str,
    pub email: &'static str,
    pub phone: &'static str,
    pub city: &'static str,
    pub is_company: bool,
}

const fn lead(
    tag: &'static str,
    name: &'static str,
    (company, contact): (&'static str, &'static str),
    (email, phone, city): (&'static str, &'static str, &'static str),
    description: &'static str,
) -> DemoLead {
    DemoLead {
        tag,
        name,
        company,
        contact,
        email,
        phone,
        city,
        description,
    }
}

const fn partner(
    name: &'static str,
    email: &'static str,
    phone: &'static str,
    city: &'static str,
) -> DemoPartner {
    DemoPartner {
        tag: TAG_DEDUP,
        name,
        email,
        phone,
        city,
        is_company: true,
    }
}

pub const TRIAGE_LEADS: [DemoLead; 5] = [
    lead(
        TAG_TRIAGE,
        "Odoo rollout for 40 users",
        ("Brouwerij De Kroon BV", "Pieter Janssens"),
        ("pieter.janssens@example.com", "+32 3 555 0101", "Antwerp"),
        "Craft brewery, 3 sites. Wants inventory + MRP, budget approved for Q1, decision maker on the call.",
    ),
    lead(
        TAG_TRIAGE,
        "Migrate from Exact Online",
        ("Logistiek Vandamme NV", "Sofie Vandamme"),
        ("sofie@example.com", "+32 9 555 0102", "Ghent"),
        "Freight forwarder, 25 staff. Unhappy with current ERP, asked for pricing only, no timeline.",
    ),
    lead(
        TAG_TRIAGE,
        "E-commerce + POS integration",
        ("Bodega Ruiz SL", "Javier Ruiz"),
        ("javier.ruiz@example.com", "+34 91 555 0103", "Madrid"),
        "Wine retailer, 2 shops + webshop. Demo booked next week, wants go-live before summer.",
    ),
    lead(
        TAG_TRIAGE,
        "Newsletter signup",
        ("Talleres Ortega SA", "Lucia Ortega"),
        ("lucia.ortega@example.com", "+34 96 555 0104", "Valencia"),
        "Downloaded a whitepaper. No reply to two follow-ups.",
    ),
    lead(
        TAG_TRIAGE,
        "Field service module question",
        ("Installaties Peeters BVBA", "Koen Peeters"),
        ("koen@example.com", "+32 16 555 0105", "Leuven"),
        "HVAC installer, 60 technicians. Asked about field service and helpdesk, evaluating two vendors.",
    ),
];

pub const DEDUP_PARTNERS: [DemoPartner; 6] = [
    partner(
        "Brouwerij De Kroon BV",
        "info@dekroon.example.com",
        "+32 3 555 0201",
        "Antwerp",
    ),
    partner(
        "Brouwerij de Kroon",
        "info@dekroon.example.com",
        "+32 3 555 0201",
        "Antwerpen",
    ),
    partner(
        "Bodega Ruiz SL",
        "hola@bodegaruiz.example.com",
        "+34 91 555 0202",
        "Madrid",
    ),
    partner(
        "Bodega Ruiz S.L.",
        "hola@bodegaruiz.example.com",
        "+34 915550202",
        "Madrid",
    ),
    partner(
        "Marta Gil",
        "marta.gil@example.com",
        "+34 93 555 0203",
        "Barcelona",
    ),
    partner(
        "Gil, Marta",
        "marta.gil@example.com",
        "+34 93 555 0203",
        "Barcelona",
    ),
];

const KROON_DEDUP: &str = "Wants stock management across three sites.";

pub const DEDUP_LEADS: [DemoLead; 2] = [
    lead(
        TAG_DEDUP,
        "Inventory module for Kroon",
        ("Brouwerij De Kroon BV", "Pieter Janssens"),
        ("info@dekroon.example.com", "+32 3 555 0201", "Antwerp"),
        KROON_DEDUP,
    ),
    lead(
        TAG_DEDUP,
        "Inventory module for Kroon (web form)",
        ("Brouwerij De Kroon BV", "Pieter Janssens"),
        ("info@dekroon.example.com", "+32 3 555 0201", "Antwerp"),
        KROON_DEDUP,
    ),
];

pub const GDPR_REQUESTER: DemoPartner = DemoPartner {
    tag: TAG_GDPR,
    name: "demo GDPR requester",
    email: "gdpr.requester@example.com",
    phone: "+32 2 555 0301",
    city: "Brussels",
    is_company: false,
};

pub const GDPR_LEAD_NAME: &str = "demo GDPR requester - consulting enquiry";
pub const GDPR_DRAFT_REF: &str = "demo-gdpr-draft";
pub const GDPR_POSTED_REF: &str = "demo-gdpr-posted";

pub const GDPR_MESSAGES: [&str; 2] = [
    "Customer called asking about their invoices and contract renewal.",
    "Customer emailed: please erase all my personal data under GDPR Art. 17.",
];
