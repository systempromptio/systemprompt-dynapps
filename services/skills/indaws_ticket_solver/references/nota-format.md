# Formato nota chatter — Ticket Solver

Escribe HTML a `/tmp/note_<ticket>.html` (revisable en el gate) y publícalo con
`orm_post_message` (prefijo `mcp__<servidor>__` resuelto en la conexión — ver
`references/credentials-setup.md`, no asumas `odoo-indaws`):

```
model="helpdesk.ticket"  res_id=<ticket>  body=<contenido del fichero>
body_is_html=true        subtype_xmlid="mail.mt_note"
message_type="comment"
```

El autor es el usuario conectado al MCP (`whoami`) — no se pasa `author_id`.

`message_type="comment"` es **obligatorio y explícito**: sin él Odoo 19 graba la
nota como `notification`, que no es lo que hacía `post_internal_note.py`.

Tras publicar, relee el `body` del `mail.message` creado y comprueba que los tags
llegaron intactos; ver GOTCHA 2 del SKILL.md para el fallback.

> Verificado en Odoo 19 (portal Dynapps): el repertorio completo de tags
> permitidos —`<p>`, `<b>`, `<code>`, `<ul>/<li>`, `<pre><code>`, `<p></p>` y la
> entidad `&lt;p&gt;`— sobrevive intacto al saneado de `message_post`.

## Tags permitidos

`<p>`, `<b>`, `<code>`, `<ul>`/`<li>`, `<pre><code>`.

## Prohibido

`<div>`, `<h3>`, `<hr>`, `<br>`, `<br/>`, `<table>`, `<ol>`, `<i>`, atributos `style=`/`class=` en cualquier tag, HTML pre-escapado fuera de `<code>`. Entre párrafos: `</p><p>`. Primer carácter no-whitespace DEBE ser `<p`, `<ul` o `<pre` (si no, el helper aborta).

## Estructura visual obligatoria (ambos modos)

Cada bloque empieza con `<p>` y abre con `<b>Etiqueta:</b>`. Entre el bloque técnico y el borrador de correo va SIEMPRE `<p></p>` vacío. El borrador lleva su propia etiqueta `<b>Borrador correo cliente:</b>` en `<p>` aparte.

## Checklist de formato — todos ✓ antes del gate

- Primer char no-whitespace: `<p`, `<ul` o `<pre`
- Sin tags prohibidos ni atributos `style=`/`class=`
- Cada bloque: `<p><b>Etiqueta:</b>` (sin fusionar bloques)
- Separador `<p></p>` vacío antes del borrador
- Borrador: `<b>Borrador correo cliente:</b>` en `<p>` propio + cuerpo en `<p>` propio
- Modo A: solo `Resolución:` / Modo B: solo `Problema Detectado:` + `Solución Propuesta:`

## Modo A — Resuelto / Duda funcional

```html
<p><b>Resolución:</b> <1-3 frases técnicas simples.></p>
<p><b>Backup:</b> <code>/tmp/rollback_&lt;ticket&gt;_&lt;ts&gt;.json</code></p>
<p></p>
<p><b>Borrador correo cliente:</b></p>
<p>Buenas <nombre>, <causa 1 frase + solución 1-2 frases, sin IDs/modelos/jerga>. Saludos.</p>
```

Línea Backup solo si hubo JSON. Write inline → deja comando inverso en `Resolución:` y omite Backup.

## Modo B — Escalado a dev / upstream

```html
<p><b>Problema Detectado:</b> <Contexto tenant+versión+módulo. Registro ejemplo con ID y valores inline con <code>campo=valor</code>. Qué compute/método no surte efecto y por qué. Scope sistémico con <code>search_count</code> exacto. Hipótesis: override/archivo. Referencia a tickets previos si los hay.></p>
<p><b>Solución Propuesta:</b> <Módulo y archivo:método a tocar. Referencia al estándar Odoo. 2-3 tests mínimos concretos. Comando recompute/backfill sobre IDs afectados. Stage sugerido y asignado si lo sabes.></p>
<p></p>
<p><b>Borrador correo cliente:</b></p>
<p>Buenas <nombre>, <lenguaje cliente: qué se está haciendo, plazo aproximado, consecuencias operativas si las hay>. Saludos.</p>
```

**Prohibido en Modo B**: `<pre><code>` con diff unified, commit draft, bullet lists.

Ejemplo canónico del equipo: ticket 13894 (inDAWS helpdesk).

## Reglas de separación

- NUNCA mezcles `Resolución:` con `Problema Detectado:` — son modos distintos.
- NUNCA mezcles el borrador de correo dentro de los párrafos técnicos.
- NUNCA mezcles consecuencias operativas dentro de `Problema Detectado:` / `Solución Propuesta:`.
- Si añades `Backup:` en Modo A, va siempre como bloque propio entre `Resolución:` y el separador.
