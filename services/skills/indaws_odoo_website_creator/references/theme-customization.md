# Theme Customization (colors & fonts) — site-wide, no repeated code

For brand tokens that apply to EVERY page (palette colors, body/heading fonts, button styles),
**prefer** setting them at the **theme** level (the Website Builder's **THEME** tab) instead of
repeating the same hex on every element — a single palette change then restyles the whole site.

> **This is a preference, not a hard rule.** Hardcoding a specific hex (inline `style="color:#…"`
> or a class with a fixed color) is perfectly fine when you need an exact one-off color that isn't
> a theme token (a specific accent, a brand color outside the 5-color palette, a decorative tint).
> Don't contort the markup to avoid hex. Rule of thumb: **recurring/brand-wide → theme token;
> specific/one-off → hex is OK.**

## API-ONLY — no server filesystem access

> On a **client** Odoo you have access ONLY to Odoo (XML-RPC/JSON-RPC via odoo-pilot or MCP).
> You do **NOT** have the server filesystem / addon source. So everything here is done through the
> API, and the EXACT url + format is discovered through the API (see "Discovery trick"). Never rely
> on reading `…/odoo/addons/website/static/src/scss/…` from disk — that only exists on a dev box.

## How the THEME panel persists changes

The builder writes SCSS variable customizations through **`make_scss_customization(url, values)`**
(called as `[url, values]`). Odoo stores the overrides as an `ir.attachment` and recompiles
`web.assets_frontend`.

> ⚠️ **EL NOMBRE DEL MODELO CAMBIA ENTRE v18 Y v19**, y esta referencia decía `web_editor.assets`
> para las dos hasta 2026-08-07. En v19 ese modelo **no existe** y la llamada falla:
>
> | Versión | Modelo | Dónde |
> |---|---|---|
> | 18.0 | **`web_editor.assets`** | `website/models/assets.py:13`, `Assets` (`_inherit`), método en `website/models/assets.py:16`, `Assets` |
> | 19.0 | **`website.assets`** | `website/models/assets.py:17`, `WebsiteAssets` (`_name`, modelo NUEVO, v19), método en `website/models/assets.py:189`, `WebsiteAssets` |
>
> El módulo `web_editor` **desaparece en v19** (no existe `addons/web_editor`, y `web_editor.assets`
> solo sobrevive como texto en un comentario de `web/__manifest__.py:44`). Verificado 2026-08-07.
>
> **Y desaparecer un módulo tiene un radio que conviene medir antes de migrar** (añadido 2026-08-17).
> Un módulo custom se rompe en v19 por **dos** vías distintas, y la segunda es la que se escapa:
>
> 1. **`depends`** — `'web_editor'` en el `__manifest__.py`. En el core de v18 lo declaran **5**
>    módulos; en tu custom, míralo con
>    `grep -l "'web_editor'," $CUSTOM/*/__manifest__.py`.
> 2. **bundles de assets `web_editor.*`** — no aparecen en `depends`, van dentro de `'assets': {…}`,
>    y **son más**: **7** módulos del core de v18 los citan. Un bundle inexistente **no revienta el
>    arranque**: el asset simplemente no se carga y el editor sale a medias.
>
> Lo que define cada uno (medido en los dos manifests):
>
> | Bundle v18 (`web_editor/__manifest__.py`) | ¿Hay equivalente en v19? |
> |---|---|
> | `web_editor.common` | **sí, mismo nombre** → `html_editor.common` |
> | `web_editor.assets_media_dialog` (`:120`) | **sí, mismo nombre** → `html_editor.assets_media_dialog` (`:64`) |
> | `web_editor.assets_wysiwyg` (`:128`) | **no por nombre.** El candidato es `html_editor.assets_editor` (`:44`), pero **no lo he verificado contenido a contenido** |
> | `web_editor.backend_assets_wysiwyg` (`:205`) | **no existe con ese nombre** |
> | `web_editor.assets_snippets_menu` (`:25`) | **no existe con ese nombre** |
> | `web_editor.wysiwyg_iframe_editor_assets` (`:30`), `web_editor.assets_tests_styles` (`:123`), `web_editor.mocha_tests` (`:283`) | **no existen con ese nombre** |
>
> Y v19 **añade** bundles que no tenían contrapartida: `html_editor.assets_readonly` (`:69`),
> `assets_history_diff` (`:60`), `assets_image_cropper` (`:94`), `assets_prism` (`:114`).
>
> **O sea: NO es un renombrado de módulo, es una reorganización.** Dos bundles pasan por sustitución
> de prefijo y el resto hay que buscarlo uno a uno en
> `odoo19/addons/html_editor/__manifest__.py`. Si alguien te dice *"cambia `web_editor.` por
> `html_editor.` y listo"*, le funcionará en dos casos de siete.
>
> Como esto se ejecuta contra la instancia del cliente, **detecta la versión antes de llamar**
> (`references/version-notes.md` § Auto-detect) y usa el modelo que toque. Si te equivocas, el error
> es de modelo inexistente, no de datos — pero te enteras después de haber decidido la paleta.

Reproduce it via API:

| What | SCSS file (url arg) | Keys / vars |
|------|---------------------|-------------|
| **Palette colors** | `/website/static/src/scss/options/colors/user_color_palette.scss` | `o-color-1` … `o-color-5` |
| **Fonts & theme values** | `/website/static/src/scss/options/user_values.scss` | `font` (body), `headings-font`, sizes, button vars… |

The created attachment ends up at url `/_custom/web.assets_frontend/<original-path>`.

## Quoting rule (this WILL break the site if wrong)

Values are written **verbatim** into a SCSS map (`'key': <value>,`). So:

- **Hex colors → UNQUOTED** (a raw SCSS color). `'o-color-1': #1B1B4B` ✅.
  Quoting a hex (`'#1B1B4B'`) makes it a *string* → `lightness($color)` fails →
  `web.assets_frontend` won't compile → **every frontend page 500s**. (Real bug; matches Odoo's
  default palette which stores unquoted hex.)
- **Font names & other string values → QUOTED.** `'headings-font': 'Arvo'` ✅ (matches the default
  `'headings-font': 'Inter Tight'`).

### Colors (confirmed v18)

```python
# build JSON in Python to avoid shell-quoting hell
url = "/website/static/src/scss/options/colors/user_color_palette.scss"
values = {"o-color-1": "#1B1B4B", "o-color-2": "#15B5A6", "o-color-3": "#8BE3D2",
          "o-color-4": "#FFFFFF", "o-color-5": "#1B1B4B"}   # hex, NO quotes
# v18: execute_method.sh web_editor.assets make_scss_customization '[url, values]' '{}'
# v19: execute_method.sh website.assets    make_scss_customization '[url, values]' '{}'
```

### Fonts (confirmed v18)

```python
url = "/website/static/src/scss/options/user_values.scss"
values = {"headings-font": "'Arvo'"}    # font NAME, quoted
```

The font must already be in `$o-theme-font-configs` — the built-ins (auto-loaded from Google,
no registration needed) are: `Roboto`, `Open Sans`, `Source Sans Pro`, `Raleway`, `Noto Serif`,
`Arvo`, `Inter Tight`, `Inter`. To use any other font, add it first via the THEME panel
("Add a font"), which registers it in `google-fonts` in `user_values.scss`.

## ALWAYS verify after writing

A bad value breaks SCSS compilation silently until a page is loaded. After every theme write, load
a published frontend page and confirm it compiles:

```bash
curl -s -o /dev/null -w "%{http_code}\n" "$ODOO_URL/contactus"   # expect 200, not 500
curl -s "$ODOO_URL/contactus" | grep -ci "lightness\|assetsbundle\|Error:"   # expect 0
```
If it 500s with a `lightness`/assetsbundle error, your value format is wrong — fix or reset.

## Discovery trick (API-only — use when unsure of url/var/format)

To learn the EXACT url + variable format for the target instance/theme/version WITHOUT source access:

1. In the Website Builder → **THEME** tab, change ONE thing (a color, or the heading font) and **Save**.
2. Read what Odoo wrote: `search_records.sh ir.attachment '[["url","ilike","scss/options"]]' '["id","name","url"]' 20`.
3. Read that attachment's `datas` (base64) → decode → you see the exact map + value format. Replicate/extend it via `make_scss_customization`.

Before any customization the attachments are absent (theme on defaults); they appear after the
first change (manual or API). This is the reliable path on a client where you can't read the source.

## When to use this vs other tiers

| Need | Use |
|------|-----|
| Brand palette, body/heading font, button look — **everywhere** | **Theme customization (this file)** |
| Arbitrary one-off CSS for a custom block | `css-injection-ladder.md` (ir.asset / Custom Code) |
| Inline color on a single element | `style="…"` attribute (fine for one-offs) |

## Using the palette in page content

Once the palette is set, **prefer** THEME classes over inline hex for brand-consistent styling — a
palette change then restyles every page. (Still fine to hardcode a hex for a specific one-off color,
per the note up top.) Bootstrap maps `primary`=o-color-1, `secondary`=o-color-2. Confirmed mappings:

| Want | Class |
|------|-------|
| Section: white bg / dark text | `o_cc1` |
| Section: dark bg / light text | `o_cc5` (combos `o_cc1`→`o_cc5` go light→dark) |
| Neutral image placeholder / light grey | explicit colour, e.g. `.wc-ph{background:#e9ecef}` (NOT `bg-light` — that's a palette colour, see ⚠️ below) |
| Heading/accent in brand primary | `text-primary` |
| Accent text (secondary) | `text-secondary` |
| Buttons | `btn-primary` / `btn-outline-primary` / `btn-secondary` |
| Badge / pill | `text-bg-secondary` |

`o_cc1`..`o_cc5` are "color combinations" — they set bg + text + headings + links + buttons
coherently from the palette in one class. To confirm a combo's exact background, grep the compiled
CSS (`curl .../web.assets_frontend...css | grep '.o_cc5'`) — it's palette-derived, no source needed.

## ⚠️ Bootstrap's contextual classes follow the PALETTE, not fixed greys

In an Odoo theme, `bg-light`/`text-light`/`bg-secondary`/`btn-success`… are bound to the palette via
CSS vars (`--light: <o-color-3>`, `--secondary: <o-color-2>`, …), NOT to Bootstrap's default greys.
So **`bg-light` can render a BRAND color** (e.g. mint), not a neutral grey (real bug seen: image
placeholders came out mint because `--light` = o-color-3). For a **neutral placeholder/grey** do NOT
use `bg-light`: use an explicit color (e.g. inject a class `.wc-ph{background-color:#e9ecef;}` and
apply it). Verify the real value by grepping the compiled CSS (`--light`, `.bg-light`).

## Reset

Re-run `make_scss_customization` with empty values (`{}`) for that url, or delete the customization
`ir.attachment`, to revert to theme defaults. Then verify a frontend page compiles.
