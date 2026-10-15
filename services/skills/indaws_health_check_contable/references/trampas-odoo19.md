# Trampas de Odoo 19 ya pagadas

Varias no dan error: devuelven cero y el chequeo sale en verde.

- **Impuestos y posiciones fiscales viven en la compañía MADRE.** Filtrar por `company_id =
  <hija>` devuelve cero y pinta verde. Cadena de compañías subiendo por `parent_id` y filtrar
  `company_id in CHAIN + [False]` (lo hace `carga.config()`).
- `account.fiscal.position.tax` ya no existe: el mapeo es `tax_ids` (m2m) + `account_ids`.
- `budget.line.account_id` es una **cuenta analítica**, no del PGC. La desviación se lee de
  `achieved_amount`.
- `account.account.deprecated` → `active`. `res.groups.users` → `user_ids`.
  **`res.users.groups_id` → `group_ids`**: el nombre viejo revienta la consulta.
- `account.move.l10n_es_edi_facturae_xml_id` no está almacenado: no va en un dominio. Detectar
  Facturae por `ir.attachment` y por los DIR3 del tercero.
- `analytic_distribution` es un JSON: no se filtra por plan en un dominio. Leerlo y mapear cuenta
  analítica → plan a mano.
- `account.analytic.plan` no tiene `company_id`: pedirlo revienta la consulta.
- **Diario de cierre y apertura**: excluirlo de los recuentos de asientos, pero **no de los
  saldos del balance**: ahí vive también la regularización, y quitarlo entero dejó el pasivo
  589.866,40 € corto. Para leer un ejercicio cerrado, medir a 1 de enero del año siguiente (cierre
  y apertura ya neutralizados). Dentro de ese diario la apertura no es el cierre: filtrar por
  fecha de fin de ejercicio.
- **Saldos cruzados en todas las cuentas a cobrar y a pagar, también sin tercero** (chequeo 13).
  Mirar solo 400 y 430 por tercero dejó pasar 480.542 € deudores en la 410 en pagos de banco sin
  cruzar. Anticipos (407, 438) tienen el signo contrario por naturaleza y se excluyen.
- El plan de cuentas puede pertenecer a la madre: `company_ids = [1]` en la compañía 2 es correcto.
- El plan de cuentas puede estar **en inglés**: clasificar por código y `account_type`, nunca por
  nombre.
- **Topes de carga**: `carga.py` pide asientos con tope 20.000 y apuntes con tope 100.000. Si se
  alcanza avisa y lo apunta en `truncados`: con la carga incompleta los chequeos afectados no son
  fiables y hay que subir el tope antes de entregar.
- **`/jsonrpc` está obsoleto en v19**: `conn.py` usa JSON2 en v19+ (detectado o `ODOO_PROTOCOL`)
  y JSON-RPC en versiones anteriores.
