# FASE 1 — Estudio previo de migración 📄

**Centro de gravedad del flujo.** Sin estudio previo, no hay planificación.

## [STOP] Solicitar estudio al usuario

```
Antes de tocar nada, necesito el **estudio previo de migración**.
Es el documento del consultor/cliente con decisiones módulo a módulo.

¿Cómo me lo pasas?
  a) Tengo un .md / .docx / .pdf → ruta del fichero
  b) Lo recibí en correo / chat → pégalo aquí
  c) No hay estudio formal → trabajamos contra detección (peor)
```

## Procesado

```bash
# Markdown / docx / pdf (autodetect por extensión)
indaws-migrate load-estudio --file /ruta/estudio.docx

# Texto pegado
indaws-migrate load-estudio --pasted-text "## mass_editing\nneutralizar..."

# Stdin (cuando el agente recibe contenido del chat)
echo "..." | indaws-migrate load-estudio --stdin
```

Output: `migration_kit_<cliente>/docs/estudio_normalizado.yaml` + `parsing_report.md`.

### Sub-agente lector (`estudio-parser`)

Cuando el estudio es **md/docx/pdf largo** (>20 páginas), lánzalo en un
sub-agente sonnet en vez de leerlo inline — el documento no contamina el
contexto del principal y vuelve normalizado.

```python
Agent(model="sonnet", description="Parse estudio", prompt=<ver agent-tree.md §1>)
```

Inputs imprescindibles en el prompt: `estudio_path`, extensión,
`from_version`, `to_version`, `cliente`. Devuelve JSON con `modules`,
`constraints`, `blockers_funcionales`. Ver schema completo en
`../agent-tree.md`.

## Información mínima esperada

- Contexto (versiones, edition, countries, deployment)
- Inventario módulos custom + OCA usados
- **Decisión por módulo**: KEEP / STUDIO / CORE_REPLACE / STUB / NEUTRALIZE / REPLACE_WITH / REMOVE / TO_DROP
- Justificación (1-2 líneas)
- Sucesores conocidos (si REPLACE_WITH)
- Reports custom + decisión
- Studio existente (si lo hay)
- Riesgos identificados

## ⚠ Recomendación timing — módulos sin uso

**Si el cliente identifica módulos sin uso real (action `TO_DROP` o `REMOVE`)
y todavía hay acceso a la BD origen (v_X) o a un staging v_X:** preferible
desinstalarlos AHÍ antes de generar el dump para el Upgrade Service.

Razones:
- Odoo v_X ejecuta el `_uninstall_hook` propio del módulo en su versión.
- El Upgrade Service no procesa el módulo (cero scripts intermedios).
- BD post-upgrade limpia (sin tablas residuales ni FKs colgando).
- Iteraciones del Upgrade Service más rápidas y con menos errores.

Si la BD ya está post-upgrade (no se puede volver atrás): el helper
`DropModules` (TO_DROP) cubre el caso, pero con pre-checks fail-fast
(dependencias, FKs externas, volumen de datos). Ver
`references/playbooks/modules/to_drop.md`.

## Modo "sin estudio"

Si el cliente no tiene estudio formal:

1. `indaws-migrate detect --propose` infiere módulos custom/OCA
2. **[STOP]** Agente lanza `AskUserQuestion` en batch por grupo (mismo tipo de decisión)
3. Genera `estudio.md` desde las respuestas via `indaws-migrate gen-estudio --file decisions.json`
4. `load-estudio` para parsear → `estudio_normalizado.yaml`

## FASE 1.5 — Init contextual + backup inmutable

```bash
indaws-migrate init --client "Pharmactive" --database pharmactive_v18
```

- Crea `migration_kit_pharmactive/` con estructura canónica (docs, sql, logs, iterations, backup, dumps)
- Infiere `to_version`, `edition`, `countries`, `features` desde la BD
- `from_version` queda vacío con WARNING REQUIRED — **[STOP] preguntar al usuario**
- Genera `migration.yaml` borrador

**Acto seguido** crear el backup inmutable FUERA del workdir (FASE 0.6 lo dejó pendiente porque el workdir aún no existía):

```bash
export DUMP_PATH=/mnt/nas/clientes/pharmactive   # o cualquier path local/NAS
mkdir -p $DUMP_PATH
python3 <SKILL_DIR>/scripts/restore_dump.py backup pharmactive_v18 --output $DUMP_PATH/original.dump
# → $DUMP_PATH/original.dump (pg_dump -Fc, una sola vez)
```

> El dump NO vive en el workdir. El workdir es versionable; los dumps no.

Leer `references/analysis.md` para SQL queries de inventario complementario (counts, balance contable, FKs rotas).
