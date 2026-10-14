# Output Format — Formato del inventario de salida

El inventario se presenta al usuario (o al skill que lo invoco) en este formato:

---

## Cabecera

```
## Inventario: <module_name>
Version: <version del manifest>
Depends: <lista de dependencias>
Clasificacion: SIMPLE | STANDARD | COMPLEX | ADVANCED
Lineas de codigo: <total Python> py / <total XML> xml / <total JS> js
```

---

## Modelos

```
### Modelos (<N> total)

| Modelo | Tipo | Campos | Computed | Constraints | Onchanges | Actions | CRUD |
|--------|------|--------|----------|-------------|-----------|---------|------|
| my.model | Nuevo | 12 | 3 | 1 | 2 | 4 | create, write |
| res.partner | Extension | +2 | 1 | 0 | 0 | 0 | — |
| my.wizard | Wizard | 5 | 0 | 0 | 1 | 1 | — |
```

Para cada modelo con logica relevante, detallar:

```
#### my.model (nuevo)
- Hereda: mail.thread, mail.activity.mixin
- Campos computados: margin_percent (stored), display_name, line_count
- Constraints: _check_dates (api.constrains), unique_reference (SQL)
- Onchanges: _onchange_partner_id, _onchange_product_id
- Actions: action_confirm, action_cancel, action_draft, action_send
- CRUD: create (custom sequence), write (state validation)
- Cron: _cron_check_expiry
- Excepciones: 3x UserError, 1x ValidationError
- State field: draft → confirmed → done → cancelled
```

---

## Vistas

```
### Vistas (<N> total)

| Vista | Tipo | Modelo | Herencia |
|-------|------|--------|----------|
| my_model_view_form | form | my.model | — |
| my_model_view_tree | tree | my.model | — |
| my_model_view_search | search | my.model | — |
| res_partner_view_form_inherit | form | res.partner | base.view_partner_form |
```

---

## Seguridad

```
### Seguridad

**Grupos definidos:**
- group_my_model_user (Usuario)
- group_my_model_manager (Administrador)

**Reglas de acceso (ir.model.access.csv):**
| Modelo | Grupo | Read | Write | Create | Unlink |
|--------|-------|------|-------|--------|--------|
| my.model | user | 1 | 1 | 1 | 0 |
| my.model | manager | 1 | 1 | 1 | 1 |

**Record rules:**
- own_records_rule: Usuarios solo ven sus propios registros
- company_rule: Aislamiento multi-compania
```

Si no hay ficheros de seguridad y hay modelos nuevos: **FLAG CRITICO**.

---

## Controladores

```
### Controladores (<N> rutas)

| Ruta | Metodo | Auth | Tipo | Website |
|------|--------|------|------|---------|
| /api/my_model | GET | user | json | No |
| /my/orders | GET | public | http | Si |
| /my/order/<int:id> | POST | user | http | Si |
```

---

## JavaScript

```
### JavaScript (<N> componentes)

| Componente | Tipo | Servicios | RPC |
|------------|------|-----------|-----|
| MyWidget | OWL Component | rpc, notification | Si |
| CustomField | AbstractField | — | No |
```

---

## Traducciones

```
### Traducciones

| Idioma | Cadenas | Fichero |
|--------|---------|---------|
| es_ES | 47 | i18n/es_ES.po |
| en_US | 47 | i18n/en_US.po (base) |
```

---

## APIs externas

```
### Dependencias externas

| Libreria | Uso | Necesita mock |
|----------|-----|---------------|
| requests | API de envio (carrier) | Si |
| paramiko | SFTP export | Si |
```

---

## Resumen de prioridades

```
### Prioridades de atencion

- CRITICO: [lista de findings criticos, ej: falta seguridad]
- ALTO: [lista de findings altos, ej: CRUD override sin test]
- MEDIO: [lista de findings medios]
- INFO: [lista informativa, ej: deprecated APIs detectadas]
```

---

## Notas para skills consumidores

Cuando otro skill invoca este inventario:

- **odoo-test-generator**: Usa la tabla de modelos + clasificacion de campos para generar el plan de tests
- **odoo-code-auditor**: Usa las metricas de complejidad + patrones deprecated + seguridad
- **odoo-database-analyzer**: Usa la clasificacion del modulo + lineas de codigo + herencias
- **odoo-module-documenter**: Usa modelos + vistas + traducciones para generar documentacion
- **odoo-module-upgrader**: Usa patrones deprecated + JS + vistas para planificar migracion
