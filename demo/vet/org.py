"""The fictional veterinary group: entities, groups, projects, people.

Everything here is invented. Names are generic European first/last names,
the domain is `vetgroup-demo.example` (RFC 2606 reserved), and no row is
derived from any real tenant. `generate.py` reads this module.
"""

DOMAIN = "vetgroup-demo.example"

# (group id, display name, AD group, entity, created) — one Entra group per
# entity role, mapped 1:1 to a console group.
GROUPS = [
    ("vg-clinics-be-finance", "Clinics BE – Finance", "VG-Clinics-BE-Finance", "Clinics BE", "2026-09-01"),
    ("vg-clinics-be-frontdesk", "Clinics BE – Front desk", "VG-Clinics-BE-FrontDesk", "Clinics BE", "2026-09-01"),
    ("vg-clinics-nl-finance", "Clinics NL – Finance", "VG-Clinics-NL-Finance", "Clinics NL", "2026-09-01"),
    ("vg-clinics-nl-frontdesk", "Clinics NL – Front desk", "VG-Clinics-NL-FrontDesk", "Clinics NL", "2026-09-01"),
    ("vg-labs-diagnostics", "Labs – Diagnostics", "VG-Labs-Diagnostics", "Labs", "2026-09-01"),
    ("vg-ambulatory-field", "Ambulatory – Field vets", "VG-Ambulatory-FieldVets", "Ambulatory", "2026-09-01"),
    ("vg-group-finance", "Group Finance", "VG-Group-Finance", "Group", "2026-09-01"),
    ("vg-data-solutions", "Data & Solutions", "VG-Data-Solutions", "Group", "2026-09-01"),
    ("vg-clinics-de-acquired", "Clinics DE – Nordheide (acquired Oct 2026)", "VG-Clinics-DE-Nordheide", "Clinics DE (acquired)", "2026-10-05"),
]

GROUP_DESCRIPTIONS = {
    "vg-clinics-be-finance": "Finance team of the Belgian clinic entity. Entra group VG-Clinics-BE-Finance; one Odoo company in the shared database.",
    "vg-clinics-be-frontdesk": "Reception and scheduling staff of the Belgian clinics.",
    "vg-clinics-nl-finance": "Finance team of the Dutch clinic entity.",
    "vg-clinics-nl-frontdesk": "Reception and scheduling staff of the Dutch clinics.",
    "vg-labs-diagnostics": "Diagnostic laboratory staff: sample intake, results, reporting to clinics.",
    "vg-ambulatory-field": "Ambulatory and farm-animal field vets working from the road.",
    "vg-group-finance": "Group-level finance: consolidation, credit control, charge-back.",
    "vg-data-solutions": "Group IT and data team. Owns Odoo, Entra ID and the AI platform.",
    "vg-clinics-de-acquired": "Clinic group acquired in October 2026. Entra group federated on 2026-10-05; onboarding to the shared Odoo in progress.",
}

# (project id, name, cost centre, practice, description)
PROJECTS = [
    ("vg-dunning", "Dunning", "CC-FIN-210", "Finance", "Overdue client invoices: reminder drafting and follow-up activities in Odoo."),
    ("vg-ticket-followup", "Ticket follow-up", "CC-OPS-120", "Operations", "Helpdesk tickets from clinics: triage, follow-up and closure notes."),
    ("vg-lab-results", "Lab results summaries", "CC-LAB-310", "Diagnostics", "Plain-language summaries of lab panels for the referring vet."),
    ("vg-no-shows", "Appointment no-shows", "CC-OPS-130", "Operations", "No-show detection and rebooking outreach across clinics."),
    ("vg-invoice-recon", "Odoo invoice reconciliation", "CC-FIN-220", "Finance", "Bank-statement to invoice matching across entities in the single Odoo database."),
    ("vg-metrics-pilot", "Metrics pilot", "CC-DAT-400", "Data", "Pilot of one governed metrics tool over the Odoo replica."),
]

# group -> projects its people work on (first is the primary project)
GROUP_PROJECTS = {
    "vg-clinics-be-finance": ["vg-dunning", "vg-invoice-recon"],
    "vg-clinics-be-frontdesk": ["vg-no-shows", "vg-ticket-followup"],
    "vg-clinics-nl-finance": ["vg-dunning", "vg-invoice-recon"],
    "vg-clinics-nl-frontdesk": ["vg-no-shows", "vg-ticket-followup"],
    "vg-labs-diagnostics": ["vg-lab-results"],
    "vg-ambulatory-field": ["vg-lab-results", "vg-no-shows"],
    "vg-group-finance": ["vg-invoice-recon", "vg-dunning", "vg-metrics-pilot"],
    "vg-data-solutions": ["vg-metrics-pilot", "vg-ticket-followup", "vg-invoice-recon"],
    "vg-clinics-de-acquired": ["vg-no-shows", "vg-dunning"],
}

# (slug, display name, group, roles, intensity) — intensity scales daily volume.
PEOPLE = [
    ("an.verstraete", "An Verstraete", "vg-clinics-be-finance", ["user", "odoo_writer"], 1.4),
    ("pieter.claeys", "Pieter Claeys", "vg-clinics-be-finance", ["user", "odoo_writer"], 1.0),
    ("lotte.de.smet", "Lotte De Smet", "vg-clinics-be-finance", ["user"], 0.7),
    ("bram.wouters", "Bram Wouters", "vg-clinics-be-finance", ["user"], 0.5),
    ("ines.goossens", "Ines Goossens", "vg-clinics-be-finance", ["user"], 0.9),
    ("sofie.lambrecht", "Sofie Lambrecht", "vg-clinics-be-frontdesk", ["user", "knowledge_worker"], 0.8),
    ("jonas.peeters", "Jonas Peeters", "vg-clinics-be-frontdesk", ["user", "knowledge_worker"], 0.6),
    ("elke.maes", "Elke Maes", "vg-clinics-be-frontdesk", ["user", "knowledge_worker"], 1.1),
    ("ruben.mertens", "Ruben Mertens", "vg-clinics-be-frontdesk", ["user", "knowledge_worker"], 0.4),
    ("nele.vandamme", "Nele Vandamme", "vg-clinics-be-frontdesk", ["user", "knowledge_worker"], 0.7),
    ("tom.desmedt", "Tom Desmedt", "vg-clinics-be-frontdesk", ["user", "knowledge_worker"], 0.3),
    ("marieke.bosman", "Marieke Bosman", "vg-clinics-nl-finance", ["user", "odoo_writer"], 1.2),
    ("joost.van.dijk", "Joost van Dijk", "vg-clinics-nl-finance", ["user"], 0.8),
    ("femke.kuipers", "Femke Kuipers", "vg-clinics-nl-finance", ["user"], 0.6),
    ("daan.visser", "Daan Visser", "vg-clinics-nl-frontdesk", ["user", "knowledge_worker"], 0.9),
    ("sanne.mulder", "Sanne Mulder", "vg-clinics-nl-frontdesk", ["user", "knowledge_worker"], 0.7),
    ("thijs.smit", "Thijs Smit", "vg-clinics-nl-frontdesk", ["user", "knowledge_worker"], 0.5),
    ("lieke.bakker", "Lieke Bakker", "vg-clinics-nl-frontdesk", ["user", "knowledge_worker"], 0.6),
    ("niels.hendriks", "Niels Hendriks", "vg-clinics-nl-frontdesk", ["user", "knowledge_worker"], 0.3),
    ("katrien.jacobs", "Katrien Jacobs", "vg-labs-diagnostics", ["user", "knowledge_worker"], 1.3),
    ("wim.de.backer", "Wim De Backer", "vg-labs-diagnostics", ["user"], 0.9),
    ("eva.martens", "Eva Martens", "vg-labs-diagnostics", ["user"], 0.8),
    ("stijn.coppens", "Stijn Coppens", "vg-labs-diagnostics", ["user"], 0.6),
    ("hanne.willems", "Hanne Willems", "vg-labs-diagnostics", ["user"], 0.5),
    ("koen.de.wolf", "Koen De Wolf", "vg-labs-diagnostics", ["user", "project_manager"], 0.7),
    ("dr.lisa.vermeulen", "Dr. Lisa Vermeulen", "vg-ambulatory-field", ["user"], 0.9),
    ("dr.maarten.aerts", "Dr. Maarten Aerts", "vg-ambulatory-field", ["user"], 0.6),
    ("dr.julie.dubois", "Dr. Julie Dubois", "vg-ambulatory-field", ["user"], 0.5),
    ("dr.sem.de.vries", "Dr. Sem de Vries", "vg-ambulatory-field", ["user"], 0.7),
    ("dr.noor.jansen", "Dr. Noor Jansen", "vg-ambulatory-field", ["user"], 0.4),
    ("dr.arne.leclercq", "Dr. Arne Leclercq", "vg-ambulatory-field", ["user"], 0.3),
    ("charlotte.renard", "Charlotte Renard", "vg-group-finance", ["user", "project_manager", "odoo_writer"], 1.5),
    ("olivier.lambert", "Olivier Lambert", "vg-group-finance", ["user", "odoo_writer"], 1.1),
    ("emma.devos", "Emma Devos", "vg-group-finance", ["user"], 0.8),
    ("lucas.simon", "Lucas Simon", "vg-group-finance", ["user"], 0.6),
    ("vincent.moreau", "Vincent Moreau", "vg-data-solutions", ["user", "admin", "platform_admin"], 1.6),
    ("sarah.dumont", "Sarah Dumont", "vg-data-solutions", ["user", "admin", "developer"], 1.8),
    ("kevin.hermans", "Kevin Hermans", "vg-data-solutions", ["user", "developer"], 1.5),
    ("julie.thys", "Julie Thys", "vg-data-solutions", ["user", "developer"], 1.2),
    ("arthur.pauwels", "Arthur Pauwels", "vg-data-solutions", ["user", "developer"], 0.9),
    ("jana.schroeder", "Jana Schröder", "vg-clinics-de-acquired", ["user"], 0.8),
    ("felix.brandt", "Felix Brandt", "vg-clinics-de-acquired", ["user"], 0.6),
    ("mia.hoffmann", "Mia Hoffmann", "vg-clinics-de-acquired", ["user"], 0.7),
    ("lukas.krueger", "Lukas Krüger", "vg-clinics-de-acquired", ["user"], 0.4),
]

# Non-interactive identities the farm runs as.
SERVICE_ACCOUNTS = [
    ("svc-dunning-farm", "svc-dunning-farm", "Nightly dunning reminders (farm service account)"),
    ("svc-noshow-farm", "svc-noshow-farm", "Appointment no-show follow-up (farm service account)"),
    ("svc-finance-client", "svc-finance-client", "Group Finance client identity for reconciliation runs"),
]

# Farm clients (the `client` run-as mode bills and reports to these).
CLIENTS = [
    ("vg-group-finance", "Group Finance", "vg-group-finance", "svc-finance-client"),
    ("vg-clinics-be", "Clinics BE", "vg-clinics-be", None),
]

# Marketplace participation: which marketplace each entity group reaches
# through a group rule. Scoped (non-admin) dashboards show the people of the
# groups/projects on the caller's marketplaces' rules — so a clinic user and
# a lab user see different populations.
MARKETPLACE_GROUP_RULES = {
    "dynapps-portal": ["vg-clinics-be-finance", "vg-clinics-be-frontdesk", "vg-clinics-nl-finance",
                        "vg-clinics-nl-frontdesk", "vg-ambulatory-field", "vg-clinics-de-acquired"],
    "dynapps-engineering": ["vg-data-solutions", "vg-labs-diagnostics"],
    "dynapps-agent-farm": ["vg-group-finance", "vg-data-solutions"],
}
MARKETPLACE_PROJECT_RULES = {
    "dynapps-agent-farm": ["vg-dunning", "vg-invoice-recon"],
}
# MCP entitlement: write access to Odoo for finance only; the acquired entity
# is explicitly denied the Odoo server until its company is migrated.
MCP_RULES = [
    ("dynapps-odoo", "group", "vg-group-finance", "allow", "Group Finance reconciles across every entity's journals in the shared Odoo database."),
    ("dynapps-odoo", "group", "vg-clinics-be-finance", "allow", "Clinics BE finance runs dunning on its own Odoo company."),
    ("dynapps-odoo", "group", "vg-clinics-nl-finance", "allow", "Clinics NL finance runs dunning on its own Odoo company."),
    ("dynapps-odoo", "group", "vg-clinics-be-frontdesk", "allow", "Front desk reads appointments and client contacts for rebooking."),
    ("dynapps-odoo", "group", "vg-clinics-nl-frontdesk", "allow", "Front desk reads appointments and client contacts for rebooking."),
    ("dynapps-odoo", "group", "vg-labs-diagnostics", "allow", "Labs read sample and result records."),
    ("dynapps-odoo", "group", "vg-data-solutions", "allow", "Data & Solutions owns the Odoo integration."),
    ("dynapps-odoo", "project", "vg-metrics-pilot", "allow", "Metrics pilot reads the reporting models through the governed tool."),
    ("dynapps-odoo", "group", "vg-clinics-de-acquired", "deny", "Acquired entity: Odoo company not migrated yet (target 2026-11-01). Lift on cut-over."),
    ("dynbrain", "group", "vg-labs-diagnostics", "allow", "Lab reference ranges and reporting templates live in the knowledge base."),
    ("dynbrain", "group", "vg-ambulatory-field", "allow", "Field vets look up protocols on the road."),
    ("dynbrain", "group", "vg-clinics-de-acquired", "allow", "Onboarding guides for the acquired clinics."),
]
