# Dynapps Migrator — Workflow detallado (BLOQUE 0 → 5)

5 bloques: prerequisitos → upgrade BD (Upgrade Service + iterate) → migración de
código → validación → go-live. El dev solo interviene en los STOP points definidos.

> **Nota go-live**: la subida del dump resultante al servidor de producción
> se realiza manualmente (fuera del flujo automatizado). Ver BLOQUE 5.

## Escenario y modo autónomo

Antes del BLOQUE 1 el orquestador ya resolvió `scenario` (`references/scenarios.md`) e
imprimió los requerimientos (`references/requirements.md`). BLOQUES 1 → 2 → 3 corren de
forma **autónoma** hasta el hito **"Entorno local listo"**, parando solo en gates
irreducibles (`references/autonomous-mode.md`).

**Hito "Entorno local listo" (fin de la corrida autónoma, dentro de BLOQUE 3):**
- BD upgraded → restaurada → **neutralizada+verificada** → `-u all` limpio (Docker 5433).
- Código custom portado en orden topológico, `installable` restaurado, scripts de
  migración presentes (incl. `end-reactivate_views.py`), módulos cargan con `-u` **sin
  ERROR/WARNING**.
- `odoo-bin` arranca contra la BD migrada + smoke RPC verde → URL `http://localhost:8069`.
- **Reproducible por terceros**: otra persona levanta el entorno con solo el código migrado en
  GitLab/GitHub + la BD del Upgrade Service; todo commiteado, sin parches locales (gate en
  `autonomous-mode.md`).
- **FUERA del hito**: MRs, validación funcional (⑧) y go-live (⑨). BLOQUE 3 llega hasta
  ANTES de crear MRs (STOP ⑦).

> **Default de decisiones — paridad 1:1**: ante features que cambian en la v-destino, mantener
> el comportamiento del cliente 1:1 por defecto; desviarse = decisión de negocio explícita.
> Aplica a STOP ④, STOP ⑥ y a la clasificación de paridad. Ver `autonomous-mode.md`.

## Paralelización (importante)

El **poste largo** es el Upgrade Service (BLOQUE 2): minutos a horas de espera pasiva. El
principio es que **el track de código NO depende del dump migrado** (portar es trabajo contra
el *source* de la v-destino), así que solapa entero esa espera. El **punto de convergencia**
es el **iterate loop**, lo único que necesita las dos cosas a la vez: BD migrada+neutralizada
Y módulos portados.

```
TRACK BD:     enviar → [ESPERA] → descargar → restaurar → NEUTRALIZAR ┐
TRACK CÓDIGO: buildout → rebase → orden topológico → PORTAR módulos ──┤► iterate -u all → smoke
```

### Nivel 1 — BD ⟂ Código (el grande)

Todo el track de código adelanta durante la espera. NO limitarlo a análisis+rebase: incluye
**el porting de los módulos**, que es el bulto.

| Trabajo paralelo (durante la espera del ZIP) | Depende del dump migrado |
|---|---|
| Crear/verificar buildout v_destino + repo | No → adelantar ya |
| Integrar histórico git + rebase sobre `develop` v_destino | No → adelantar ya |
| Análisis de módulos + orden topológico | No → adelantar ya |
| **Portar los módulos custom** (editar código a la v-destino) | No → adelantar ya |
| Verificar provider git (`glab`/`gh`) | No → adelantar ya |
| Restaurar + **neutralizar** + iterate loop | **Sí** → esperar ZIP + convergencia |

### Nivel 2 — Módulos independientes en paralelo (fan-out)

El orden es topológico (`<prefijo>_base` antes que sus dependientes), pero los módulos del
**mismo nivel topológico** (sin dependencia entre ellos) se **portan en paralelo** con varios
workers (subagentes/worktrees), uno por módulo. Reglas:
- Serializar por **niveles** topológicos: no arrancar un nivel hasta que el anterior está portado.
- El **`-u` de verificación** contra la BD Docker compartida (5433) **se serializa** — o cada
  worker usa su **BD scratch** propia. Nunca dos `-u` concurrentes sobre la misma BD.
- Es **opt-in** (más coste/tokens). Por defecto, port secuencial en orden topológico.

### Nivel 3 — Verificación en dos capas (adelantar la barata)

- **Install-sobre-limpia** (`-i <modulo>` sobre BD v-destino vanilla): caza errores de
  sintaxis/API/manifest. **NO necesita el dump migrado** → adelantar durante la espera.
- **Iterate-sobre-migrada** (`-u` contra el dump migrado): caza errores de **datos**/migración.
  Es la convergencia → tras llegar el ZIP y neutralizar.

Un módulo que ya pasó install-sobre-limpia llega a la convergencia con solo los fallos de
datos por resolver, no los de código.

### Consulta preventiva antes de converger

Antes del `-u all` de convergencia, **volver a consultar el catálogo de errores comunes**
(`dynbrain-feedback-loop.md`): es el punto donde históricamente aparece la mayoría, y las
fichas traen la medida preventiva y la consulta de detección.

### Regla operativa

**Lanzar el upgrade en background con un vigilante que notifique al terminar**; mientras,
ejecutar Niveles 1–3. Comprobar el estado del upgrade periódicamente; no espera pasiva. Al
llegar el ZIP → converger: restore → **gate de neutralización** → iterate `-u all` (con los
módulos ya portados y pre-verificados) → smoke.

**Serie inevitable** (no paralelizable): la neutralización y el iterate `-u all` final (una
sola BD, gate de seguridad).

## BLOQUE 0 — Detección de estado
Detectar qué existe y qué falta. Enrutar al bloque correcto.
Ver `references/block-routing.md`.

## BLOQUE 1 — Preparación local
- Máquina sin prerequisites → `developer-skills:dynapps-dev-bootstrap` → **STOP ①**
- Buildout no existe → `developer-skills:dynapps-local-env-setup` → **STOP ②**
- AUTO: levantar contenedor Docker PostgreSQL en 5433 para iterate loop

## BLOQUE 2 — Upgrade de BD
- **`onpremise_gitlab`**: Upgrade Service on-premise → `developer-skills:dynapps-db-upgrade-runner`
  (SSH al servidor de BD de producción — path lo resuelve la skill delegada; decisión
  **test/producción**; lanzar en background + vigilante).
- **`odoosh_github`**: delegar a `developer-skills:odoo-database-upgrader` con `target=odoo.sh`
  (upgrade **nativo** de odoo.sh, **siempre `test`** — sin STOP por test/prod; fixes vía glue module).
- **GATE de neutralización (obligatorio)**: el dump es de **producción** y el Upgrade
  Service lo devuelve SIN neutralizar. Antes de arrancar cualquier `odoo-bin` se ejecuta
  y se **verifica** la neutralización (`neutralize` + `neutralize_extras.sql`).
  Nunca se arranca Odoo contra la BD con el gate en rojo. Ver
  `dynapps-db-upgrade-runner/references/neutralization.md`.
- Iterate loop hasta `-u all` limpio → `developer-skills:odoo-database-upgrader`
  - **STOP ③**: estudio previo del consultor
  - **STOP ④**: decisión por módulo (KEEP / STUDIO / NEUTRALIZE / TO_DROP)
- El dump migrado final queda en local — **la subida al servidor es manual** (BLOQUE 5)

## BLOQUE 3 — Migración de código
Skill: `developer-skills:dynapps-code-migration`
Sub-skill por módulo: `developer-skills:odoo-module-upgrader`
Commits/MRs: `developer-skills:git-manager` (proveedor auto-detectado: `glab`/`gh`)

**Ramificación por `scenario`** (ver `references/scenarios.md`):

| | `onpremise_gitlab` | `odoosh_github` |
|---|---|---|
| Prerrequisitos | STEP 0.5 `gitlab-prerequisites.md` (obligatorio) | **se salta** `gitlab-prerequisites.md` |
| Proveedor git | `gitlab` (`glab`) | `github` (`gh`) |
| Addons OCA | `parts/oca_pull` | `parts/oca` (vendorizado a mano) |
| Gate (hito local) | verde en local (CI es posterior) | pre-commit + tests locales |

El **port por módulo y el iterate `-u` son idénticos** en ambos escenarios. Para el hito
"Entorno local listo" **no se crean MRs**: se porta y se deja arrancando limpio en local.

**Verificación previa OBLIGATORIA — solo `onpremise_gitlab`** (antes de tocar código):
ejecutar el `STEP 0.5` de `dynapps-code-migration` (`references/gitlab-prerequisites.md`):
- **Histórico git integrado** en `develop` (ancestro común con el repo viejo + commit `[MIG] Initial
  Commit`). Si falta (repo scaffoldeado con trabajo pusheado pero sin el rebase) → hacer los cimientos
  primero; nunca arrancar MRs por módulo encima de una base que habrá que reescribir. **STOP**.
- **Accesos GitLab**: `develop` protegida (push solo Maintainers → el rebase lo aplica un Maintainer
  vía rama lateral + reset); usuario **no External** (si no, CI = 0 jobs); acceso a
  `dynapps-apps/dyn-edi/*` (si no, el build de CI casca al clonar el EDI). Fallos aquí = petición a infra.

**Validación estricta (fase de entrega — POSTERIOR al hito "Entorno local listo"):**
- Un MR por módulo a la rama `develop` (origen). En `odoosh_github` es PR en GitHub (`gh`);
  no hay pipeline GitLab → gate = pre-commit + tests locales.
- Pipeline **100% verde** antes de merge (`onpremise_gitlab`) — gate duro, sin excepciones. Si el
  CI está bloqueado por accesos (External / dyn-edi), el gate es **verde en local** (pre-commit + tests).
- **Patrón de deps**: el módulo depende solo de `<prefijo>_base` + custom; las deps estándar/OCA van a
  `<prefijo>_base` (se añaden en el MR del módulo). Nunca deps estándar en el manifest del módulo.
- **Mensajes de commit sin `Co-Authored-By`** / atribución a IA (repos de cliente).
- **STOP ⑤**: conflictos de rebase sin patrón
- **STOP ⑥**: TODOs sin patrón tras port de módulo
- **STOP ⑦**: pipeline verde → notificar al técnico, esperar merge manual antes de continuar
  - Si pipeline rojo → diagnosticar y corregir primero, nunca avanzar con fallos

> **Registro en Dynbrain**: los fallos del mecanismo detectados en el iterate y en el port
> se registran genericados en el catálogo de errores comunes, deduplicando antes. Los fallos
> propios del cliente NO se registran ahí. Ver `dynbrain-feedback-loop.md`.

## BLOQUE 4 — Validación
Skills: `developer-skills:odoo-migration-parity` (paridad origen↔migrado) + `common-skills:odoo-pilot` (smoke RPC)

**Posterior al hito autónomo**: todo este bloque corre cuando el entorno local ya está
entregado. Nada de aquí entra en la corrida autónoma.

- **Criterio de paridad**: por defecto **1:1** con la v-origen. Una diferencia de comportamiento
  se trata como **regresión a corregir**, no como "cambio de versión aceptado", salvo decisión
  de negocio explícita.
- **Capa 2b — validación en navegador** (opcional, requiere Playwright MCP): cubre lo que solo
  existe en el cliente web (OWL/`t-inherit`, widgets, vistas Studio, wizards). La lista de
  flujos sale de los **hallazgos** de la Capa 1, no de un catálogo: 5–15 flujos, ≤12 llamadas
  cada uno. **Produce hallazgos, nunca un pass/fail**, y no entra en CI. Si el MCP no está,
  se omite con una línea en el informe. Ver `odoo-migration-parity:references/browser-layer.md`.
- **STOP ⑧**: validación funcional del consultor por módulo

## BLOQUE 5 — Go-live
- AUTO: verificar gates (módulos smoked, MRs merged, pipeline verde)
- **STOP ⑨**: confirmación final
- **MANUAL**: subir dump migrado al servidor de producción (fuera del flujo)
- AUTO: smoke RPC en producción post-subida → `common-skills:odoo-pilot`

## STOP points (resumen)

| # | Bloque | Razón | Acción requerida |
|---|---|---|---|
| ① | 1 | Login Vault/OIDC | Ejecutar `vault-login` en terminal |
| ② | 1 | Buildout no existe | Confirmar acceso repo y continuar setup |
| ③ | 2 | Estudio previo | Entregar md/xlsx/pdf del consultor |
| ④ | 2 | Módulos ambiguos | KEEP/STUDIO/NEUTRALIZE/TO_DROP por lote |
| ⑤ | 3 | Conflictos rebase | Revisar diff + decidir resolución |
| ⑥ | 3 | TODOs sin patrón | Revisar y resolver antes de continuar |
| ⑦ | 3 | MR listo (pipeline verde) | Técnico revisa y mergea manualmente; skill espera confirmación |
| ⑧ | 4 | Validación funcional | Checklist por módulo (Sí/No/Parcial) |
| ⑨ | 5 | Confirmación go-live | Gates OK + subida manual al servidor |

Ver detalle en `references/stop-points.md`.
