# Accounting Auditor — Workflow Detail

## Paso 0: Recopilación de Datos

Antes de hacer cualquier análisis, **SIEMPRE** recopila:

1. **Credenciales Odoo** (si no están configuradas):
   - URL, base de datos, API Key, usuario

2. **Rango de fechas a auditar**: Fecha inicio y fecha fin. Obligatorio.

3. **Company ID** (opcional): Si la instancia es multi-compañía. Default: compañía del usuario.

**DETENTE** y espera la respuesta del usuario antes de continuar.

## Paso 1: Extracción de Datos de Odoo

```bash
export ODOO_URL="<url>"
export ODOO_DB="<db>"
export ODOO_KEY="<api_key>"
export ODOO_USER="<usuario>"

./scripts/fetch_accounting_data.sh <date_from> <date_to> [--company-id X] --pilot-dir /path/to/odoo-pilot/scripts
```

**El script obtiene** (10 conjuntos de datos):

1. **Plan contable** (`account.account`): code, name, account_type, reconcile, tax_ids
2. **Asientos** (`account.move`): state, date, move_type, journal_id, partner_id, amount_total
3. **Líneas contables** (`account.move.line`): account_id, debit, credit, partner_id, reconciled, amount_residual, tax_ids
4. **Diarios** (`account.journal`): type, default_account_id
5. **Extractos bancarios** (`account.bank.statement`): state, date, balance_end_real
6. **Datos de compañía** (`res.company`): lock dates
7. **Partners con facturas** (`res.partner`): vat, country_id
8. **Módulos de localización** (`ir.module.module`): l10n_es*, estado
9. **Activos fijos** (`account.asset`): state, original_value, book_value
10. **Impuestos** (`account.tax`): name, amount, type_tax_use

**Validaciones post-extracción**:
- ✅ Conexión exitosa y datos extraídos
- ✅ Plan contable disponible
- ⚠️ Sin módulo l10n_es → Advertencia pero continuar
- ❌ Sin asientos en el período → No hay datos que auditar

## Paso 2: Auditoría

```bash
python3 ./scripts/audit_accounting.py < /tmp/audit_data.json > /tmp/audit_results.json
```

**El script ejecuta ~42 comprobaciones en 10 categorías**:

1. Plan General Contable (7 checks)
2. Períodos y cierres (4 checks)
3. Asientos contables (7 checks)
4. Conciliación bancaria (4 checks)
5. IVA y retenciones (5 checks)
6. Clientes y proveedores (6 checks)
7. Nóminas y Seguridad Social (6 checks)
8. Inmovilizado y amortización (4 checks)
9. Cuentas prohibidas (2 checks)
10. Coherencia general y ratios (6 checks)

Cada hallazgo incluye: categoría, severidad (🔴 crítico / 🟡 aviso / ℹ️ info), descripción, datos concretos e impacto.

Consulta `./references/audit_rules.md` para la documentación detallada de cada regla.

## Paso 2.5: Validación del Dashboard (OBLIGATORIO)

**DETENTE** y presenta al usuario un dashboard resumen:

```markdown
## DASHBOARD: Auditoría Contable {date_from} — {date_to}

### Score de Salud Contable: XX/100

| Categoría | Críticos | Avisos | Info | Score |
|-----------|----------|--------|------|-------|
| Plan Contable | X | X | X | XX |
| Períodos y cierres | X | X | X | XX |
| ...

### Top Hallazgos Críticos
1. {hallazgo_1}
2. {hallazgo_2}

**¿Los datos son correctos? ¿Genero el informe completo?**
```

**Espera la respuesta del usuario** antes de continuar.

## Paso 3: Generación del Informe

Solo tras validación. Genera informe Markdown:

**Fichero de salida**: `informe_auditoria_{company}_{YYYY}_{MM}.md`

**Estructura** (11 secciones):

1. Resumen ejecutivo + Score de salud
2. Plan General Contable
3. Períodos y cierres
4. Asientos contables
5. Conciliación bancaria
6. IVA y retenciones
7. Clientes y proveedores
8. Nóminas y Seguridad Social
9. Inmovilizado y amortización
10. Coherencia general y ratios
11. Plan de acción priorizado

## Notas contables

- Referencia: Plan General Contable español (RD 1514/2007)
- Todas las comprobaciones asumen contabilidad en EUR
- Los checks de IVA asumen tipos españoles (21%, 10%, 4%, exento)
- Los checks de retenciones asumen IRPF español
- Lock dates son por compañía
- Formato importes: punto miles, coma decimales (ej: 12.345,67)
