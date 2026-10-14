# Website Forms (lead, email, task, ticket, job…)

How to build `s_website_form` blocks that, on submit, perform an Odoo action: send an email,
create a CRM lead, a task, a ticket, a job application, etc. Same snippet, different
`data-model_name` + the matching module installed. Template: `templates/lead_form_block.html`.
Scripts: `build_lead_form.py`, `ensure_website_crm.py`.

## How it works

`s_website_form` posts to `/website/form/`. The `data-model_name` on the `<form>` selects the
**action/target model**. The model must be website-form-enabled (`website_form_access=True`), which
the corresponding module provides. The controller validates posted fields against the model's
whitelist and performs the action.

## Form action types (all the options Odoo offers)

| Action (builder label) | `data-model_name` | Module needed | Key field names |
|---|---|---|---|
| **Send an Email** | `mail.mail` | website (core) | hidden `email_to`; `name` (subject), `email_from`, `description` |
| **Create a Lead/Opportunity** | `crm.lead` | `website_crm` | `contact_name`, `email_from`, `phone`, `partner_name`, `name` (Subject, model-required), `description` |
| **Apply for a Job** | `hr.applicant` | `website_hr_recruitment` | `partner_name`, `email_from`, `partner_phone`, `description`, `name` |
| **Create a Task** | `project.task` | `website_project` (+ project) | `name` (required), `email_from`/`partner_id`, `description` |
| **Create a Ticket** | `helpdesk.ticket` | `website_helpdesk` (Enterprise) | `name`, `email_from`, `description`, `partner_name` |
| **Create a Customer** | `res.partner` | website (core) | `name`, `email`, `phone`, `company_name` |
| **Custom model** | `<your.model>` | set `website_form_access=True` on the `ir.model` | whitelisted fields only |

> The builder's "Action" dropdown lists exactly the models with `website_form_access=True`.
> Installing the module adds its model to that list. Verify availability with
> `search_records.sh ir.model '[["model","=","crm.lead"]]' '["website_form_access"]'` — and install
> the module (e.g. `ensure_website_crm.py --install`) if the action isn't available.

## Modern field markup (Odoo 18/19) — use THIS structure

Hand-authored forms that omit the inner `row`/`col` wrappers render but Odoo flags them as an
**old version**. The current structure wraps each field's label + input in an inner row:

```html
<div data-name="Field" class="s_website_form_field mb-3 col-12 s_website_form_required" data-type="char">
  <div class="row s_col_no_resize s_col_no_bgcolor">
    <label class="col-form-label s_website_form_label" for="iw_contact_name">
      <span class="s_website_form_label_content">Name</span><span class="s_website_form_mark"> *</span>
    </label>
    <div class="col-sm">
      <input class="form-control s_website_form_input" type="text" name="contact_name"
             required="1" data-fill-with="name" id="iw_contact_name"/>
    </div>
  </div>
</div>
```

Key modern bits vs the old simplified form:
- Inner `<div class="row s_col_no_resize s_col_no_bgcolor">` wrapping label + `<div class="col-sm">input</div>`.
- `data-fill-with="name|email|phone|commercial_company_name"` + `data-pre-fill="true"` on the `<form>` → pre-fills the logged-in visitor's data.
- The model's required field (e.g. `crm.lead.name`) is a **visible** field with class
  `s_website_form_model_required` (label "Subject"), not a hidden input.
- Each input has an `id` and the label `for` matches it.

Form tag:
```html
<form action="/website/form/" method="post" enctype="multipart/form-data"
      class="o_mark_required" data-mark="*" data-pre-fill="true"
      data-model_name="crm.lead"
      data-success-mode="redirect" data-success-page="/contactus-thank-you">
```

## Success behavior — `/contactus-thank-you` already exists

Odoo ships a default thank-you page at **`/contactus-thank-you`**. Use it as the redirect target —
**do not create your own thank-you page**. Modes:
- `data-success-mode="redirect"` + `data-success-page="/contactus-thank-you"` (or any existing page).
- `data-success-mode="message"` — shows an inline success message, stays on the page (needs the
  `.s_website_form_end_message` element; verify markup on the instance).

## Newsletter subscribe block — DIFFERENT mechanism (not `s_website_form`)

The "subscribe to the newsletter" block is **not** an `s_website_form` and does **not** post to
`/website/form/`. It's the native `s_newsletter_block` with the `js_subscribe` widget, which
subscribes the email to a **`mailing.list`** (mass-mailing). Needs the `mass_mailing` module.

```html
<div class="s_newsletter_subscribe_form s_newsletter_list js_subscribe"
     data-vxml="001" data-list-id="1" data-name="Newsletter Form">
  <div class="js_subscribed_wrap d-none">
    <p class="h4-fs text-success"><i class="fa fa-check-circle-o" role="img"></i> Thanks for subscribing!</p>
  </div>
  <div class="js_subscribe_wrap">
    <input type="email" name="email" class="js_subscribe_value form-control …" placeholder="Your email"/>
    <a role="button" href="#" class="btn js_subscribe_btn o_submit …">Suscribirme</a>
  </div>
</div>
```
Contract the widget keys on: outer `.js_subscribe` + `data-list-id="<mailing.list id>"`; the email
`<input class="js_subscribe_value">`; the button `.js_subscribe_btn`; the success `.js_subscribed_wrap`.
Keep those classes intact and **never** add `onclick` — the JS ships with the theme/`mass_mailing`.

### GOTCHA — the email field looks "pre-filled with a random email". It isn't a bug.
When the visitor is **logged in**, `js_subscribe` auto-fills the input with the **session user's
email**. Building/previewing as `admin`, you'll see admin's address in "Your email" — that's expected.
An **anonymous** visitor sees the empty `placeholder`. Don't try to "fix" it in markup (no `value=""`,
no JS) — verify by opening the page in a private/incognito window (logged out) and the field is empty.

### DEPLOY — `data-list-id` is instance-specific, remap it (like attachment ids)
The `mailing.list` id differs per instance. A deploy-to-master script must **look up (or create) the
mailing list by NAME** on the target and rewrite `data-list-id` in the arch — same portability problem
as `/web/image/<id>` attachment ids (see `layout-recipes.md` §6). Never ship a hardcoded local id.

## Whitelisting & gotchas

1. **Module gating:** without the right module the action isn't available and the submit fails. Verify/install first.
2. **Only whitelisted fields** are accepted. Standard fields per model are pre-whitelisted by the
   module. To expose a custom field: `website_form_access=True` on the `ir.model` +
   `website_form_label` on the `ir.model.fields`. Prefer standard fields.
3. **CSRF + send JS** are wired via the `s_website_form*` classes and the `.s_website_form_send`
   button — keep them intact; never add `onclick`.
4. **`mail.mail` (Send an Email)** needs the hidden `email_to` and a `name` (subject). The recipient
   is set in `email_to`.
5. **Required model fields** (e.g. `crm.lead.name`, `project.task.name`) must be present
   (`s_website_form_model_required`) or the create fails.
6. **Editing in the builder rewrites the arch** — see SKILL.md GOTCHA; coordinate API writes.
