---
name: indaws-dev2-adoption-report
version: 0.1.3
effort: low
model: sonnet
description: >-
  Informe de adopción del estándar "Desarrollador 2.0" a partir de señales de git (fase 1).
  Mide qué skills se usan de verdad (marca `Made with` en el código generado) y quién
  contribuye al ecosistema (commits que tocan skills y gotchas, por autor). Sirve para
  calibrar el umbral de entrega con datos y hacer visible la adopción/contribución.
  Triggers: "informe de adopción", "telemetría de skills", "quién usa las skills",
  "quién aporta gotchas", "adopción desarrollador 2.0", "métrica de skills".
---

# dev2-adoption-report

Telemetría de adopción del **Desarrollador 2.0** (ver `references/shared/delivery-standard.md`).
Fase 1 = **solo señales de git**; el % de cumplimiento por tarea (leyendo `project.task`) se añade después.

## Workflow

1. **Configurar roots** — `config.json`: `code_roots` = buildouts de cliente donde vive el código
   generado (ahí se cuenta la marca `Made with`); `skills_root` = ecosistema de skills.
2. **Ejecutar** — `python3 scripts/adoption_report.py [--since "90 days ago"]` → informe Markdown.
3. **Presentar** las dos señales:
   - **Adopción**: `Made with <skill> · vX` por skill en el código generado. (Arranca en 0 — la marca
     es reciente; el 0 es la línea base y crece según se usan las skills.)
   - **Contribución**: commits por autor tocando `skills/` (mejoras) y `*gotchas*` (conocimiento).
4. **Calibrar** (opcional) — con la serie histórica, proponer mover el corte en
   `references/shared/delivery-thresholds.json` si procede (ver §recalibration del JSON).

## GOTCHAS

1. **La marca NO se cuenta en el repo de skills** — ahí `Made with` aparece como convención/ejemplo
   (falso positivo). El script solo escanea `code_roots` y exige `· vN` real (excluye `· v<version>`).
2. **Adopción 0 al principio es correcto** — la convención de procedencia es nueva; no hay código
   generado con ella todavía. Es la línea base, no un fallo.
3. **Identidades git ruidosas** — un mismo dev aparece con varios nombres (`Toni`, `Toni Hermosilla`,
   `Antonio Hermosilla`). Fase 1 no des-duplica; interpretar agrupando. (Mejora futura: alias-map.)
4. **Commits ≠ PRs** — fase 1 cuenta commits (git puro); sustituir por PRs cuando se integre `gh`.
5. **`code_roots` vacío/erróneo → adopción 0 silenciosa** — el informe avisa si no hay roots escaneados.

## NEVER
1. Escanear `skills_root` buscando la marca de adopción (falso positivo por la convención).
2. Exponer los conteos como ranking punitivo — es señal de adopción, no vara de medir personas.

## ALWAYS
1. Apuntar `code_roots` a los buildouts de cliente reales antes de sacar conclusiones de adopción.
2. Reportar la ventana temporal (`--since`) y agrupar identidades git del mismo dev al interpretar.

## Delegates to

| Skill | Suite | Para qué |
|-------|-------|----------|
| — | — | Standalone (git-only). Fase 2 delegará en `odoo-pilot` para leer `project.task`. |

## Consumed by

| Skill | Suite | Para qué |
|-------|-------|----------|
| executive-projects-tracker | management | Señal de adopción/contribución para el reporting de dirección |

## REFERENCES

- `scripts/adoption_report.py` — motor (grep de marca + git log por autor)
- `config.json` — `code_roots`, `skills_root`, `since`
- `references/shared/delivery-standard.md` — el estándar que esto mide
- `references/shared/delivery-thresholds.json` — umbral a calibrar con estos datos
- `references/shared/skill-provenance-marker.md` — la marca `Made with` que se cuenta
