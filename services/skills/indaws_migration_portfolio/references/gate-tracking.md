# Gate tracking — Estado multi-cliente por gate

La **fuente de verdad de los gates es `migration-assistant`** (orquestador maestro
del ciclo). Esta skill **no define ni recalcula** gates: los **lee** de ahí y
**consolida** una vista multi-cliente de toda la cartera.

> Conexión a Odoo: `references/shared/odoo-connection.md` (siempre `odoo-pilot`).

## Gates de la migración Dynapps (de DESIGN §3.5)
La facturación de Julio se ancla a estos hitos (ver `migration-billing-tracker`):

| Gate | Significado | Quién valida |
|---|---|---|
| `report_sent` | Informe + presupuesto enviados a Janik | Indaws |
| `accepted` | Dynapps acepta → sale.order confirmado | Dynapps |
| `delivered` | Entrega: BD migrada + módulos en GitLab | Indaws |
| `production` | Salida a producción | Dynapps / cliente final |

> El estado **detallado** y la definición canónica de cada gate los posee
> `migration-assistant`. Aquí solo se mapea a nivel cartera.

## Vista consolidada
Para cada migración, mostrar: `client`, `assignee`, `status`, gate actual,
% avance, `est_hours`, riesgo. Tabla multi-cliente ordenada por riesgo/fecha.

- El **roll-up ejecutivo** (semáforos, riesgo agregado) se **delega** a
  `executive-projects-tracker`. No re-implementar el dashboard aquí.
- La vista de **cobro por gate** es de `migration-billing-tracker` (Fase 3); esta
  skill solo le aporta el estado consolidado.

## Almacenamiento (estado persistido)
Guardar la cartera en `${CLAUDE_PLUGIN_DATA}/migration-portfolio/portfolio.json`
(sobrevive a upgrades del skill; NO dentro del directorio del skill). Una entrada
por migración con los campos de `intake.md` + `assignment.md` + gate actual.

Estructura (JSON compacto, una migración por objeto):
```json
{
  "migrations": [
    {"client":"", "contact":"Janik", "version_src":"", "version_dst":"19.0",
     "db_size":"", "est_hours":0, "est_eur":0, "assignee":"", "target_date":"",
     "priority":2, "status":"intake", "gate":"report_sent", "project_id":null,
     "sale_order":null, "drive_link":"", "zip_name":"", "intake_date":""}
  ]
}
```

## Reglas
- **No crear ni mutar gates** fuera de `migration-assistant`; solo reflejar su lectura.
- **No re-estimar** ni recalcular horas.
- **NUNCA enviar correo desde Odoo** al actualizar estados.
- Persistir tras cada cambio; el JSON es la cartera viva que consumen Fase 3 y 4.
