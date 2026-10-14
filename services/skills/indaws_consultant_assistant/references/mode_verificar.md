# Consultant Assistant - Mode VERIFICAR

## Requirements

- Credenciales Odoo: ODOO_URL, ODOO_DB, ODOO_KEY, ODOO_USER
- project_id: ID numerico del proyecto a auditar

## Workflow

### Paso 1: Ejecutar auditoria

```bash
python3 scripts/verify_project_standards.py \
  --project-id {project_id} \
  --pilot-dir {ruta_odoo_pilot}
```

El script ejecuta los 12 estandares via odoo-pilot y devuelve JSON con resultados.
Lee `references/estandares_proyecto.md` para entender que evalua cada estandar.

### Paso 2: VALIDACION dashboard obligatoria

Presentar tabla resumen ANTES de generar recomendaciones:

```
| # | Estandar | Estado | Severidad | Detalle |
|---|----------|--------|-----------|---------|
| 1 | partner_id asignado | OK | BLOCKING | ... |
| 2 | Knowledge configurado | KO | BLOCKING | ... |
...

Score de cumplimiento: 75% (8/12 OK, 2 KO, 2 WARNING)
```

Esperar confirmacion del usuario antes de continuar.

### Paso 3: Recomendaciones

Para cada estandar KO o WARNING:
1. Explicar que significa el fallo
2. Indicar la accion correctiva
3. Sugerir el skill concreto para corregirlo (si aplica)
