# Tracking de módulos (status + blockers)

A partir de `indaws_migration_kit` v2.3.0, el plan declara también
**dónde está cada módulo**, no solo qué hacer con él.

---

## Estados y transiciones

Cada `ModuleDecision` tiene un campo `status` (default `pending`) y una lista
opcional `blockers`. Los 8 estados:

| Estado | Significa |
|---|---|
| `pending` | Decisión tomada, port sin empezar |
| `porting` | Dev trabajando el código v_destino |
| `code_ready` | Código portado existe en `migrated_modules/`, sin probar |
| `verified` | `-u all` lo instala sin error |
| `smoked` | Smoke RPC + UI OK para los modelos del módulo |
| `done` | Consultor firma OK funcional |
| `blocked` | Esperando algo (decisión cliente, dependencia, info) |
| `skipped` | NEUTRALIZE/REMOVE/STUB/STUDIO — no requiere port real |

**Auto-skip**: si declaras `action: NEUTRALIZE|REMOVE|STUB|STUDIO`, el status se
fuerza a `skipped` al cargar el yaml. Puedes sobrescribirlo manualmente.

**Transiciones permitidas**: solo hacia delante (pending → porting → code_ready
→ verified → smoked → done). Cualquier estado puede ir a `blocked`. `blocked`
puede ir a cualquier estado.

---

## Comandos CLI

```bash
# Tabla con barra de progreso
indaws-migrate modules --workdir . list

# Filtros
indaws-migrate modules --workdir . list --filter-status pending
indaws-migrate modules --workdir . list --has-blockers

# Orden topológico por depends (lee manifests de migrated_modules/ + OCA/...)
indaws-migrate modules --workdir . order

# Cambiar status (con validación de transición)
indaws-migrate modules --workdir . set sale_order_type --status verified \
    --note "iter 003 OK"

# --force salta validación (uso restringido)
indaws-migrate modules --workdir . set foo --status done --force

# Gestionar blockers
indaws-migrate modules --workdir . blocker foo add --text "esperando feedback"
indaws-migrate modules --workdir . blocker foo remove --text "esperando feedback"
indaws-migrate modules --workdir . blocker foo clear

# Historial de transiciones (todos los módulos, o uno)
indaws-migrate modules --workdir . history
indaws-migrate modules --workdir . history sale_order_type

# Report markdown para playbook/cliente
indaws-migrate modules --workdir . report --output docs/progress.md
```

---

## Auto-update (transiciones híbridas)

El runner + iterate + smoke actualizan status automáticamente al observar
eventos no destructivos:

| Evento del runner | Transición auto |
|---|---|
| `manifest_appeared` (manifest portado aparece) | `pending` → `code_ready` |
| `u_all_ok` (módulo instala sin error en `-u all`) | `code_ready` → `verified` |
| `smoke_rpc_ok` (smoke RPC pasa para sus modelos) | `verified` → `smoked` |
| `smoke_rpc_failed` (smoke falla) | `smoked` → `blocked` |

Transiciones **comprometidas** (siempre manuales):
- `smoked → done` (firma del consultor)
- Cualquier transición a `blocked` con motivo
- `blocked → cualquiera` (resolución del blocker)

Cada cambio queda registrado en:
- `migration.yaml` (estado actual, vive en git)
- `audit.sqlite` tabla `module_status_history` (historial completo, gitignored)
- `playbook.md` Bitácora (auto-log vía `PlaybookHook.on_module_status_change`)

---

## Dónde vive el estado (recap)

Híbrido (opción A + C):

| Información | Dónde | Versionable |
|---|---|---|
| `status` actual | `migration.yaml` modules[X].status | Sí (git) |
| `blockers` | `migration.yaml` modules[X].blockers | Sí (git) |
| Historial de transiciones | `audit.sqlite` module_status_history | No (gitignored, regenerable solo de auditoría local) |

**Por qué no audit-only**: el otro dev al clonar el workdir necesita saber
dónde está cada módulo. Si lo guardas solo en audit (gitignored), empieza
desde cero.

**Por qué no yaml-only**: el historial de 50 transiciones por módulo
contaminaría el yaml. Audit es mejor sitio para "cómo llegamos aquí".

---

## Cómo se ve en el playbook

Tras varios iterates, la bitácora del `playbook.md` acumula entradas como:

```
- **2026-05-19 14:30** — → `sale_order_type`: code_ready → verified [auto_u_all_ok]
- **2026-05-19 15:12** — → `custom_pnt`: verified → smoked [auto_smoke_rpc_ok]
- **2026-05-19 15:45** — → `custom_pnt`: smoked → blocked [auto_smoke_rpc_failed]
- **2026-05-19 16:20** — → `custom_pnt`: blocked → smoked [manual]
```

Permite reconstruir la timeline de la migración sin SQL.

---

## Orden topológico

`indaws-migrate modules order` lee los `__manifest__.py` (vía `context.source_repos`)
y construye un DAG. Devuelve los módulos por **tiers** topológicos:

```
Tier 1 (sin deps internas):
  · base_uncontrolled_pnt        [pending]    · 14 dependen
  · security_access_menu_pnt     [pending]    · 8 dependen
  · eq_restrict_action_option    [pending]

Tier 2 (depende solo de Tier 1):
  · user_agent_pnt               [porting]    · 5 dependen
  ✓ tecniman_supplier_default    [verified]
```

**Tie-breaker dentro de un tier**: los módulos con más **inverse-deps** (más
módulos dependen de ellos) van primero. Así se atacan los "fundamentos"
del tier antes que los aislados.

**Ciclos**: si el dependency_resolver detecta un ciclo, sale con error y
lista los módulos involucrados.

---

## Cuándo usar cada cosa

| Necesidad | Comando |
|---|---|
| Empezar la sesión, ver dónde dejé las cosas | `modules list --filter-status pending` |
| Decidir qué portar a continuación | `modules order` |
| Marcar avance manual del dev | `modules set X --status code_ready` |
| Bloquear hasta decisión | `modules blocker X add --text "…"` |
| Entregar informe al cliente | `modules report --output docs/progress.md` |
| Investigar regresión | `modules history X` |
