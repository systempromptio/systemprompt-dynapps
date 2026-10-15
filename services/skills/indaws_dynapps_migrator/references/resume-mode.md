# Resume Mode — Dynapps Migrator

Cuando se detecta una migración en curso, la skill no empieza de cero.
En su lugar, lee el estado guardado y ofrece continuar desde el punto exacto donde se dejó.

---

## Detección de migración en curso

### 1. Comprobar sesión activa de indaws-migrate

```bash
SESSION=$(indaws-migrate playbook-session 2>/dev/null)
```

Si `$SESSION` no está vacío → migración en curso.

### 2. Buscar workdir por convención de nombres

```bash
find ~/migrations ~/odoo /tmp -maxdepth 2 -name "migration_kit_${CLIENTE}" -type d 2>/dev/null
find ~/migrations ~/odoo /tmp -maxdepth 2 -name "odoo-migkit-${CLIENTE}" -type d 2>/dev/null
```

### 3. Comprobar migration.yaml

Si existe `<workdir>/migration.yaml`:
```bash
cat <workdir>/migration.yaml | grep -E "^(phase|status|current_module):"
```

### 4. Comprobar repo odoo-migkit-`<cliente>`

```bash
git -C <workdir> log --oneline -5 2>/dev/null
git -C <workdir> status --short 2>/dev/null
```

---

## Lectura del estado actual

Una vez localizado el workdir, extraer:

```bash
# Estado global de la migración
indaws-migrate status --workdir <workdir>

# Lista de módulos con su estado actual
indaws-migrate modules list --workdir <workdir>

# Última acción ejecutada
indaws-migrate playbook-session --workdir <workdir>

# Errores pendientes
indaws-migrate errors list --workdir <workdir> 2>/dev/null
```

### Campos del migration.yaml relevantes para resume

```yaml
# Campos de estado que leer
phase: 0-5               # fase actual (0=init, 1=dump, 2=estudio, 3=detect, 4=compile, 5=iterate)
status: in_progress      # in_progress / paused / error / completed
current_module: <nombre> # módulo en proceso si estamos en el loop
last_stop: 4             # último STOP point alcanzado
maintenance_window: ~    # si está fijada
client_notified: false   # gate 8 del bloque 5

modules:
  <modulo_1>:
    status: smoked        # pending / porting / ported / iterating / smoked / error
    branch: migration/v<version_destino>-<modulo_1>
    mr_url: <url>
    todos_pending: 0
  <modulo_2>:
    status: porting
    branch: migration/v<version_destino>-<modulo_2>
    todos_pending: 3
```

---

## Determinación del bloque de retoma

Lógica de mapeo estado → bloque de retoma:

```
phase == 0 AND status == "error"
  → BLOQUE 1 (algo falló en init)

phase == 0 AND status == "in_progress"
  → BLOQUE 2 inicio (init a medias)

phase IN [1, 2, 3, 4] AND status == "in_progress"
  → BLOQUE 2 (BD side, FASE correspondiente)

phase == 5 AND current_module != null AND status == "in_progress"
  → BLOQUE 3, módulo current_module

phase == 5 AND todos los módulos en "smoked"
  → BLOQUE 4 (validación)

last_stop == 8 AND algún módulo en "error" post-validación
  → BLOQUE 3, módulos marcados como error

last_stop == 8 AND todos "smoked"
  → BLOQUE 5 (go-live)
```

---

## Mensaje de oferta de resume

Una vez determinado el estado, presentar al usuario:

```
Resume Mode — Migración en curso detectada

Encontré una migración activa de <cliente>:

  Workdir: <path>
  Fase actual: BLOQUE <N> — <descripción>
  Último STOP alcanzado: STOP <X>
  
Estado de módulos:
  [✓] smoked: <módulo_1>, <módulo_2>  (N módulos)
  [→] en curso: <módulo_actual>
  [ ] pendientes: <módulo_3>, <módulo_4>  (N módulos)
  [!] con errores: <módulo_error>  (N módulos)

Última sesión: <fecha y hora>

¿Qué quieres hacer?
  a) Continuar desde donde se dejó (recomendado)
  b) Retomar desde un bloque específico (indica cuál)
  c) Ver el estado completo (módulos list + errors)
  d) Abandonar y empezar de cero (destructivo — pide confirmación)
```

---

## Retoma desde bloque específico

Si el usuario pide retomar desde un bloque concreto:

```
"Retoma desde el Bloque 3, módulo tecniman_sale_custom"
→ Saltar a BLOQUE 3.2, con current_module = tecniman_sale_custom
→ Leer el estado del módulo: branch existente / TODOs / errores previos

"Retoma desde el STOP ④"
→ Re-ejecutar detect → presentar módulos sin decisión → STOP ④

"Retoma desde el Bloque 4"
→ Verificar que todos los módulos están en "smoked"
→ Si no → advertir y preguntar si forzar igualmente
→ Si sí → lanzar BLOQUE 4 (smoke RPC + compare)
```

---

## Abandono y reinicio

Solo con confirmación explícita:

```
¿Quieres realmente empezar de cero?

Esto eliminará:
  - Workdir: <path> (contiene migration.yaml, logs, módulos portados)
  - Repo: odoo-migkit-<cliente> (en GitHub/GitLab)
  - BD de trabajo: <cliente>_v<version_destino>_migration

Esta acción es IRREVERSIBLE. Escribe "confirmo borrar todo" para continuar.
```

Si confirma:
```bash
indaws-migrate destroy --workdir <workdir> --confirm
# Luego reiniciar desde BLOQUE 0
```

---

## Guardado de contexto durante la ejecución

La skill guarda el contexto de progreso en dos lugares:

### 1. migration.yaml (gestionado por indaws-migrate)
Actualizado automáticamente por los comandos de indaws-migrate.

### 2. Nota de estado en el último commit del repo migkit

Tras cada STOP point completado, crear un commit de checkpoint:
```bash
git -C <workdir> commit --allow-empty -m "checkpoint: STOP <N> completed — $(date -u +%Y-%m-%dT%H:%M:%SZ)"
```

Esto permite saber en qué punto exacto se dejó incluso si migration.yaml se corrompe.

> **Modo autónomo**: como entre STOPs hay tramos largos sin intervención, el checkpoint se
> hace también tras **cada fase completada** (buildout OK, upgrade descargado, neutralización
> verificada, `-u all` limpio, cada módulo `smoked`). Así una corrida autónoma interrumpida
> reanuda desde el último hito sin rehacer. Ver `autonomous-mode.md`.

### 3. Log de sesión

```bash
~/.dynapps/migrations/<cliente>/session.log
```

Formato:
```
2025-01-15T10:23:00Z STOP_④ completed: estudio cargado desde ~/estudio_tecniman.xlsx
2025-01-15T11:45:00Z STOP_⑤ completed: 12 módulos con decisión
2025-01-15T14:30:00Z BLOQUE_3 started: 8 módulos en orden topológico
2025-01-15T16:00:00Z MODULE tecniman_sale_custom: smoked
2025-01-15T16:30:00Z STOP_⑦ triggered: tecniman_report_custom tiene 3 TODOs
```
