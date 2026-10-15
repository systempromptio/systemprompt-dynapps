# Verificación runtime DIFERENCIAL A/B (el corazón de la skill)

Por qué es obligatoria: un módulo que altera un cálculo fiscal puede **instalar limpio, tener
tests verdes y calcular bien su propio régimen** y aun así **romper los casos que no son su
objetivo**. Ejemplos reales (REAV v19):
- Un filtro `if line.display_type: continue` (`display_type=='product'` es truthy **desde v16**, no solo en v19 — medido 2026-08-17) que no
  acota → el override se dispara donde no debe (over-reach) o se vuelve no-op.
- La estrategia `base_line` inyecta/recalcula sobre TODAS las líneas de venta, no solo las del
  régimen → una factura estándar 21% también queda alterada.
- Los tests del módulo solo asertan el caso REAV (`amount_tax`) y **nunca** una venta normal, así
  que la regresión pasa en verde.

Conclusión: el objeto NO es "¿el régimen calcula bien?". Es **"¿el cambio está acotado a su
objetivo y deja el resto legalmente correcto e idéntico?"**. Eso se mide con un **diferencial
A/B**, no con una sola DB.

## Procedimiento diferencial A/B

### 1. Batería de casos (definir ANTES de tocar runtime)
- **Caso(s) objetivo**: la operación que el módulo DEBE cambiar (p.ej. venta REAV / con la
  posición fiscal del régimen).
- **Casos no-objetivo** (los que DEBEN quedar intactos y legalmente correctos): venta nacional
  estándar 21%, venta con otro tipo (10%/4%), venta exenta/ISP si aplica, abono/rectificativa,
  compra, factura multi-línea. Elige los que el estándar + l10n_es soportan sin el módulo.

### 2. DB-A (referencia legal, SIN el módulo)
```bash
docker exec <db> psql -U odoo -d postgres -c "DROP DATABASE IF EXISTS fisc_A;"
docker exec <db> psql -U odoo -d postgres -c "CREATE DATABASE fisc_A OWNER odoo;"
docker exec <odoo> odoo -c <conf> -d fisc_A -i l10n_es,sale_management --stop-after-init --http-port=<pA>
```
Ejecuta la batería en `odoo shell` y captura por cada caso: `amount_untaxed`, `amount_tax`,
`amount_total`, base/cuota por línea, número/serie, `invoice_date`/`date`. Esto ES la salida
legalmente correcta de los no-objetivo (por construcción del estándar). El caso objetivo aquí
sale "como estándar" (todavía sin el régimen) — es tu punto de comparación.

### 3. DB-B (CON el módulo)
```bash
docker exec <db> psql -U odoo -d postgres -c "CREATE DATABASE fisc_B OWNER odoo;"
docker exec <odoo> odoo -c <conf> -d fisc_B -i <modulo>,l10n_es,sale_management --stop-after-init --http-port=<pB>
```
Ejecuta **la MISMA batería con el mismo script** y captura los mismos números.

### 4. Diff (el veredicto)
- **No-objetivo: B == A** (y legalmente correcto contra la cita). Cualquier delta = **REGRESIÓN**
  (over-reach). Reporta el número A vs B y el caso afectado.
- **Objetivo: B cambia respecto a A exactamente a lo que la ley exige, y no más.**
  - No cambia → `NO-OP` (el módulo no hace su trabajo sobre su propio objetivo).
  - Cambia mal (número ≠ fórmula legal) → `TARGET-WRONG`.
  - Cambia bien y solo él → `SCOPED-OK`.

### 5. Comprobar el filtro del override (causa raíz)
Antes de dar `SCOPED-OK`, verifica en shell que el `filtered(...)`/dominio del override devuelve
**exactamente** el subconjunto objetivo y **solo** ese sobre la batería (un recordset vacío o
demasiado ancho es la raíz del no-op o del over-reach). Comprueba también el efecto de
`auto_install` y de la limpieza dependiente de `onchange` (crea el caso no-objetivo de forma
**programática**, no por Form, para exponer filtros que solo actúan en onchange).

### 6. Documento (cuando el cambio toca la factura como documento)
Si el cambio afecta a lo que ve el cliente (menciones, desglose de cuota en régimen de margen),
renderiza el report en DB-B y compáralo con DB-A:
```python
html = env['ir.actions.report']._render_qweb_html('account.account_invoices', move.ids)[0]
```
El art. 142 / RD 1619/2012 rigen el PDF, no el asiento. Pero recuerda: aquí el eje es la
regresión — que el documento de una operación **no-objetivo** no haya cambiado.

### 7. Limpiar
`DROP DATABASE fisc_A; DROP DATABASE fisc_B;`

## Salida de esta fase
Por caso de la batería: salida legal esperada (cita), observado en A, observado en B, y veredicto
(`SCOPED-OK`/`REGRESIÓN`/`TARGET-WRONG`/`NO-OP`/`UNVERIFIED`/`RUNTIME-N/A`) con el snippet usado.
Alimenta la matriz de regresión de `assets/compliance-matrix.md`.
