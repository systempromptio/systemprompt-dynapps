# Template HTML — Nota Interna del Chatter (qualification-form, Paso 1)

Nota que se publica en el chatter del `crm.lead` con el **link al formulario**. Objetivo: que el usuario de Odoo vea un texto **legible y renderizado**, con el link **clicable**. NADA de volcado de investigación (eso vive dentro del form).

## Reglas innegociables (leer ANTES de postear)

1. **Postea con `orm_execute_kw(model='crm.lead', method='message_post', ids=[lead_id], kwargs={body, subject:'Form de cualificacion pre-rellenado', subtype_xmlid:'mail.mt_note', message_type:'comment'})`** vía el proxy **Dynapps** (`88fb5415-...`). Es la vía que preserva el HTML sin escapar → el link queda clicable. (El tool dedicado `post_message` del viejo connector `97489e40` ya NO existe.)
2. **El link SIEMPRE es `<a href="{{url}}">…texto…</a>`, NUNCA la URL cruda.** Odoo **NO autolinka** una URL pelada dentro del body del chatter — queda como texto no clicable (bug real lead 32712, 2026-06-17). El único link clicable es el ancla `<a href>`.
3. **NUNCA `orm_create` sobre `mail.message` con body HTML**: ESCAPA el HTML y el SDR ve `&lt;div&gt;...` como texto literal (ilegible). (El escapado de `message_post` era del viejo connector `97489e40`; el proxy Dynapps lo conserva.) Si `message_post` falla → **NO postear una nota degradada** (el texto plano deja la URL no clicable): avisa al SDR del error y dale la URL en el chat para que la pegue él, o reintenta; nunca dejes una nota con link muerto.
4. **Minimalista**: 2-3 `<p>`. SIN "card" con header oscuro, SIN secciones de research, SIN chips.
5. **Link = `<a>` de texto violeta Dynapps `#951CF3`** (Unicorn Violet) en negrita + subrayado, con `target="_blank"`. NUNCA botón con `background` (Odoo lo strippea → texto blanco invisible).
6. **Sin `<!DOCTYPE>`, `<html>`, `<body>`, ni scripts** — Odoo chatter espera un fragmento.
7. **Verifica tras postear**: relee el body de la nota; debe contener `<a href="https` (link clicable). Si en su lugar hay la URL pelada fuera de un `<a>`, o `&lt;`/`&gt;` (escapado) → `orm_unlink` y reintenta el `message_post`.

## Placeholders

| Placeholder | Ejemplo |
|---|---|
| `{{lead_name}}` | "Toldos Transol" |
| `{{fecha}}` | "2026-06-17" |
| `{{sdr_name}}` | "Anna Kirillova" |
| `{{url}}` | "https://share.dynapps.io/v/8865fbce..." |
| `{{version}}` | "v3.x" |

## Plantilla HTML (la que se postea)

```html
<div style="font-family:'Readex Pro',system-ui,Verdana,sans-serif;color:#050543;font-size:13px;line-height:1.6;max-width:600px">
  <p style="margin:0 0 10px 0"><b>Formulario de cualificación pre-rellenado</b> &mdash; {{lead_name}}</p>
  <p style="margin:0 0 12px 0">Abre el formulario, revisa los campos marcados con &#10024; y completa los que faltan. Al terminar clica <b>Copiar resumen para Odoo</b> y pégalo en el chatter como <i>Log note</i>.</p>
  <p style="margin:0 0 12px 0"><a href="{{url}}" target="_blank" style="color:#951CF3;font-weight:700;text-decoration:underline">Abrir formulario de cualificación &rarr;</a></p>
  <p style="margin:0;color:#65637A;font-size:11px">SDR: {{sdr_name}} &middot; {{fecha}} &middot; qualification-form {{version}}</p>
</div>
```

- Si `suggestions = {}`, añade entre el párrafo 1 y 2: `<p style="margin:0 0 12px 0"><i>(Sin investigación pública disponible — rellena todo manualmente.)</i></p>`

## Si `message_post` falla (`Internal server error`)

El proxy Dynapps `message_post` es la vía que conserva el HTML; si devuelve `Internal server error`, NO hay fallback que deje un link clicable (`orm_create` sobre `mail.message` ESCAPA el HTML → `&lt;a&gt;...` ilegible; el texto plano deja la URL **no clicable** — Odoo no autolinka URLs peladas en el chatter, verificado en el lead 32712). Por tanto, **no dejes una nota con link muerto**: dile al SDR que `message_post` está devolviendo error, dale la URL del form directamente en el chat (`{{url}}`) para que la pegue él en el chatter, y/o reintenta (suele ser intermitente).
