# Multi-language page output

The page *content* language is a user choice (asked in step 2), independent of the language this
skill or its operator uses. Odoo serves one `website.page` record in every enabled language via
per-field **translations** and a `/<lang>/` URL prefix — you do **not** create a page per language.

## Model of how Odoo stores it

- **Languages**: a language is usable only if its `res.lang` is `active=True` **and** it is listed in
  the website's `website.language_ids` (m2m). The site's base language is `website.default_lang_id`.
- **Translatable content**: `ir.ui.view.arch_db` (the page arch), plus `website.page` SEO fields
  (`website_meta_title`, `website_meta_description`, …) and `website.menu.name`, are translatable.
  Modern Odoo (v16+) stores each translation per-lang on the same field — no separate records.
- **URL**: the same `website.page.url` serves all languages; Odoo prefixes non-default languages with
  `/<lang>/` (e.g. `/fr/servicios`) through the language switcher. One record, many URLs.

## Recipe

1. **Confirm the target languages are enabled** before writing anything. Read active languages and
   the website's enabled set:
   - `res.lang` `search_read [('active','=',true)]` → codes like `en_US`, `es_ES`, `fr_FR`, `nl_NL`.
   - read `website.language_ids` / `default_lang_id` on the target `website`.
   If a requested language is missing, install/activate it (`base.language_install`) and add it to
   `website.language_ids` first — or tell the user it isn't available and stop. See GOTCHA 18.
2. **Author + create the base-language page** in `default_lang_id` (steps 3–9 as usual). Have
   `odoo-frontend` write the copy in the base language.
3. **Add each extra language as a translation on the SAME records.** Have `odoo-frontend` produce the
   translated copy, then `write` the translated `arch_db` under that language's context — this sets
   only that language's value and leaves the base untouched:
   ```
   # via odoo-pilot / execute_kw — context lang selects which translation you write
   execute_kw('ir.ui.view', 'write',
              [[view_id], {'arch_db': '<translated arch>'}],
              {'context': {'lang': 'fr_FR'}})
   ```
   Repeat for SEO fields and any menu name. (MCP: pass the same `context` in the call.)
4. **Verify** each `/<lang>/` URL renders the translated copy and the base URL is unchanged.

## GOTCHAS

- **A page per language is wrong.** Duplicating the `website.page`/`ir.ui.view` per language
  fragments the URL, breaks the language switcher, and desyncs SEO/menus. Always translate onto the
  same records via `with_context(lang=…)`.
- **Write the base first.** Writing a translation under a lang context before the base value exists
  can store the translation as the source. Create in `default_lang_id`, then translate.
- **Language must be enabled, not just installed.** An `active` `res.lang` that is not in
  `website.language_ids` won't get a `/<lang>/` URL and its translation stays invisible on the front
  end.
- **Modes B/D too.** Local-HTML and duplicated pages start single-language; if the user wants them
  multilingual, apply steps 3–4 after the base page exists.
- Keep this page UNPUBLISHED like any other output (skill default); enabling translations does not
  publish it.
