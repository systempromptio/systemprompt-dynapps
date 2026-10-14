# Delegacion y fusion de resultados

Este skill orquesta. No reimplementa inventario, clasificacion ni auditoria de
codigo. Aqui esta como invocar cada skill y como integrar su salida.

## odoo-module-scanner (common) — inventario

**Cuando:** paso 3, siempre.
**Como:** Agent(model: sonnet, description: "scan module") con el prompt:
"Invoca odoo-module-scanner sobre {ruta}. Devuelve inventario: modelos, campos,
vistas, security (ir.model.access, record rules), controladores, JS/OWL,
manifest (`__manifest__.py` completo), ficheros en `data/` y `demo/`."

**Uso:** la lista de ficheros `data/` vs `demo/` alimenta el anti-patron 3; el
manifest alimenta el bloque C; el inventario acota donde buscar anti-patrones.

## odoo-classification-guide (common) — Golden Rules / Core Supremacy

**Cuando:** paso 4, siempre.
**Como:** consulta sus references (`golden-rules.md`, `classification-rules.md`).
Pregunta a responder: dada la funcionalidad del modulo, ¿deberia ser Standard,
Studio, otro modulo Core ya existente, o Custom? ¿Viola GR1 (import/export) o
GR2 (dashboards/KPIs)? ¿Hay un modulo Core que ya lo cubre (Core Supremacy)?

**Uso:** veredicto → bloque B (anti-codigo). Si classification-guide dice que la
funcionalidad es Standard o ya existe en Core, es 🔴 en bloque B.

## Contexto de dependencias (paso 3.5) — SOLO LECTURA, NO auditar

**Cuando:** el modulo audita hace `_inherit` / `super()` / override de metodos
de modulos Core listados en sus `depends`.
**Como:** localiza el codigo de esos padres (mismo repo/addons-path) y lee
**solo la superficie publica relevante**: la firma y el cuerpo minimo de los
metodos/campos que el modulo auditado extiende (p.ej. si overridea
`_prepare_attendace_domain`, `_create_or_update_timesheet`, `name_search`,
confirma que existen en el padre y que el override los extiende via `super()`
sin ensombrecer ni duplicar logica). Hazlo con Grep/Read directos o un
Agent(model: sonnet) si son varios ficheros.

**Uso:** valida el **bloque B** (Core Supremacy / "extiende, no reinventa") con
base real en vez de docstrings. Si el override duplica/contradice al padre en
vez de extenderlo → hallazgo 🟠/🔴 en B del modulo auditado.

**Limites estrictos:**
- Los modulos padre **NO se auditan, NO se corrigen, NO se puntuan**. No generan
  hallazgos propios ni entran en el Core Compliance Score.
- Solo se reportan como **contexto** ("verificado contra
  `indaws_hr/...:NN`") para respaldar/levantar hallazgos del modulo auditado.
- Si el codigo del padre no esta disponible, dilo: el bloque B queda
  "inferido de docstrings (no verificado)", no lo des por verificado.

## odoo-code-auditor (developer) — calidad/seguridad/migrabilidad

**Cuando:** paso 6, siempre.
**Como:** Agent(model: sonnet, description: "audit {modulo} code v{version}")
con un prompt que **debe** incluir: la version vigente {version} derivada del
`__manifest__.py`, y si NO hay migracion (instruir explicitamente que no marque
como deuda APIs correctas en esa version). Focus areas base: Standard Features,
Conventions, Security, Performance, Code Quality. **Anade Migrability SOLO si el
usuario pidio migracion**, indicando el target (p.ej. v18→v19). Nivel {nivel}.
Pide findings con severidad y score /10 por categoria.

Plantilla sin migracion: "Invoca odoo-code-auditor sobre {ruta}. El modulo es
v{version} y SE MANTIENE en esa version: NO evalues migrabilidad ni marques
como fallo APIs correctas en v{version}. Focus: Standard Features, Conventions,
Security, Performance, Code Quality."

Plantilla con migracion: añade "...migrar de v{version} a v{target}: incluye
Migrability y reporta los cambios de API como tareas de migracion."

**Uso:**
- "Standard Features" → refuerza/origina hallazgos del bloque B.
- "Conventions" → complementa bloque C (estructura, naming interno).
- Code Quality + Security + Performance (+ Migrability si aplica) → bloque E
  (normaliza: media de scores/10 * peso). Criticos de seguridad se elevan a 🔴.
- Sin migracion: el bloque E NO incluye Migrability; `version` en su serie
  vigente es correcto y C no se penaliza por ello.

## Reglas de fusion

1. **No dupliques.** Si code-auditor ya reporta un hallazgo de estandar, citalo
   en bloque B; no lo repitas en E.
2. **Filosofia manda.** Tus hallazgos propios (A, B, C, D) son el nucleo del
   informe; lo delegado (E) es soporte.
3. **Trazabilidad.** Cada hallazgo indica su origen: "(propio)",
   "(classification-guide)" o "(code-auditor)".
4. **Si una delegacion falla** (skill no disponible, sin permisos): dilo
   explicitamente en el informe y marca ese bloque como "no evaluado", nunca lo
   inventes ni lo des por aprobado.

## Invocacion desde otras skills (este skill como consumido)

Cuando lo invoque dev-assessment / quality-gate / git-pr / odoo-code-auditor:
- Reutiliza el contexto que traigan (ruta, version, inventario ya hecho) y no
  repreguntes.
- dev-assessment: aun no hay codigo; audita la *propuesta* contra filosofia
  (multi-cliente, Golden Rules, Core Supremacy) y devuelve veredicto apto/no.
- quality-gate / git-pr: modo pre-merge; devuelve Core Compliance Score +
  bloqueantes (cualquier 🔴) de forma compacta para el gate.
