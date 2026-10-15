# Conexión y seguridad

## Credenciales (nunca hardcoded)

Todos los scripts leen la conexión de variables de entorno. Pídeselas a la persona y expórtalas
en la sesión; no las escribas en ningún fichero del repo ni en el código:

```bash
export ODOO_URL="https://odoo.cliente.com"
export ODOO_DB="nombre_db"
export ODOO_USER="usuario@cliente.com"
export ODOO_KEY="contraseña_o_api_key"
# opcional, para acotar a una compañía:
export ODOO_COMPANY_ID="1"
```

Si la persona prefiere no pegar la contraseña, sugiérele crear una **API key** en Odoo
(Preferencias → Seguridad de la cuenta → Nueva API key) y usarla como `ODOO_KEY`.

## Por qué solo lectura

El entorno suele ser **producción con gente trabajando**. La skill es de *diagnóstico*, no de
cambios. El módulo `scripts/_odoo.py` impone una lista blanca de métodos
(`search`, `search_read`, `read`, `fields_get`, `search_count`, `read_group`, `name_search`…):
cualquier intento de `create`/`write`/`unlink` lanza `PermissionError`. Es una red de seguridad,
no un sustituto del criterio: no desactives la lista blanca.

**Lo que la lista blanca NO impide** (medido, `methodology.md` §«Solo lectura»): que tus lecturas
ejecuten el código a medida del cliente. Un `search_read` de un campo calculado corre su `compute`;
un dominio sobre un campo con `search=` corre su `_search_…`. Si ese código escribe, escribe — y el
`PermissionError` no llega a verse, porque tú pediste una lectura. La lista blanca protege de tus
errores, no de los efectos del sistema que estás midiendo.

`search_count([])` con dominio **vacío** solo cuenta filas: es seguro, y así lo usa
`count_records.py`. Un `search_count` **con dominio** sobre un campo calculado ya no lo es, por lo
de arriba.

`ir.logging` es la única tabla en la que escribir es inocuo (es técnica y borrable); puede usarse
como canal de captura activo si hiciera falta, pero **no es necesario** para el flujo estándar y
requiere OK explícito del cliente. No escribas en ningún otro sitio.

## Protocolo por versión

- Odoo < 19.0 → XML-RPC / JSON-RPC. Los scripts usan `xmlrpc.client` directamente.
- Odoo ≥ 19.0 → protocolo JSON2. Si te topas con una v19+, apóyate en la skill `odoo-pilot`
  para la capa de transporte y reutiliza la lógica de estos scripts.

## Identificar el código a medida

`discover_modules.py` lista los módulos instalados agrupados por **autor** y propone un primer
corte excluyendo Odoo S.A., OCA y localizaciones (`l10n_*`). El método más fiable es preguntar
quién desarrolló el sistema y filtrar por ese autor:

```bash
python3 scripts/discover_modules.py <out_dir> --author "Nombre del Autor"
```

Revisa `modules.json` con la persona antes de seguir: confirmar la lista de módulos a medida es
la decisión que condiciona todo lo demás.
