# Payroll Accounting — Workflow Detail

## Paso 1: Ejecutar process_payroll.py

```bash
python3 skills/payroll-accounting/scripts/process_payroll.py \
  --excel "<ruta_al_excel>" \
  --month "<mes>" \
  --year "<año>" \
  --move-id <id_del_asiento> \
  --odoo-pilot-path "<ruta_a_odoo_pilot_scripts>"
```

El script produce un JSON con:
- Lista de empleados clasificados por departamento
- Líneas contables calculadas (debit/credit por cuenta)
- Totales de verificación (total debit = total credit)
- Errores encontrados (si los hay)

**Si el JSON contiene errores** (empleados no encontrados, sin departamento, departamentos sin mapeo):
- Mostrar los errores al usuario
- NO continuar hasta que el usuario los resuelva

## Paso 2: Tabla resumen para validación

Ejemplo de tabla:

```
| Cuenta  | Descripción                        | Debe      | Haber     |
|---------|------------------------------------|-----------|-----------|
| 640001  | NÓMINA FEBRERO 2026                | 12.345,67 |           |
| 640004  | NÓMINA FEBRERO 2026                |  8.901,23 |           |
| 642010  | NÓMINA FEBRERO 2026                |  3.456,78 |           |
| 642013  | NÓMINA FEBRERO 2026                |  2.345,67 |           |
| 475100  | NÓMINA FEBRERO 2026                |           |  4.567,89 |
| 465000  | NÓMINA FEBRERO 2026                |           | 18.901,23 |
| 476000  | NÓMINA FEBRERO 2026                |           |  3.580,23 |
|---------|------------------------------------|-----------|-----------|
| TOTAL   |                                    | 27.049,35 | 27.049,35 |
```

Mostrar también el desglose de empleados por departamento.

**Esperar confirmación explícita del usuario antes de continuar.**

## Paso 3: Crear account.move.line en Odoo

Para cada línea contable del JSON:

1. Buscar la cuenta contable por código:
```bash
<PILOT_DIR>/search_records.sh \
  account.account \
  '[["code","=","640001"]]' \
  '["id","code","name"]' \
  1
```

2. Crear la línea contable:
```bash
<PILOT_DIR>/create_record.sh \
  account.move.line \
  '{"move_id": <MOVE_ID>, "account_id": <ACCOUNT_ID>, "name": "NÓMINA FEBRERO 2026", "debit": 12345.67, "credit": 0}'
```

3. Repetir para cada línea (cuentas debit con credit=0, cuentas credit con debit=0).

4. Verificar que el asiento cuadra (total debit = total credit).

## Paso 4: Nota interna en el asiento

**IMPORTANTE**: NO usar `message_post` vía JSON-RPC — escapa el HTML del body.
Crear directamente un `mail.message`:

### Obtener subtype_id de mail.mt_note

```bash
<PILOT_DIR>/search_records.sh \
  ir.model.data \
  '[["module","=","mail"],["name","=","mt_note"]]' \
  '["res_id"]' 1
# → subtype_id = result[0]["res_id"]
```

### Obtener partner_id del usuario actual

```bash
<PILOT_DIR>/search_records.sh \
  res.users \
  '[["id","=",<UID>]]' \
  '["partner_id"]' 1
# → author_id = result[0]["partner_id"][0]
```

### Construir body HTML

```python
html_parts = []
for dept_name, employees in data["employees_by_department"].items():
    count = len(employees)
    names = "<br>".join(emp['odoo_name'] for emp in employees)
    html_parts.append(f"<p><strong>{dept_name} ({count}):</strong><br>{names}</p>")
body = "".join(html_parts)
```

### Crear mail.message directamente

```bash
<PILOT_DIR>/create_record.sh \
  mail.message \
  '{"model": "account.move", "res_id": <MOVE_ID>, "body": "<HTML>", "message_type": "comment", "subtype_id": <SUBTYPE_ID>, "author_id": <AUTHOR_ID>}'
```

Esto crea una nota interna visible en el chatter con HTML correctamente renderizado.
