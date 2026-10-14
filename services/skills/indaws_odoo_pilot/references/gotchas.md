# Gotchas — Errores frecuentes al usar odoo-pilot

Errores reales observados al interactuar con Odoo via API. Actualizar cada vez que se detecte uno nuevo.

---

## Autenticacion y Protocolo

### 1. Usar JSON2 con Odoo < 19.0
**Error**: Claude intenta usar endpoints `/json/2/` con instancias Odoo 17 o 18.
**Resultado**: 404 Not Found silencioso.
**Fix**: SIEMPRE comprobar version primero. `auth.sh` lo hace automaticamente. Si dudas, ejecuta `detect_version.sh`.

### 2. Olvidar `eval $(...)` al autenticar
**Error**: Ejecutar `./scripts/auth.sh` sin `eval $()`. El script imprime las variables pero no las exporta.
**Resultado**: Las siguientes llamadas fallan con "Access Denied" o "Authentication required".
**Fix**: Siempre `eval $(./scripts/auth.sh)`.

### 3. Olvidar `ODOO_USER` para JSON-RPC
**Error**: Configurar `ODOO_URL`, `ODOO_DB` y `ODOO_KEY` pero no `ODOO_USER` en instancias < 19.0.
**Resultado**: JSON-RPC necesita el username para autenticar. Falla con error poco descriptivo.
**Fix**: `export ODOO_USER="admin"` (o el login del usuario).

### 4. Session cookie expirada (JSON-RPC)
**Error**: Autenticar una vez y seguir trabajando durante horas. La session expira (por defecto ~7200s).
**Resultado**: Errores "Session expired" o "CSRF token missing" despues de un rato.
**Fix**: Re-ejecutar `eval $(./scripts/auth.sh)` si llevas tiempo sin hacer llamadas.

---

## Dominios y Busquedas

### 5. Dominio vacio como string en lugar de lista
**Error**: Pasar `'[]'` (string) cuando el script espera un JSON array. O pasar `''` (vacio).
**Resultado**: Error de parsing o busqueda que no devuelve resultados.
**Fix**: Siempre pasar el dominio como JSON valido: `'[["field","=","value"]]'` o `'[]'` para todo.

### 6. Notacion polaca mal construida (OR/NOT)
**Error**: Escribir `[["field1","=","a"],"or",["field2","=","b"]]` (estilo SQL).
**Resultado**: Error de dominio invalido.
**Fix**: Notacion polaca: `["|",["field1","=","a"],["field2","=","b"]]`. El operador va ANTES de los operandos.

### 7. Buscar sin `context: {"lang": "es_ES"}`
**Error**: Buscar productos o campos con nombre en español sin especificar contexto de idioma.
**Resultado**: Los nombres devueltos estan en ingles (idioma base). Busquedas por nombre fallan.
**Fix**: Anadir `"context": {"lang": "es_ES"}` en las llamadas. Los scripts lo soportan via kwargs.

### 8. Usar `search_records.sh` sin limite para tablas grandes
**Error**: `search_records.sh res.partner '[]' '["name"]'` sin limite.
**Resultado**: Timeout o respuesta enorme que bloquea. Tablas como `mail.message` o `ir.logging` tienen millones de registros.
**Fix**: SIEMPRE pasar un limite: `search_records.sh res.partner '[]' '["name"]' 100`.

---

## Escritura y Relaciones

### 9. Many2many con lista directa en lugar de Command API
**Error**: Escribir `{"tag_ids": [1, 2, 3]}` para asignar tags.
**Resultado**: Error o comportamiento inesperado. Odoo espera tuplas de comando.
**Fix**: Usar `{"tag_ids": [[6, 0, [1, 2, 3]]]}` para reemplazar todos, `[[4, id]]` para anadir uno.

### 10. One2many con write directo
**Error**: Intentar hacer `update_record.sh` sobre un campo One2many como si fuera un campo normal.
**Resultado**: Error. Los One2many se modifican via Command API en el padre.
**Fix**: `{"line_ids": [[0, 0, {"name": "nueva linea"}]]}` para crear, `[[1, id, {"name": "edit"}]]` para editar.

### 11. Crear registros sin campos obligatorios
**Error**: `create_record.sh sale.order '{"partner_id": 1}'` sin `order_line` ni otros campos requeridos.
**Resultado**: `ValidationError: Missing required fields`. El error a veces no indica cual campo falta.
**Fix**: Consultar la vista form del modelo para saber que campos son `required=True`.

### 12. Escribir en campos computados no-stored
**Error**: Intentar hacer `update_record.sh` con un campo computado que no tiene `inverse`.
**Resultado**: El valor se ignora silenciosamente o da error.
**Fix**: Los campos computados se actualizan via sus dependencias, no directamente.

---

## Modulos

### 13. Instalar modulo con nombre incorrecto
**Error**: Usar `install_module.sh "Ventas"` (nombre visible) en lugar del nombre tecnico.
**Resultado**: Modulo no encontrado.
**Fix**: Usar nombre tecnico: `install_module.sh sale_management`. Buscar antes: `search_records.sh ir.module.module '[["name","ilike","sale"]]' '["name","shortdesc","state"]'`.

### 14. No esperar a que termine la instalacion
**Error**: Instalar un modulo y ejecutar operaciones inmediatamente despues.
**Resultado**: Los modelos o campos del modulo no estan disponibles aun.
**Fix**: Verificar estado del modulo despues de instalar: `search_records.sh ir.module.module '[["name","=","sale_management"]]' '["state"]'`.

---

## JSON2 Especificos (Odoo >= 19.0)

### 15. Usar `/json/2/call` como endpoint generico
**Error**: Claude intenta usar un endpoint generico `/json/2/call` como en JSON-RPC.
**Resultado**: 404. JSON2 NO tiene endpoint generico.
**Fix**: Cada llamada va a `/json/2/{model}/{method}`. Ej: `/json/2/res.partner/search_read`.

### 16. Envolver parametros en `{"jsonrpc": "2.0", ...}` para JSON2
**Error**: Copiar la estructura de JSON-RPC al hacer llamadas JSON2.
**Resultado**: Error de formato. JSON2 espera parametros directos, sin wrapper.
**Fix**: Body directo: `{"domain": [...], "fields": [...], "limit": 10}`.

### 17. Olvidar header `X-Odoo-Database` en JSON2
**Error**: Enviar solo `Authorization: bearer KEY` sin el header de base de datos.
**Resultado**: Error de autenticacion o base de datos incorrecta (si hay multiples DBs).
**Fix**: Siempre incluir `X-Odoo-Database: nombre_db` en las cabeceras JSON2.

---

## Modelo y Campos

### 18. Confundir `product.product` con `product.template`
**Error**: Buscar productos en `product.template` cuando se necesita la variante, o viceversa.
**Resultado**: No encuentra el producto, o encuentra el template sin la variante especifica.
**Fix**: `product.template` = ficha de producto. `product.product` = variante especifica. Las busquedas por nombre suelen funcionar mejor en `product.template`.

### 19. Campos Studio con prefijo incorrecto
**Error**: Buscar `x_custom_field` cuando Studio lo creo como `x_studio_custom_field`.
**Resultado**: Campo no encontrado.
**Fix**: Campos Studio siempre empiezan con `x_studio_`. Verificar con `execute_method.sh ir.model.fields search_read '[["model","=","res.partner"],["name","ilike","x_studio"]]' '{"fields": ["name","field_description","ttype"]}'`.

### 20. Acceder a modelos sin permisos
**Error**: Buscar en un modelo para el que el usuario API no tiene acceso.
**Resultado**: JSON2 devuelve 403 Forbidden. JSON-RPC devuelve un error de acceso menos claro.
**Fix**: Verificar que el usuario tiene permisos. En desarrollo, usar usuario admin. En produccion, crear un usuario API con los grupos necesarios.

---

## Regla general

Ante un error no esperado:
1. Verificar que `auth.sh` se ejecuto correctamente (`echo $ODOO_PROTOCOL`)
2. Verificar version (`echo $ODOO_VERSION`)
3. Probar con una busqueda simple primero (`search_records.sh res.partner '[]' '["name"]' 1`)
4. Revisar la respuesta completa del servidor (suele contener el traceback)
