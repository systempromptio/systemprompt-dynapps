# Agent Tree — Sub-agentes y delegaciones

Catálogo de `Agent()` que la skill invoca durante el flujo de migración.
Cada entrada incluye **prompt mínimo + contract de salida**. Los prompts
deben ser **self-contained**: nunca referenciar variables del orquestador
que el sub-agente no puede ver.

> **Regla**: inyecta SIEMPRE versión Odoo, ruta del workdir y contexto del
> cliente. El sub-agente no comparte tu contexto.

---

## Patterns (recordatorio)

- **Pattern A** — Delegación a skill hermana (ejecuta otra skill como sub-agente).
- **Pattern B** — Sub-agente lector (lee N archivos pesados, devuelve resumen estructurado).
- **Pattern C** — Sub-agente post-action (valida resultado de una operación ya ejecutada).

---

## 1. `estudio-parser` (FASE 1 — Pattern B)

**Cuándo**: el usuario entrega el estudio previo del consultor (md/docx/pdf, suele tener 30-80 páginas).

```python
Agent(
    model="sonnet",
    description="Parse estudio del consultor",
    prompt=f"""
Lee el documento en `{estudio_path}` (formato {ext}) — estudio previo de
migración Odoo v{from_version} → v{to_version} para cliente `{cliente}`.

Tarea: extraer la decisión funcional por módulo y los constraints globales
del proyecto.

Devuelve JSON con este schema:
{{
  "modules": [
    {{
      "name": "<technical_name>",
      "action": "KEEP|REPLACE_WITH|CORE_REPLACE|STUDIO|NEUTRALIZE|TO_DROP|TBD",
      "successor": "<name o null>",
      "reasoning": "<resumen de 1 línea del consultor>"
    }}
  ],
  "constraints": ["<deadline, multi-company, idiomas, etc.>"],
  "blockers_funcionales": ["<decisiones pendientes del cliente>"]
}}

Si una decisión no aparece en el estudio, márcala `action: "TBD"` (no inventes).
"""
)
```

---

## 2. `db-detector` (FASE 3.1 — Pattern B)

**Cuándo**: tras restaurar el dump, antes de compilar el plan.

```python
Agent(
    model="sonnet",
    description="Escanear BD restaurada",
    prompt=f"""
Conecta a Postgres `{pg_host}:{pg_port}` BD `{db}` (Odoo v{from_version}
recién restaurada). Sin escribir nada.

Tarea: inventariar módulos installed y patrones que coinciden con las
recetas del kit `indaws_migration_kit`.

Queries clave:
- `SELECT name, state, latest_version FROM ir_module_module WHERE state='installed';`
- Patrones P1-P8 (ver `references/helpers-catalog.md`): cron rotas,
  unowned_views, jsonb_company_dependent, ir_model_data huérfanos, etc.

Devuelve JSON:
{{
  "installed": [{{name, version, author}}],
  "patterns": [{{id: "P5", count: 12, target_helper: "post_restore_validator"}}],
  "suggested_actions": [<lista de fixes auto-detectados>]
}}
"""
)
```

---

## 3. `iter-parser` (FASE 3.2 — Pattern B)

**Cuándo**: tras `indaws-migrate iterate` que falla con N errores.

```python
Agent(
    model="sonnet",
    description="Parse iter logs",
    prompt=f"""
Lee `{workdir}/iterations/{latest}/parse.json` y
`{workdir}/iterations/{latest}/upgrade.log`.

Cliente: `{cliente}` (Odoo v{from_version}→v{to_version}).

Tarea: agrupar errores por `pattern_id`, deduplicar (un patrón aplicado N
veces cuenta una vez), proponer cambios al `migration.yaml`.

Devuelve JSON:
{{
  "errors": [{{pattern_id, count, sample_traceback, affected_files: [...]}}],
  "proposed_yaml_changes": [
    {{"path": "fixes.post_restore_validator.delete_unowned_views",
      "value": true, "confidence": 0.8, "reason": "..."}}
  ],
  "manual_review": [<errores sin pattern_id reconocido>]
}}
"""
)
```

---

## 4. `suggest-validator` (FASE 3.2 — Pattern C)

**Cuándo**: tras `indaws-migrate suggest --apply` editar el yaml.

```python
Agent(
    model="sonnet",
    description="Validar yaml tras suggest",
    prompt=f"""
Acabamos de aplicar sugerencias automáticas al `{workdir}/migration.yaml`.

Antes (commit anterior): `{git_sha_before}`
Después: estado actual del fichero.

Tarea: validar que (a) el yaml sigue siendo parseable, (b) los flags
añadidos son coherentes con `fixes:` previos (no contradictorios), (c) la
próxima `iterate` tiene probabilidad alta de pasar los patrones tratados.

Devuelve JSON:
{{
  "yaml_valid": true,
  "conflicts": [<flags contradictorios o duplicados>],
  "next_iterate_likely_passes": "high|medium|low",
  "manual_check_needed": [<entradas que requieren ojo humano>]
}}
"""
)
```

---

## 5. `module-port` (FASE 3.3 — Pattern A, delegación)

**Cuándo**: módulo con `action: KEEP` o sucesor `REPLACE_WITH` que necesita
port v_X → v_Y.

```python
Agent(
    model="opus",
    description=f"Port {modulo} v{from}→v{to}",
    prompt=f"""
Invoke skill 'odoo-module-upgrader'.

Tarea: portar el módulo `{modulo}` de Odoo v{from_version} a v{to_version}.

- Fuente: `{source_path}` (módulo original en v{from_version})
- Destino: `{workdir}/migrated_modules/{modulo}/`
- Rol en la migración: `{role}` (KEEP standalone | successor de REPLACE_WITH `{predecessor}`)
- Cliente: `{cliente}` — restricciones funcionales: `{contexto_funcional}`

Aplica los patrones de port estándar v{from}→v{to} (Python, XML, JS,
manifest). Si algún archivo no tiene patrón conocido, deja TODO con marker
`# TODO-PORT-v{to}`.

Al terminar, devuelve:
{{
  "ported": true,
  "files_changed": [<lista de paths relativos al módulo>],
  "manifest_diff": "<resumen del nuevo __manifest__.py>",
  "smoke_status": "ok|warn|error",
  "todos_pendientes": [<si quedaron TODOs>]
}}
"""
)
```

---

## 6. `compare-runner` (FASE 4 — Pattern B)

**Cuándo**: tras validación funcional, comparar BD migrada vs BD de referencia.

```python
Agent(
    model="sonnet",
    description="Compare 8-level",
    prompt=f"""
Ejecuta `indaws-migrate compare --reference {ref_dump} --workdir {workdir}`
y procesa el output JSON.

Los 8 niveles son: schema, módulos, vistas, datos maestros, transaccional,
configuración, seguridad, performance.

Tarea: agrupar diffs por severidad y bloqueos para go-live.

Devuelve JSON:
{{
  "level_1_schema": {{status, diffs: [...]}},
  ...
  "level_8_perf": {{status, diffs: [...]}},
  "blockers": [<diffs que impiden go-live>],
  "cosmetic": [<diffs aceptables>],
  "summary": "PASS|WARN|FAIL"
}}
"""
)
```

---

## 7. `smoke-validator` (FASE 4 — Pattern C)

**Cuándo**: tras `indaws-migrate run --apply` final.

```python
Agent(
    model="sonnet",
    description="Validar run --apply final",
    prompt=f"""
Acabamos de ejecutar `indaws-migrate run --plan {workdir}/migration.yaml
--apply`. Cliente `{cliente}`, Odoo v{to_version}.

Inputs disponibles:
- `{workdir}/audit.sqlite` — estado de cada step del pipeline
- `{workdir}/logs/run-{ts}.log` — log completo
- `{workdir}/iterations/{latest}/smoke_rpc.json` — smoke automático del kit

Tarea: confirmar que cada step declarado en el plan se completó OK, listar
los que fallaron, y reportar el resumen del smoke_rpc.

Devuelve JSON:
{{
  "steps_total": N,
  "steps_ok": N,
  "steps_failed": [{{step_id, error, retryable}}],
  "rpc_smoke": {{models_ok, models_fail}},
  "ready_for_compare": bool
}}
"""
)
```

---

## 8. `gate-checker` (FASE 5 — Pattern B)

**Cuándo**: antes de `indaws-migrate go-live`.

```python
Agent(
    model="sonnet",
    description="Gate readiness check",
    prompt=f"""
Cliente `{cliente}` está candidato a go-live. Verifica los 8 gates de
salida.

Inputs:
- `{workdir}/gates.json` — acks registrados
- `{workdir}/playbook.md` — timeline de la migración
- `{workdir}/audit.sqlite` — última ejecución
- `{workdir}/migration.yaml` — plan final

Los 8 gates: backup, compile, iterate-clean, validation-consultor,
compare-pass, smoke-pass, credentials-ready, communication-sent.

Devuelve JSON:
{{
  "gates": [
    {{"id": "backup", "status": "ack|pending|blocked", "blocker": "<si aplica>"}}
  ],
  "can_go_live": bool,
  "outstanding_actions": [<lista de acciones pendientes ordenadas por prioridad>]
}}

NO ejecutes `go-live` — solo evalúa.
"""
)
```

---

## Delegaciones (Pattern A) a otras skills

### `odoo-studio-migration`

**Cuándo**: FASE 2, módulo con `action: STUDIO` o ambiguo.

```python
Agent(
    model="opus",
    description=f"Studio fit-check {modulo}",
    prompt=f"""
Invoke skill 'odoo-studio-migration'.

Cliente `{cliente}`, módulo `{modulo}` (Odoo v{from}→v{to}). El consultor
propone migrarlo a Studio.

Tarea: determinar si la funcionalidad cabe en Studio (campos manuales x_*
+ vistas configurables + automated actions). Si SÍ, devolver `studio_spec`.
Si NO, decir por qué (JS custom, ORM avanzado, reports complejos, etc.).

Devuelve JSON:
{{
  "viable": bool,
  "studio_spec": {{fields: [...], views: [...], actions: [...]}},
  "blockers": [<si no viable>]
}}
"""
)
```

### `odoo-pilot`

**Cuándo**: FASE 3.3 post-port + FASE 4 smoke RPC.

```python
Agent(
    model="sonnet",
    description=f"RPC validate {modulo}",
    prompt=f"""
Invoke skill 'odoo-pilot' contra `{odoo_url}` (db `{db}`, user `{user}`).

Tarea: validar que el módulo `{modulo}` está installed, sus campos
`x_studio_*` declarados en `studio_spec` existen en el modelo `{model}`, y
los registros migrados desde `{legacy_column}` tienen valor.

Devuelve JSON:
{{
  "installed": bool,
  "fields_present": [{{name, ttype, ok}}],
  "records_migrated": N,
  "errors": [<si alguna validación falla>]
}}
"""
)
```

### `common-skills:docker-deployment`

**Cuándo**: FASE 0.6 si el cliente no tiene `docker-compose.yml` previo.

```python
Agent(
    model="sonnet",
    description="Bootstrap docker env",
    prompt=f"""
Invoke skill 'docker-deployment'.

Cliente `{cliente}`, target Odoo v{to_version}. Dump disponible en
`{DUMP_PATH}/original.dump`.

Tarea: generar `docker-compose.yml` + restaurar el dump en el contenedor,
asegurando que el filestore queda accesible.

Devuelve:
- Path al `docker-compose.yml` generado
- Nombre del contenedor `odoo` y `db`
- Confirmación de que `pg_restore` terminó OK
"""
)
```

---

## Reglas comunes a TODOS los prompts

1. **Inyectar versión Odoo** (`from_version`, `to_version`) — el sub-agente NO la conoce.
2. **Inyectar workdir absoluto** — nunca usar rutas relativas.
3. **Inyectar cliente** — algunos sub-agentes leen `~/.claude/projects/<cliente>/memory/`.
4. **Pedir JSON estructurado**, no texto libre — facilita parsear el resultado.
5. **Prohibir efectos secundarios** salvo que el sub-agente sea Pattern A
   sobre una skill ejecutora (port, docker-deployment).
6. **Limitar lectura**: si el agente solo necesita N archivos, listarlos
   explícitamente — evita lecturas masivas innecesarias.
