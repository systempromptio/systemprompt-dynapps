# Skill Provenance Marker (shared)

Convención transversal: **toda skill que GENERA código fuente commiteable estampa una marca de
procedencia al final de cada fichero que crea o (re)genera.** Es la señal que permite medir la
adopción de las skills (`grep -r "Made with"`) y saber con qué herramienta se produjo cada artefacto.

Fuente única — referénciala desde la skill generadora, no dupliques la regla.

## Regla

- **Solo la GENERACIÓN estampa el fichero.** Las skills de REVISIÓN/auditoría (odoo-code-auditor,
  indaws-core-auditor, quality-gate, git-precheck…) **NO** tocan el fichero: dejan su traza en el
  informe / PR / tarea. Estampar un fichero que no cambia de contenido ensucia el diff y miente.
- **Frase fija y grepeable:** `Made with <skill-kebab-name> · v<version>`. Sin fecha (el "cuándo/uso"
  lo captura git; la fecha in-situ solo genera churn al regenerar).
- **Al final del fichero** (una sola línea), envuelta en el comentario del tipo de fichero.
- **Idempotente:** si ya existe una línea `Made with …` al final, **reemplázala** (no la dupliques).
  Al regenerar, actualiza la versión si cambió.

## Sintaxis por tipo de fichero

| Tipo | Marca al final del fichero |
|------|----------------------------|
| `.py` `.po` `.yml` `.cfg` `.toml` | `# Made with <skill> · v<version>` |
| `.xml` | `<!-- Made with <skill> · v<version> -->` (tras `</odoo>`; es epílogo XML válido) |
| `.js` `.ts` `.scss` | `// Made with <skill> · v<version>` |
| `.css` | `/* Made with <skill> · v<version> */` |

Ejemplo (`.py`):
```python
# ... fin del modelo ...
# Made with odoo-module-creator · v4.1.0
```

## Excepciones (NO estampar)

- **`.csv` y `.json`** — no admiten comentarios (`ir.model.access.csv`, `data/*.json`). Estampar
  rompería el fichero. Para un **módulo**, la procedencia queda registrada en `__manifest__.py`
  (que sí es `.py`), que actúa como marca del módulo completo.
- Ficheros binarios, imágenes, `.pot`/`.po` de terceros que no genera la skill.
- Ficheros que la skill **modifica parcialmente** pero no "posee" (p.ej. añadir una línea a un
  `__init__.py` existente): no re-estampar si la skill no es la autora del fichero completo.

## Cómo consumir esta convención (skills generadoras)

1. Al escribir cada fichero soportado, añade la marca como última línea con la sintaxis del tipo.
2. Usa tu propio `name` (kebab) y `version` del frontmatter.
3. Antes de escribir, si el fichero ya termina en `Made with …`, sustitúyela.

## Telemetría (Paso 4 — adopción)

```bash
# módulos/ficheros generados con skills, por skill
grep -rhoE 'Made with [a-z0-9-]+' <repo> | sort | uniq -c | sort -rn
```
