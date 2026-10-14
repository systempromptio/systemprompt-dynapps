---
name: indaws-odoo-impact-analyzer
version: 0.2.2
effort: medium
model: opus
description: >-
  Analiza el impacto de renombrar o eliminar un campo, modelo o metodo Odoo. Usala cuando preguntes que depende de algo o si es seguro cambiarlo. Genera informe de dependencias y checklist de cambio seguro antes de un rename, borrado o breaking change.
allowed-tools: [Bash, Read, Agent]
---

# Odoo Impact Analyzer

Rastrea TODAS las dependencias de un símbolo Odoo (campo, modelo, método) antes
de cambiarlo y emite un informe de impacto + checklist de cambio seguro.

Inspección en vivo y límites: ver `references/shared/odoo-connection.md` y
`references/shared/batch-limits.md`. Para comportamientos ocultos, apóyate
en la skill `odoo-deep-knowledge`.

## Workflow

### 1. Identificar el objetivo [STOP]

Pedir al usuario:
- **Símbolo objetivo**: `modelo.campo`, `modelo.método`, o nombre de modelo
- **Tipo de cambio**: renombrar / eliminar / cambiar tipo / cambiar comportamiento
- **Ruta raíz de búsqueda**: directorio base (p.ej. `$PROJECTS_ROOT/indawsodoo/18`)
- **Versión Odoo** (v17/v18/v19) — afecta a patrones XML de atributos

Si el usuario da solo `campo` sin modelo, pedir el modelo antes de continuar.

### 2. Escanear dependencias — MOTOR determinista [script, ~0 tokens LLM]

`python3 scripts/scan_impact.py --root R --model M [--field F | --method MET] [--exclude-base]`
→ JSON de dependientes CLASIFICADOS por tipo (fichero:línea + `risk` y `confidence`, ver paso 3) + bloque
`confidence` que SIEMPRE marca `needs_db_pass`. `--md` para tabla legible. NO grep a mano: el motor
implementa `references/scan-protocol.md`. El `string_ref`/acceso directo es heurístico (trae falsos
positivos, marcados BAJO); `csv_acl` solo se emite cuando el TARGET es el modelo (borrado/rename).
**Fallback:** si el script falla o el entorno no lo ejecuta, escanea a mano con `references/scan-protocol.md`.

### 2.5. Introspección en instancia viva [OBLIGATORIA si `needs_db_pass` y hay instancia]

El grep es estático: NO ve campos Studio `x_` ni vistas/dominios/reglas/acciones creadas por UI (BD).
Delegar en `odoo-pilot` (API) o `odoo shell` y fusionar con los hallazgos del motor:
- `fields_get([CAMPO])` / `ir.model.fields` — tipo, `related`/`compute`/`store`, campos Studio `x_`.
- `ir.ui.view` (arch_db) · `ir.filters` · `ir.rule` — vistas/dominios/reglas en BD.
- `ir.actions.server` / `base.automation` — código Python inline que usa el campo.

### 3. Revisar clasificación y riesgo [LLM]

El motor emite **dos ejes** (`references/dependency-types.md` §«Lo que pasa de verdad»):

- **`risk` = cuánto tarda en verse el fallo**, medido, no supuesto: CRÍTICO se rompe sin aviso ·
  ALTO revienta al primer uso · MEDIO revienta al instalar (**no llega a producción**) · BAJO
  mención suelta. La tabla completa, con qué cae en cada uno, en `references/dependency-types.md`.
- **`confidence`** = si el TIPO está bien identificado. **No** dice que el símbolo sea de tu
  modelo: el grep no resuelve tipos.

Revisa: baja los `string_ref` falsos, resuelve a qué modelo pertenece cada `python_access`, y sube
lo confirmado por la pasada de BD (2.5). **Que algo reviente al instalar es la mejor noticia**: te
enteras el mismo día.

### 5. Inventario de módulos [delega a odoo-module-scanner]

Si hay dependientes en módulos distintos al módulo origen, invocar
`odoo-module-scanner` para listar sus componentes y confirmar que el campo
aparece en sus manifiestos o herencias.

### 6. Emitir informe + checklist

Usar `references/report-format.md` para estructurar:
1. Resumen ejecutivo (símbolo, tipo de cambio, nº dependientes, riesgo global)
2. Tabla de dependientes por tipo con nivel de riesgo
3. Checklist de cambio seguro (qué tocar, en qué orden, qué probar)
4. Módulos a actualizar con sus pasos específicos

## GOTCHAS

1. **Lo que se rompe en SILENCIO es `@api.onchange` y `@api.constrains`** — medido: con un campo
   inexistente en el decorador, el módulo **instala sin decir nada** y el método **no se ejecuta
   jamás**. La constraint que vigilaba ese campo deja de validar y el registro prohibido entra. No
   hay error, ni al instalar ni al usar: solo datos malos. Es el peor resultado de un rename mal
   hecho y ningún test de negocio lo caza.
2. **`related=` propaga el nombre del campo** — renombrar el campo origen rompe
   TODOS los `related=` que lo usan, aunque estén en otros módulos no escaneados.
   (Eso sí avisa: `KeyError` al instalar.)
3. **La mitad del XML interesante va DENTRO de un `<field>`, no en un atributo** — dominios de
   `ir.filters`/`act_window`/`ir.rule` y el código de las server actions. Un grep de atributos los
   pierde enteros. Y al revés: `<field name="state">code</field>` **no** es una referencia al campo
   `state`, es el valor de la propia acción.
4. **Los dominios creados por UI viven en BD** — `ir.filters` guardados por el usuario, reglas y
   acciones hechas desde la interfaz no están en ningún `.xml`. Sin la pasada del paso 2.5 no los
   ve nadie.
5. **`attrs` deprecated en v17+** — en v17+ buscar también `invisible=`, `required=`,
   `readonly=` directamente en el tag XML, no solo `attrs`.
6. **`_inherit` sin `_name`** — extiende el modelo original; los campos añadidos
   ahí también pueden tener `@api.depends` del campo objetivo.
7. **Herencia de vistas (`inherit_id`)** — una vista hija puede referenciar un campo
   aunque la vista padre no lo use directamente.
8. **`search=` es un método Python**, no un campo; buscarlo también en `.py`.
9. **`ir.model.access.csv`** no referencia campos individuales, solo modelos;
   si se elimina un modelo, revisar este fichero en todos los módulos.
10. **Automatizaciones técnicas** (`base_automation`) pueden tener código Python
   inline con referencias al campo — buscar también en registros de BD.

## NEVER

- NO emitir el informe sin haber escaneado `.py`, `.xml` Y `.csv`
- NO omitir módulos `_inherit` externos aunque sean de Odoo base
- NO clasificar como BAJO riesgo un `compute=` o `related=` sin verificar
- NO asumir que un campo sin `compute` no tiene dependientes compute en otros módulos

## ALWAYS

- SIEMPRE incluir el checklist de cambio seguro, no solo el inventario
- SIEMPRE indicar el orden de migración entre módulos (upstream primero)
- SIEMPRE verificar el campo en `_sql_constraints`/índices SQL y en migraciones existentes (`migrations/`)

## Delegates to

| Skill | Suite | Cuándo |
|-------|-------|--------|
| odoo-pilot | common | Introspección en instancia viva (paso 2.5): `fields_get`, `ir.model.fields`, `ir.ui.view`, `ir.filters`, `ir.rule`, `ir.actions.server`, `base.automation` |
| odoo-module-scanner | common | Inventario de componentes de módulos dependientes |
| odoo-code-auditor | ai-claude-developer | Deuda técnica relacionada con el símbolo |
| odoo-version-api-reference | common | Verificar si el patrón es deprecated en la versión objetivo |

## Consumed by

| Skill | Suite | Para qué |
|-------|-------|----------|
| odoo-module-upgrader | ai-claude-developer | Planificar migración tras el análisis de impacto |
| odoo-traceback-doctor | ai-claude-developer | Correlacionar errores con campos/modelos modificados |
| odoo-module-migration-scripts | developer | Medir impacto/dependencias de un rename antes de escribir el script |
| odoo-orm-modeler | developer | Impacto antes de cambiar/renombrar/borrar un campo en producción |

## REFERENCES

| Fichero | Contenido | Cargar cuando |
|---------|-----------|---------------|
| [references/dependency-types.md](references/dependency-types.md) | Dónde se esconden las dependencias en Odoo (todos los tipos) | Paso 3 — clasificar |
| [scripts/scan_impact.py](scripts/scan_impact.py) | **Motor determinista** — scan-protocol → dependientes clasificados + riesgo + `confidence` (JSON/`--md`) | Paso 2 — escanear |
| [references/scan-protocol.md](references/scan-protocol.md) | Patrones grep exactos (los implementa el motor; fallback/consulta) | Paso 2 — escanear |
| [references/report-format.md](references/report-format.md) | Formato del informe de impacto y del checklist | Paso 6 — emitir informe |
