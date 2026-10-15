# Patrones de fix por familia + verificacion

Para cada familia de `error-families.md`: el **fix minimo** y **como verificar**. El principio es siempre el cambio mas pequeno que corrige la causa raiz, no un refactor. Si el error destapa deuda mas amplia, eso va aparte (delega en `odoo-code-auditor`). Diferencias por version: delega en `odoo-version-api-reference`.

Comandos de verificacion (sustituir `<modulo>`, `<db>`):
- **Re-`-u`:** `odoo-bin -u <modulo> -d <db> --stop-after-init` (Indaws: anadir `2>&1 | grep -v 'unknown parameter.*export\|selection attribute will be ignored\|Will use the Wkhtmlto'`). Exito = arranca sin traceback.
- **Test dirigido:** `odoo-bin -u <modulo> -d <db> --test-tags <tag> --stop-after-init 2>&1 | tail -30`.
- **Shell:** `odoo-bin shell -d <db>` para inspeccionar. Para datos/estado en vivo (constraints, registros infractores, modulos) usar `odoo-pilot`.

---

## Familia A — Instalacion / `-u`

### A1. ParseError (XML)
- **Fix:** corregir el XML en el fichero/linea del log: cerrar etiqueta, escapar `&` -> `&amp;`, arreglar el `ref=` roto o la expresion del `arch`.
- **Verificar:** re-`-u`. El ParseError ya no aparece.

### A2. Validacion de vista
- **Fix:** ajustar el xpath al arch real de la vista padre (puede haber cambiado de version), o quitar/renombrar el campo inexistente del arch. Si es atributo retirado (`attrs`/`states` en v17+), reescribir como atributo dinamico directo (`invisible="domain"`, `readonly="..."`).
- **Verificar:** re-`-u`; abrir la vista en UI y confirmar que renderiza.

### A3. "Field X does not exist"
- **Fix segun rama:** (1) typo -> corregir nombre; (2) dependencia -> anadir el modulo a `depends` del manifest; (3) orden -> reordenar `data` para que el modelo/vista base cargue antes, o reforzar `depends`.
- **Verificar:** re-`-u`.

### A4. Dependencia faltante en manifest
- **Fix:** anadir el modulo que falta a `depends` en `__manifest__.py`.
- **Verificar:** re-`-u` (Odoo recalcula el orden topologico).

### A5. KeyError/AttributeError en carga del registry
- **Fix:** corregir el `_inherit`/`_name` (modelo destino existe y su modulo esta en `depends`), o resolver el `_name` duplicado. Si es orden, ajustar `depends`.
- **Verificar:** re-`-u`; el registry carga completo (log "Modules loaded").

---

## Familia B — Base de datos

### B1. IntegrityError
- **Fix:** depende de la causa. (a) Dato sucio -> limpiar/corregir los registros infractores (localizarlos en vivo via odoo-pilot mapeando el nombre del constraint a modelo/campo). (b) `_sql_constraints` nuevo incompatible con datos existentes -> migrar datos en un script `pre`/`post` antes de crear el constraint, o relajar el constraint si la regla no aplica a historico.
- **Verificar:** reintentar la operacion / re-`-u`; el constraint ya no salta.

### B2. UndefinedColumn / UndefinedTable
- **Fix:** ejecutar el `-u` del modulo que define el campo/modelo (crea la columna). Si fue por anadir `store=True`, el mismo `-u` la materializa.
- **Verificar:** re-`-u`; arranque limpio.

### B3. SerializationFailure / Deadlock
- **Fix:** de diseno: serializar el acceso (evitar crons solapados, ordenar escrituras, reducir el alcance de la transaccion). No se arregla tocando un registro.
- **Verificar:** reproducir la concurrencia (o el cron) y confirmar que no reaparece.

### B4. DataError
- **Fix:** ampliar el tipo/size del campo o sanear el valor de entrada antes de escribir.
- **Verificar:** reintentar con el valor que fallaba.

---

## Familia C — Excepciones de negocio

### C1. AccessError
- **Fix:** SOLO si el acceso debe existir: anadir la ACL en `ir.model.access.csv` o ajustar la `ir.rule`/grupo correcto. Si el usuario NO deberia acceder, el error es correcto: el fix es de proceso, no de codigo. NUNCA relajar reglas para silenciar.
- **Verificar:** repetir la accion con el usuario afectado.

### C2. ValidationError
- **Fix:** corregir el dato que incumple el `@api.constrains`/`_sql_constraints`. Solo cambiar el constraint si la regla de negocio estaba mal definida.
- **Verificar:** reintentar la operacion con dato valido.

### C3. UserError
- **Fix:** normalmente NO hay bug: es una regla de negocio. Guiar al usuario por el flujo correcto. Solo tocar codigo si el `raise` esta mal condicionado.
- **Verificar:** completar el flujo cumpliendo la precondicion.

### C4. MissingError
- **Fix:** validar existencia antes de operar (`if record.exists():`), o corregir el origen del id obsoleto.
- **Verificar:** reproducir el escenario de borrado/cache y confirmar que ya no rompe.

---

## Familia D — Cliente (assets / OWL)

### D1. Assets
- **Fix:** declarar el asset en el bundle correcto de `__manifest__['assets']`, corregir orden o el SCSS/JS roto.
- **Verificar:** re-`-u` + recarga con cache vacia (modo dev `?debug=assets`); consola del navegador sin errores.

### D2. OWL
- **Fix:** corregir referencia al template QWeb-JS, props o uso de API de framework cambiada entre versiones (delega en odoo-version-api-reference para el diff).
- **Verificar:** recargar la vista; consola del navegador limpia y el componente renderiza.

---

## Familia E — Migracion

### E1. `-u` por API retirada
- **Fix:** reemplazar la API/atributo retirado por su equivalente en la version destino (consultar el diff via odoo-version-api-reference). Ej.: `name_get` (retirado en v18, deprecado con warning en v17) -> `_compute_display_name`, `attrs`/`states` -> atributos dinamicos, `@api.one` -> iterar sobre `self`.
- **Verificar:** re-`-u` en la version destino; arranque limpio.

### E2. Scripts de migracion
- **Fix:** corregir el script en la fase correcta (`pre` antes del modelo nuevo, `post`/`end` despues), anadir guardas (`column_exists`, `table_exists`) y no asumir esquema previo.
- **Verificar:** re-`-u` del modulo en la version destino sobre una copia de la BD origen; el script corre sin traceback.
