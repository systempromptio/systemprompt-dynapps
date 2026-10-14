# Fase 1-2 — Triaje de módulos: alcance y profundidad

## El alcance lo manda la BD: solo lo INSTALADO

No se audita lo que hay en disco, sino lo que está **instalado** (`ir_module_module.state IN
('installed','to upgrade')`). En disco suele haber módulos no instalados y árboles enteros de
OCA sin usar. `scripts/module_triage.sh` (solo lectura) lista los instalados y los clasifica.

## Clasificación por AUTOR, no por carpeta

La carpeta engaña: hay módulos OCA vendorizados dentro del repo "custom". La señal fiable es
`ir_module_module.author`:

| Clase | Criterio (author) | Qué se hace |
|-------|-------------------|-------------|
| **STANDARD** | `Odoo S.A.` | Se **ignora** (es core) |
| **OCA / comunidad** | contiene `Community Association`, `(OCA)`, `Localization Team` | **Solo continuidad** (no auditoría exhaustiva) |
| **CUSTOM** | el resto (partner anterior, cliente) | **Auditoría EXHAUSTIVA** + va a `odoo_studio` lo de Studio |

Patrones en `config.json → module_triage.author_class`. Los **autores ambiguos** (p. ej.
`Camptocamp / Odoo`, integradores mixtos) los **confirma el principal (opus)** caso a caso; la
heurística solo prioriza.

## Profundidad por clase

- **CUSTOM + Studio (`odoo_studio`)** → `odoo-code-auditor` exhaustivo del **estado actual**:
  Calidad/Deuda, Seguridad, Rendimiento, Convenciones. Es el núcleo del informe. (Migrabilidad =
  extra opcional, ver abajo.)
- **OCA → estado actual**: versión instalada y si está mantenido. *(EXTRA opcional)* continuidad
  contra la última versión (`config.json → target_version_default`): ¿continúa la rama?,
  ¿absorbido por el core?, ¿descontinuado? (`odoo-version-api-reference` + `odoo-classification-guide`).
- **STANDARD** → no se toca.

## Instalado SIN fuente = riesgo

Cruzar los instalados (no estándar) con **todas** las rutas de addons del cliente. Si un módulo
instalado no tiene fuente en ninguna → o es Enterprise estándar (normal) o **se perdió el código**
(riesgo serio para la migración). `module_triage.sh --src <ruta>` lo marca; pásale todas las
rutas (custom **y** OCA), no solo una, para no generar falsos positivos.

**Pero "sin fuente" ≠ caja negra.** Para los CUSTOM sin código en disco, correr
`scripts/reconstruct_custom.sh` (por SQL, sin Odoo): reconstruye modelos, campos (con
ttype/relation/related/compute/store/depends + selection), vistas (`arch_db`), seguridad, acciones
(con su código Python), menús, crons, informes y plantillas de correo → `custom_reconstruct/<mod>/FOOTPRINT.md`.
Y `scripts/db_intelligence.sh` para dimensionar (dependencias, complejidad, fill-rate, actividad).
Se recupera ~toda la capa declarativa; solo el Python imperativo queda opaco. Esto alimenta la
sección 03/05 incluso sin fuente (con la salvedad de que la salud/calidad no se puntúa sin código).

## Puntuación 0-10 por módulo (OBLIGATORIO)

**Núcleo — SALUD del módulo (estado actual), 0-10** (10 = impecable, 0 = insostenible). Combina
deuda técnica, seguridad, rendimiento y convenciones (score de `odoo-code-auditor` →
`report-generation.md`). Es lo que mide el Rescue Pack: cómo está el código **hoy**. El informe
DEBE mostrarla por módulo, no solo SIMPLE/STANDARD/COMPLEX.

Guía: 9-10 limpio · 7-8 sano con detalles · 5-6 deuda moderada · 3-4 problemas serios · 0-2
insostenible/reescritura.

**EXTRA (opcional) — dificultad de MIGRACIÓN, 0-10** (10 = reescritura). Solo si el cliente pide
el plus de versión. Es otra escala distinta (Migration Complexity Score, `migrability-checklist.md`):
0-2 trivial · 3-4 cambios mecánicos · 5-6 hooks de áreas reorganizadas · 7-8 API inestable · 9-10
reescritura. Para OCA no se puntúa: continuidad (continúa / absorbido / descontinuado).

## Salida

`MODULE_TRIAGE.md` con: resumen por clase, lista CUSTOM **con su 0-10**, lista OCA (continuidad)
e instalados-sin-fuente. Alimenta la **sección 03** y define el trabajo de la **sección 05/07**.
