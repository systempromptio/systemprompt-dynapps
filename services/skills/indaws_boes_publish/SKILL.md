---
name: indaws-boes-publish
version: 1.0.1
model: sonnet
effort: low
description: "Regenera y publica el BOES (Boletín Oficial de Skills) en sus dos formatos: el artifact navegable (catálogo HTML en claude.ai) y el PDF, a partir del `description` y el cuerpo de cada SKILL.md del ecosistema. Usa este skill cuando el usuario diga 'publicar BOES', 'actualizar el BOES', 'nueva edición del BOES', 'regenera el catálogo de skills', 'sube el BOES', 'actualiza el artifact del BOES', o invoque '/boes-publish'. NO edita los outputs a mano: siempre regenera desde los SKILL.md (las skills son la única fuente de verdad)."
---

# boes-publish — publicar el Boletín Oficial de Skills

El **BOES** es el registro navegable del ecosistema de skills de Dynapps/Indaws. Vive en
`ai-claude-common/boes/` y se publica en **dos formatos derivados** de los mismos datos:

- **Artifact navegable** (`boes-catalogo.html`) → hospedado en claude.ai. Catálogo por repositorio con
  buscador, qué hace / cómo se usa / casos de uso, cadena de skills navegable y SKILL.md completo.
- **PDF** (`BOES-principal.pdf` / `BOES-NNN.pdf`) → publicación on-brand para compartir/imprimir.

## Regla de oro: la base son las skills

Los `SKILL.md` son la **única fuente de verdad (SSOT)**. El artifact, el PDF y `skills.json` son
**vistas generadas**. **Nunca** se editan a mano: se regeneran. Si algo del catálogo está mal, se
arregla en el `SKILL.md` de origen, no en el output.

```
SKILL.md (SSOT) ─► extract_skills.py ─► skills.json ─┬─► build_artifact.py ─► boes-catalogo.html ─► Artifact
                                                     └─► render_boes_pdf.py ─► BOES-*.pdf
```

## Procedimiento

Todas las rutas son relativas a `ai-claude-common/boes/`. Los scripts son portables (se autolocalizan).

### 1. Regenerar los datos
```bash
cd <repo>/ai-claude-common/boes/scripts && python3 extract_skills.py
```
Escanea `ai-claude-*/skills/*/SKILL.md`, extrae frontmatter (name, version, effort, model,
description), separa *qué hace* / *casos de uso* (triggers), captura el cuerpo del SKILL.md y calcula
la **cadena de skills** (quién menciona a quién, solo nombres distintivos con guion). Escribe
`boes/skills.json`. Imprime el total y el desglose por repo — compáralo con la edición anterior para el diff.

### 2. Generar el artifact
```bash
python3 build_artifact.py    # lee ../skills.json + fuentes Readex + logos BOES → ../boes-catalogo.html
```

### 3. Publicar el artifact (⚠️ paso clave)
Llama a la tool **Artifact** con el HTML generado. Para **actualizar el BOES existente y conservar la
URL**, hay que pasar `url` — si no, se crea un artifact nuevo:

- `file_path`: `<repo>/ai-claude-common/boes/boes-catalogo.html`
- `url`: **`https://claude.ai/code/artifact/d22ce7b9-2c94-42b7-ab67-d8e86361d86e`** ← URL oficial del BOES
- `label`: `vN-<resumen>` (versión legible del cambio) — no pases `icon` al actualizar: el artifact conserva el suyo.
- `description`: una frase con el estado actual (nº skills, novedades)

> **Gotcha:** desde una sesión que **no** publicó este artifact, omitir `url` **acuña una URL nueva**.
> Pasa siempre la URL oficial de arriba. (Si se pierde, `Artifact action:"list"` la localiza por título
> "BOES · Catálogo de Skills".) El artifact es privado hasta que el usuario lo comparta desde su menú.

### 4. (Opcional) PDF y edición del boletín
- PDF completo al día: `python3 render_boes_pdf.py` → `../BOES-principal.pdf`.
- PDF diff de una edición: `python3 render_boes_pdf.py --diff N` → `../BOES-NNN.pdf`.
- Si es una **nueva edición** (hito/tanda de cambios), antepón en `boes/BOES.md` una sección
  `## BOES nº N — <fecha>` con el **diff** (altas/bajas/deprecaciones/bumps) respecto a la anterior;
  no re-listes todo. La nº1 es la línea base completa.

## Verificación antes de publicar
- El total de skills cuadra con lo esperado (hoy: **168**); si cambió, hay altas/bajas que deben ir en el diff.
- `boes-catalogo.html` no referencia hosts externos (CSP del artifact lo bloquea): fuentes, logos y datos van embebidos.
- Sanity opcional del JS: extraer el `<script>` y `node --check`.

## GOTCHAS
1. **Artifact sin `url=` acuña URL nueva.** Desde una sesión que no publicó este artifact, la tool Artifact crea otra URL en vez de actualizar. Pasa SIEMPRE `url=https://claude.ai/code/artifact/d22ce7b9-2c94-42b7-ab67-d8e86361d86e`. Si se pierde, `Artifact action:"list"` la localiza por título.
2. **Los outputs son vistas generadas, NO fuente.** Editar `boes-catalogo.html`, `skills.json` o los PDF a mano se pierde en la siguiente regeneración. Arregla el `SKILL.md` de origen.
3. **CSP del artifact bloquea hosts externos.** Fuentes (Readex base64), logos (SVG inline) y datos (JSON) deben ir embebidos en el HTML — nada de CDNs ni fetch.
4. **Los scripts asumen el workspace de Toni.** Rutas relativas a `boes/scripts/` esperan los repos `ai-claude-*` como hermanos (`../../..`). Fuera de ese layout, ajústalas.
5. **El catálogo se cuenta a sí mismo.** `boes-publish` es una skill más → al regenerar, el total sube. Es esperado.

## Delegates to
- `dynapps-brand` — fuentes Readex y paleta que embebe el artifact y el PDF.

## REFERENCES
- `references/shared/dynapps-identity.md` — identidad Dynapps (color/tipografía) que deben respetar el artifact y el PDF.

## Consumed by
None — skill de mantenimiento invocada directamente (`/boes-publish`).

## Notas
- Multiidioma: **aparcado**. Si se retoma, el chrome se traduce en el build (diccionario ES/EN) y el
  contenido, si acaso, como caché derivada (`skills.<lang>.json`) — **nunca** metiendo idiomas en el
  `description` (es un campo funcional que dispara el triggering).
- Salud del ecosistema (refs rotas, duplicados): `skill-designer/scripts/doctor.py <repo>`.
