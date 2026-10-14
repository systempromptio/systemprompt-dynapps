# Gotchas — falsos positivos a evitar

Leer ANTES de reportar. Marcar como fallo algo de esta lista destruye la
confianza en el auditor.

## Multi-cliente: lo que NO es hardcoding

- **`self.env.company`, `company_id`, `self.env.companies`**: forma correcta de
  ser multi-compania. Nunca es hardcoding.
- **`self.env.user`, `self.env.ref('module.xmlid')`**: referencias dinamicas o
  a XML data propio del modulo. Correcto.
- **`ir.config_parameter`, `res.config.settings`, campos en `res.company`**:
  son LA solucion de parametrizacion. Encontrarlos es bueno, no un fallo.
- **IDs en XML data del propio modulo** (`<record id="...">` y `ref="..."`
  internos): correcto, no son IDs de BD hardcodeados.
- **Demo data en `demo/`** y datos en tests: pueden tener nombres/CIF ficticios;
  no penalizar. El problema es dato de cliente real en `data/` con `noupdate`.
- **Constantes tecnicas**: codigos de pais ISO, monedas, codigos AEAT/SII,
  esquemas SEPA/Facturae, namespaces XML, MIME types, puertos estandar. No son
  datos de cliente.
- **URLs de esquema o de organismos publicos**: namespaces XML
  (`http://www.w3.org/...`), AEAT, schemas SEPA pain.008, docs de Odoo. No son
  endpoints de cliente.
- **`default_get` / `default=` dinamicos** (lambda, `_default_*`): no son
  valores fijos de cliente.

## Version y target de migracion (el falso positivo mas caro)

- **Deriva la version de la serie del `version` del `__manifest__.py`**, no la
  preguntes ni la asumas. Un modulo `version` `18.0.x.y.z` es un modulo v18 y se
  audita como v18.
- **No asumas migracion.** Por defecto NO hay target de migracion: solo se
  evalua migrabilidad si el usuario lo pide ("migrar a 19", "para el catalogo
  Core 19"). Asumir "target 19" sobre un modulo v18 hace que se marquen como
  deuda APIs correctas (`res.users.groups_id`, `name_search()`, import
  `AND/OR` de `odoo.osv.expression`) que solo cambian en v19 — deuda
  inexistente para un modulo que se queda en v18.
- **En contexto v18 (sin migracion)**: lo anterior es CORRECTO, no se reporta.
  `version` en serie 18.0 es correcto y NO penaliza el bloque C.
- **En contexto migracion a v19**: esas mismas APIs SI son tareas de migracion,
  y `version` debe pasar a `19.0.x.y.z` (penaliza C hasta hacerse).
- Pasa siempre al subagente de code-auditor la version vigente y si hay o no
  target de migracion; sin ese dato, asume "sin migracion".

## Gate opt-in y empaquetado nuevo: matices

- Un **flag opt-in que existe y por defecto esta OFF es lo CORRECTO** — no lo
  marques como problema; el problema es su ausencia (anti-patron 9), no su
  presencia.
- **Config funcional neutra ya existente** (un enum tipo `auto_timesheet_type`,
  un `groups=` en un campo, un `active` estandar) **ES un gate valido**: no es
  "default pensado para un cliente". Solo es anti-patron 9 cuando el override
  altera un flujo nativo para todos **sin** ningun gate.
- `post_init_hook` que escribe flags **condicionado al gate opt-in** es
  correcto; el fallo es el `post_init` **incondicional**.
- `auto_install: True` es correcto en modulos de **infraestructura**
  (`indaws_base_*` y similares); solo es fallo en una feature opt-in.
- `icon.png` / logo divergentes: si existe un **workstream global de
  rebranding/logo V2** que unificara todos los modulos en su propio PR, **NO**
  lo reportes como fallo aqui (seria ruido); registralo como nota informativa,
  no penaliza el bloque C.
- "Modulo nuevo": aplica el reset de version/`migrations/` **solo si el modulo
  es nuevo en el repo**. Un modulo existente con historial de versiones
  legitimo NO se penaliza por tener `migrations/`.

## Dependencias = contexto, nunca objeto

- Los modulos de los que depende el auditado se leen **solo** para validar
  "extiende, no reinventa" (bloque B). **No se auditan, no se corrigen, no se
  puntuan** y no generan hallazgos propios. Reportar un fallo en un modulo padre
  como si fuera del modulo auditado es un error de alcance.
- "Extiende via `super()`" basado solo en docstrings/comentarios del modulo
  auditado NO es verificacion: es el anti-patron Trust-the-Doc. Confirma contra
  la firma real del padre o marca B como "inferido, no verificado".

## Anti-codigo: matices

- Que un modulo Core dependa de modulos **estandar de Odoo** o de **otros
  modulos Core** es correcto y deseable; no lo marques como acoplamiento.
- "Reimplementa el estandar" debe basarse en el veredicto de
  **odoo-code-auditor (Standard Features)** y **classification-guide**, no en
  una corazonada. Si no se delego, dilo en el informe; no inventes el hallazgo.
- Extender el estandar via herencia (`_inherit`) NO es reimplementarlo: es el
  patron correcto.

## Empaquetado

- La **ausencia de `x_`/Studio es correcta** en un modulo Core: es codigo
  empaquetado, no Studio. No pidas Studio.
- Un modulo Core pequeno (pocos modelos) es valido; no penalices por tamano ni
  exijas arquitectura de modulo grande. Se mide filosofia, no volumen.
- `version` con serie de Odoo distinta a la auditada no es fallo si el modulo
  declara su serie correctamente; comprueba contra la version objetivo.

## Documentacion

- El bloque D audita **solo documentacion dentro del modulo** (README/manifest).
  La ficha en Conocimiento es proceso interno del equipo: **no se audita, no se
  reporta, no penaliza** — esta fuera del alcance de este auditor.
- Un `README` breve pero correcto es suficiente para el bloque D; no exijas
  documentacion extensa en el codigo.

## Severidad

- No infles severidad por cantidad de coincidencias del grep: agrupa
  ocurrencias del mismo anti-patron en un solo hallazgo con ejemplos.
- Un unico 🔴 real de hardcoding pesa mas que veinte 🟡 de estilo: prioriza
  la esencia (multi-cliente, anti-codigo) sobre la forma.
