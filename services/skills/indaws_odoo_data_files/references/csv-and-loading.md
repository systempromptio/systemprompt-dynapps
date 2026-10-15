# Ficheros CSV de datos y orden de carga

Fuente: `convert_csv_import` en `odoo18/odoo/tools/convert.py::convert_csv_import`.

## Cuando CSV vs XML

- **CSV**: muchos registros homogeneos del mismo modelo (paises, monedas, categorias, lineas de
  configuracion masiva). Mas compacto y legible para tablas.
- **XML**: registros con logica (`ref`/`eval`/`search`), pocos registros, registros de sistema,
  one2many anidados, plantillas, menus, funciones, borrados. Todo lo no-trivial.

## Formato CSV

```csv
id,name,parent_id:id,active,sequence
mi_modulo.cat_root,Raiz,,True,1
mi_modulo.cat_hijo,Hijo,mi_modulo.cat_root,True,2
```

Reglas (convert.py:671-700):
- **El nombre del fichero es el modelo:** `model = filename.split('-')[0]`
  (convert.py:679). Asi `res.partner.category.csv` -> modelo `res.partner.category`. El sufijo
  tras `-` se ignora (`res.partner.category-extra.csv` tambien vale).
- Delimitador `,`, comillas `"`, encoding UTF-8 (docstring convert.py:673-676).
- Primera fila = cabecera de campos (`fields = next(reader)`, convert.py:681).
- Columna **`id`** = external id del registro (necesaria salvo en modo `init`,
  convert.py:683-685). Sin `id` en update aborta con error.
- Lineas totalmente vacias se filtran (convert.py:687-691).
- Se carga con `env[model].load(fields, datas)` (convert.py:700); el `noupdate` del manifest se
  propaga via context (convert.py:693-699).
- Si `load` devuelve algun error, **falla la instalacion** del modulo (convert.py:701-709).

## Columnas relacionales

- **many2one por external id:** sufijo `/id` -> `parent_id:id`. El valor es un xml_id.
  ```csv
  id,name,country_id:id
  mi_modulo.st_x,Estado X,base.es
  ```
- **many2one por valor (import estandar):** `field` a secas hace name-search (menos robusto;
  prefiere `:id`).
- **many2many por external ids:** `tag_ids:id` con xmlids separados por coma dentro de la celda
  (entre comillas):
  ```csv
  id,name,category_id:id
  mi_modulo.p1,Acme,"base.res_partner_category_0,base.res_partner_category_1"
  ```
- `field:id` (con dos puntos) es la notacion del importador estandar (`load`) para "el valor es
  un external id". El loader CSV usa exactamente el mismo motor que la importacion de la UI.

## Orden de carga en el manifest

```python
"data": [
    "security/ir.model.access.csv",   # 1. seguridad primero
    "data/grupos.xml",                # 2. grupos (referidos por records y vistas)
    "data/secuencias.xml",
    "data/parametros.xml",
    "views/acciones.xml",             # 3. acciones antes que menus
    "views/vistas.xml",
    "views/menus.xml",                # 4. menus al final (ref a acciones)
],
"demo": [
    "demo/demo_data.xml",
],
```
Reglas:
- Un fichero que hace `ref` a un xml_id debe declararse **despues** del fichero que lo crea.
  El loader procesa los ficheros en el orden del manifest; un `ref` no resuelto = error.
- `ir.model.access.csv` (seguridad) suele ir primero.
- `demo` solo se carga si la BD tiene demo activada; se omite con `--without-demo=all` o si la
  BD se creo sin demo. NO uses `demo` para datos que el modulo necesita en produccion.
- La declaracion `data`/`demo` y el scaffolding del manifest los gestiona
  `odoo-module-creator`.

## data vs demo

| | `data` | `demo` |
|--|--------|--------|
| Se carga en | todas las instalaciones | solo si demo activo |
| Proposito | config y estructura imprescindible | datos de ejemplo/pruebas |
| `--without-demo` | no afecta | omite la carga |

## Diffs v17 -> v18 -> v19

- `convert_csv_import` (nombre-fichero->modelo, columna `id`, `load`) es **identico** v17-v19.
- v19: `convert_file` deprecia el argumento `kind` (warning, no afecta a CSV/manifest) y anade
  type hints. El comportamiento de carga es el mismo.
- La notacion `field/id`, `field:id` y m2m por xmlids es estable en las tres versiones (es del
  motor `load`, no de convert).
