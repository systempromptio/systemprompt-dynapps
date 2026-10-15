# Scripts - Documentacion y Uso

**OBLIGATORIO: SIEMPRE usar estos scripts para consultas a Odoo.**

NUNCA hacer llamadas directas a la API de Odoo. Los scripts ya incluyen:
- Contexto {"lang": "es_ES"}
- Deteccion automatica de protocolo (JSON-RPC/JSON2)
- **Paginacion automatica** (obtiene TODOS los registros, no solo los primeros 100)

---

## odoo_call.sh (Base)

Llamada generica a Odoo API. Detecta automaticamente si usar JSON-RPC (< 19.0) o JSON2 (>= 19.0).

```bash
./scripts/odoo_call.sh <model> <method> <domain_json> <fields_json> [kwargs_json]
```

---

## odoo_search_all.sh (Paginacion)

Busqueda paginada que obtiene **TODOS** los registros. Hace llamadas de 100 en 100.

```bash
./scripts/odoo_search_all.sh <model> <domain_json> <fields_json> [order]
```

---

## get_user_tickets.sh

Obtener tickets de un usuario, opcionalmente filtrado por estado y campo de usuario.

```bash
./scripts/get_user_tickets.sh <user_id> [status] [date_from] [date_to] [field]
```

**Parametros**:
- user_id: ID del usuario en res.users
- status: "open" (default), "all", "solved", "in_progress"
- date_from: Fecha inicio YYYY-MM-DD (opcional)
- date_to: Fecha fin YYYY-MM-DD (opcional)
- field: **"developer"** (default, usa x_studio_assigned_developer_id) o **"assigned"** (usa user_id)

**Ejemplos**:
```bash
# Desarrollador: tickets abiertos como dev asignado
./scripts/get_user_tickets.sh 1249 open "" "" developer

# Desarrollador: tickets abiertos como asignado a
./scripts/get_user_tickets.sh 1249 open "" "" assigned

# Consultor: tickets abiertos (solo usa user_id)
./scripts/get_user_tickets.sh 1249 open "" "" assigned
```

---

## get_closed_tickets.sh

Obtener tickets cerrados en un periodo.

```bash
./scripts/get_closed_tickets.sh <user_id> <date_from> <date_to> [field]
```

**Parametros**:
- field: "developer" (default) o "assigned" (igual que get_user_tickets.sh)

**Para Desarrolladores**: ejecutar dos veces (field=developer y field=assigned)
**Para Consultores**: ejecutar una vez (field=assigned)

---

## get_ticket_chatter.sh

Obtener mensajes del chatter de uno o varios tickets.

```bash
./scripts/get_ticket_chatter.sh <ticket_ids_json> [limit]
```

---

## calculate_ticket_stats.py

**USAR SIEMPRE para calculos estadisticos. NUNCA calcular manualmente.**

```bash
# Modo: resolution (media de resolucion)
echo '<tickets_json>' | python3 scripts/calculate_ticket_stats.py resolution

# Modo: workload (analisis de carga)
echo '<tickets_json>' | python3 scripts/calculate_ticket_stats.py workload
```
