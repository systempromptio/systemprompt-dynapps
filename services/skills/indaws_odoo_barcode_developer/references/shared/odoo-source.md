# Resolver de fuente Odoo (no asumir `$ODOO_SRC`)

Política compartida para localizar el **código fuente de Odoo** cuando una skill
necesita verificar API/comportamiento contra fuente. **Nunca asumas `$ODOO_SRC`**:
esa es la convención de un entorno concreto, no la de todos.

## Citas de procedencia ≠ ruta del usuario
Las references de las skills citan `fichero:línea` (ej. `odoo/orm/fields.py:482`)
como **recibo** de dónde se verificó un hecho **en el entorno del autor**. El hecho
documentado es válido aunque tu fuente esté en otra ruta (o no la tengas). No falla
nada por no tener ese path: solo significa que no puedes **re-verificar** ahí.

## Orden de resolución de la base de fuente
Para localizar la base del código (donde están `odoo{N}/`, enterprise, OCA), en este orden:
1. **Variable de entorno** `ODOO_SRC` / `ODOO_BASE` si está definida.
2. **CLAUDE.md** del proyecto o global → sección "Rutas Locales Odoo" (si existe).
3. **Ubicaciones comunes**: `$ODOO_SRC`, `~/odoo`, `~/src/odoo`, `./odoo` — comprobar
   cuál existe (`ls <base>/odoo18` o equivalente para la versión objetivo).
4. Si nada resuelve → **PREGUNTAR AL USUARIO** la ruta de su fuente Odoo (no la
   inventes ni la asumas; el usuario debe indicártela si no existe).

Para Enterprise y OCA, misma lógica con su base (`ODOO_ENTERPRISE_SRC`/`OCA_SRC` o
las rutas del CLAUDE.md). Convención del equipo Indaws: `$ODOO_SRC/odoo{N}`,
`$ODOO_ENTERPRISE_SRC/{N}`, `$OCA_SRC/v{N}` (pero **resuélvelo, no lo des por hecho**).

## Si la fuente NO está disponible
No inventes ni asumas. Según el caso:
- **El hecho ya está documentado** en la reference de la skill → úsalo, y avisa de que
  **no se ha podido re-verificar en este entorno** (transparencia).
- **Necesitas verificar algo nuevo** → pide al usuario la ruta, **o** indícale cómo
  obtener la fuente:
  - Skill **`indaws-project-setup`** (common) — clona/actualiza CE/EE/OCA/Indaws Core
    a las rutas estándar y genera el CLAUDE.md con ellas.
  - Sugerir descarga **shallow** (rápida, sin historia):
    `git clone --depth=1 -b <N>.0 https://github.com/odoo/odoo` (CE).
    Enterprise (`odoo/enterprise`) y OCA según acceso del cliente/equipo, igual con `--depth=1`.
- Para firmas/diffs de API sin fuente local → la skill **`odoo-version-api-reference`**
  es dato documentado (v18 canónico + diffs v15-v19) y **no requiere** la fuente.

## Convención de cita (procedencia)
- Cita **relativa a la fuente + símbolo**, no ruta de máquina ni línea suelta:
  `addons/sale/models/sale_order.py::_compute_amount_total` (o `…py:120 (_método)`
  si quieres la línea como pista). El símbolo NO driftea; la línea y el `/opt/…` sí.
- Las citas absolutas `/opt/…py:línea` heredadas son "recibos" válidos (no tener la
  ruta no es fallo), pero **migra a relativa+símbolo al tocar el fichero**.
- Lo verifica el gate único del ecosistema: `skill-designer/check_source_citations.py`.

## Scripts (resolución en código)
- Un script que necesite la base **resuelve por env primero** (`ODOO_SRC`/`ODOO_BASE`/
  `ODOO_SRC_ROOTS`), luego prueba ubicaciones comunes (con `isdir`, no asumir). Una lista
  de candidatos `/opt/…` es válida como *fallback que se prueba*, nunca como ruta dada por
  hecha. Sin fuente → el script se omite, no falla.

## Regla
- NUNCA hardcodear ni asumir `$ODOO_SRC` como si fuera universal.
- NUNCA fabricar una firma/comportamiento "como si" lo hubieras verificado.
- SIEMPRE: resolver la fuente por el orden de arriba; si no hay, usar el dato
  documentado con aviso, o pedir la ruta / derivar a `indaws-project-setup`.
