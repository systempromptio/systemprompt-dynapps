---
name: indaws-hpfo-tracker
description: >-
  Captura incremental de notas para el HPFO (sync quincenal CTO & TL) y volcado a resumen
  Markdown en las 5 secciones oficiales. Usar durante el ciclo para ir apuntando cosas segun
  pasan (logros, prioridades, fuegos, oportunidades, temas de agenda) y al final para consolidar
  el resumen listo para subir al chat 24h antes de la reu. Triggers: "apunta para el HPFO",
  "nota HPFO", "esto va al HPFO", "monta el HPFO", "resumen HPFO", "cierra el ciclo HPFO",
  "que llevo del HPFO", "/hpfo".
version: 1.1.3
effort: low
model: sonnet
allowed-tools: [Bash, Read, Write]
---

# HPFO Tracker

**CONFIDENTIAL — MANAGEMENT TEAM ONLY** · Critical Management Know-How

## Important (reglas de consolidación — no las saltes)

Al montar el resumen final, SIEMPRE:
- **Bullets, no párrafos.** Resultado/decisión, no "estuvimos trabajando en".
- **Dedup y agrupa** temas relacionados; no vuelques todo si hay ruido — filtra.
- **No dupliques** un tema entre secciones (un fuego no va también en Highlights).
- **Coherencia temporal:** las Priorities de este ciclo deberían poder ser los Highlights del siguiente.
- **Agenda** como decisión pedida ("Decidir si…", "Aprobar…"). **Ignora las fechas internas** de cada bullet (del `add`, no de la reu). Sección sin contenido → "—", no inventes.

## Overview

El **HPFO** es la herramienta de ejecucion/alineacion del sync **quincenal CTO & TL**.
No es un informe de estado: es corto, factual y orientado a accion (bullets). Se rellena 24h
antes de la reu y se avisa por el chat; CTO y TL comentan de forma asincrona; la reu solo
toca lo destacable (decisiones, alineacion, escalado). Esta skill resuelve el problema real: **ir anotando durante las dos semanas** para no llegar en blanco, y **consolidar** sin que se pierda nada.

Las 5 secciones (orden fijo):
1. **Highlights (past week)** — qué se cerró/consiguió (resultado, no actividad).
2. **Priorities (next week)** — 2-4 prioridades reales atadas a entregable/cliente.
3. **Fires (risks, issues, action needed)** — riesgos, líos de equipo, bloqueos; marcar si se quiere intervención del CTO.
4. **Opportunities** — ideas que aporten a clientes/equipo (MCP, skills, dev 2.0, proceso).
5. **Suggested agenda items** — solo lo que necesita decisión/alineación/escalado en la reu.

## When to Use This Skill

- Configurar (1ª vez): "configura el HPFO" (pide y guarda las rutas una vez).
- Captura rápida en cualquier momento: "apunta para el HPFO: fuego, cliente X bloqueado". Repaso: "¿qué llevo del HPFO?".
- Consolidar (~24h antes): "monta el resumen del HPFO" (+ docx si hace falta). Cierre tras la reu: "cierra el ciclo HPFO".

## Estado y estructura

- Estado por-usuario en `~/.hpfo/state.json` (FUERA del repo): `workspace`, `template` (docx), `active_cycle`.
- Cada ciclo = carpeta `<workspace>/HPFO-<fecha-reu>/` con `HPFO - <fecha>.docx` (copia plantilla), `notes.md`, y al consolidar `HPFO-<fecha>-resumen.md`.
- Al cerrar, la carpeta queda como **histórico**; el siguiente `start` crea otra.

## Workflow

### 0. Primer uso — configurar (una vez) [STOP: preguntar al usuario]
Antes de nada, comprueba el estado:
```bash
python3 scripts/hpfo.py status
```
Si `configured=false`, **pregunta al usuario dos rutas** y guárdalas:
- `workspace`: carpeta local donde vivirán las carpetas de cada ciclo.
- `template`: ruta al `.docx` plantilla del HPFO.
```bash
python3 scripts/hpfo.py setup --workspace <dir> --template <docx>
```

### 1. Arrancar el ciclo (cada dos semanas)
```bash
python3 scripts/hpfo.py start --date <YYYY-MM-DD de la reu>
```
Crea la carpeta del ciclo y copia el docx dentro. Si ya hay ciclo activo, ciérralo antes.

### 2. Apuntar una nota (en cualquier momento)
```bash
python3 scripts/hpfo.py add <categoria> "texto conciso, orientado a resultado/accion"
```
Categorias (alias ES/EN): `highlights|logro|hecho`, `priorities|prioridad`,
`fires|fuego|riesgo|issue`, `opportunities|oportunidad`, `agenda|decision`.
- Para decidir **si algo entra y en que seccion**, aplica `references/decision-criteria.md`.
  Ante la duda, captura: el filtro fino es en la consolidacion.
- Reescribe la nota a bullet corto; no la pegues tal cual si es divagante. Si dudas de la
  categoria, pregunta en una frase.
- Fuegos con intervencion: añade `(revisar con el CTO)` al texto.
- Si no hay ciclo activo, el script avisa → arranca uno con `start` (pregunta la fecha de la reu).

### 3. Repasar
```bash
python3 scripts/hpfo.py show
```

### 4. Consolidar el resumen (juicio — lo haces tu, no el script)
1. Lee `notes.md` del ciclo (`show`).
2. Consolida por seccion aplicando las **reglas de `## Important`**.
3. Escribe `HPFO-<fecha>-resumen.md` en la carpeta del ciclo, con las 5 cabeceras en orden.
4. Muestra el resultado al usuario. **[STOP]** No montes el docx ni cierres hasta que confirme.

### 5. Montar el docx final (cuando haga falta)
```bash
python3 scripts/hpfo.py build-docx
```
Rellena las 5 celdas del docx del ciclo desde `HPFO-<fecha>-resumen.md`. Listo para subir al chat.

### 6. Cerrar ciclo (tras la reu)
```bash
python3 scripts/hpfo.py close
```
Deja la carpeta como histórico y limpia el ciclo activo. El proximo `start` crea uno nuevo.

## Rules and Guidelines

- Rutas SIEMPRE en `~/.hpfo/state.json` vía `setup`; nunca hardcodees rutas de usuario en el repo.
- `config.json` (en el repo) solo lleva defaults sugeridos (p.ej. `default_workspace`) y cadencia.
- La captura es barata: ante la duda, apunta. El filtro se hace en la consolidacion.
- Idioma del usuario (ES por defecto); cabeceras de seccion en ingles como en la plantilla oficial.

## Examples

- "configura el HPFO" → `status`; si no está, preguntar workspace+template → `setup ...`.
- "esto va al HPFO: migramos AUTOSUR a v18, validado" → `add highlights "Migracion AUTOSUR a v18 validada por cliente"`.
- "apunta un fuego: el dev nuevo va saturado, que lo vea el CTO" → `add fires "Sobrecarga dev junior Tokyo — riesgo plazos (revisar con el CTO)"`.
- "monta el HPFO para la reu del 15" → `show` → consolidar → `HPFO-...-resumen.md` → mostrar → (si piden) `build-docx`.

## GOTCHAS

1. **Un solo ciclo activo** (`active_cycle`); `start` con uno abierto falla → `close` primero (no mezcles reuniones).
2. **`add` exige ciclo activo**; si no, `start --date <reu>` antes (cada ciclo = su carpeta + copia del docx).
3. **Fechas de los bullets = día del `add`, no de la reu** → al consolidar, ignóralas.
4. **No pongas `## ` al inicio del texto de una nota** (el insert por cabecera usa regex; rompería el parsing).
5. **Estado por-usuario en `~/.hpfo/state.json`, NO en el repo** (cada TL el suyo; `config.json` del repo solo defaults).
6. **`build-docx` necesita `python-docx` + el `HPFO-<fecha>-resumen.md`**; mapea por palabra clave; si el docx no tiene la tabla de 5 secciones, avisa.
7. **La consolidación es juicio, no script** (no hay comando `consolidate`): aplicas `## Important` + `references/decision-criteria.md`.

## Delegates to
Ninguna — skill autónoma; el docx lo monta su propio `scripts/hpfo.py build-docx` (python-docx).

## Consumed by
Ninguna — skill personal/terminal del TL (dirección).

## REFERENCES
- `references/decision-criteria.md` — qué merece entrar en un HPFO y en qué sección. Consultar al capturar y consolidar.
- `references/shared/coding-standards.md` — convención de idioma (ES con tildes/ñ; cabeceras EN de la plantilla).

## Success Criteria
- Toda nota queda capturada con fecha y en su sección durante el ciclo.
- El resumen final: 5 secciones, bullets, sin duplicados ni ruido, listo 24h antes de la reu.
- Al cerrar, el ciclo queda archivado y el siguiente arranca limpio.
