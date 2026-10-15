# Roles - Deteccion y Comportamiento

## Deteccion de Rol

**OBLIGATORIO**: Al iniciar el skill, preguntar al usuario su rol usando AskUserQuestion:

```
Ticket Report - Informe de Tickets Helpdesk

Cual es tu rol en el equipo?
```

**Opciones**:
1. **Desarrollador** - Desarrollador del equipo tecnico
2. **Consultor** - Consultor funcional / Project Manager
3. **Team Leader** - Lider tecnico con desarrolladores a cargo
4. **Project Manager** - PM con personas a cargo (solo campo "Asignado a")

---

## Desarrollador

Los desarrolladores tienen tickets asignados en **dos campos diferentes**:
- **x_studio_assigned_developer_id** (Desarrollador asignado): responsable tecnico del desarrollo
- **user_id** (Asignado a): responsable directo de la gestion

**TODOS los informes para desarrolladores DEBEN mostrar DOS secciones separadas:**

1. "Tickets como Desarrollador asignado": filtrados por x_studio_assigned_developer_id
2. "Tickets como Asignado a": filtrados por user_id

**IMPORTANTE**: Un mismo ticket puede aparecer en ambas secciones. NO eliminar duplicados entre secciones, ya que cada seccion representa un rol diferente.

Para cada informe, ejecutar los scripts **dos veces**:
```bash
./scripts/get_user_tickets.sh <user_id> open "" "" developer
./scripts/get_user_tickets.sh <user_id> open "" "" assigned
```

Para estadisticas y carga de trabajo, calcular estadisticas **separadas** para cada campo y luego un **resumen combinado**.

### Mensaje de bienvenida

```
Ticket Report - Modo Desarrollador

Conectado como: [nombre del usuario]

Puedo ayudarte con:
- **Informe semanal completo** (recomendado): "informe semanal" / "dame mi informe"
  -> Tickets abiertos + seguimiento >1 semana + cerrados esta semana + estadisticas
- Tickets abiertos: "Que tickets tengo?" (muestra tickets como dev asignado Y como asignado a)
- Tickets cerrados: "Que he cerrado esta semana?"
- Estadisticas: "Media de resolucion de este mes"
- Carga de trabajo: "Tengo mucha carga?" / "Detectar atascos"

Que necesitas?
```

---

## Consultor

Los consultores solo tienen tickets en el campo estandar:
- **user_id** (Asignado a): unico campo relevante

**Los informes para consultores muestran UNA sola seccion** sin distincion de campos.

```bash
./scripts/get_user_tickets.sh <user_id> open "" "" assigned
```

### Mensaje de bienvenida

```
Ticket Report - Modo Consultor

Conectado como: [nombre del usuario]

Puedo ayudarte con:
- Tickets abiertos: "Que tickets tengo?" / "Tickets prioritarios"
- Tickets cerrados: "Que he cerrado esta semana?"
- Estadisticas: "Media de resolucion de este mes"
- Carga de trabajo: "Tengo mucha carga?" / "Detectar atascos"

Que necesitas?
```

---

## Team Leader

Los Team Leaders tienen su propio informe individual (igual que Desarrollador) MAS un **informe de equipo** que agrupa el informe semanal de cada desarrollador a su cargo.

**Identificar desarrolladores a cargo**:
1. Buscar el hr.employee del TL usando su user_id
2. Buscar empleados con parent_id = <employee_id_del_TL> y department_id = 4
3. Para cada desarrollador, obtener su user_id y generar su informe individual

**Informe de equipo** contiene para CADA desarrollador:
- Tickets abiertos (como dev asignado, seccion unica)
- Tickets con > 1 semana que requieren seguimiento (con accion recomendada)
- Tickets cerrados esta semana
- Estadisticas de resolucion del mes en curso

**Al final**: tabla resumen del equipo con totales y alertas.

**Triggers**: "informe de mi equipo", "informe del equipo", "tickets de mi equipo", "como van mis desarrolladores"

### Mensaje de bienvenida

```
Ticket Report - Modo Team Leader

Conectado como: [nombre del usuario]
Desarrolladores a cargo: [lista de nombres]

Puedo ayudarte con:
- **Informe semanal de mi equipo** (recomendado): "informe de mi equipo"
  -> Informe completo por cada desarrollador + resumen
- **Mi informe individual**: "mis tickets" / "mi informe semanal"
- Tickets de un desarrollador concreto: "tickets de [nombre]"

Que necesitas?
```

---

## Project Manager

Los Project Managers gestionan personas que tienen tickets en el campo estandar user_id (Asignado a). El PM ve el estado de su equipo para poder hacer seguimiento.

**Identificar personas a cargo**:
1. Buscar el hr.employee del PM usando su user_id
2. Buscar empleados con parent_id = <employee_id_del_PM> y active = true
3. Para cada persona, obtener su user_id y generar su informe

**Diferencia clave respecto a Team Leader**:
- Solo se usa el campo user_id (Asignado a) -- NUNCA x_studio_assigned_developer_id
- Ejecutar los scripts siempre con field=assigned
- Las personas a cargo pueden ser de cualquier departamento (no solo Desarrollo)

**Informe de equipo PM** contiene para CADA persona a cargo:
- Tickets abiertos (solo como "Asignado a", una seccion)
- Tickets con > 1 semana que requieren seguimiento (con accion recomendada)
- Tickets cerrados esta semana
- Estadisticas de resolucion del mes en curso

**Al final**: tabla resumen del equipo con totales y alertas.

**Triggers**: "informe de mi equipo", "tickets de mi equipo", "como van mis consultores", "seguimiento de mi equipo"

### Mensaje de bienvenida

```
Ticket Report - Modo Project Manager

Conectado como: [nombre del usuario]
Personas a cargo: [lista de nombres]

Puedo ayudarte con:
- **Informe semanal de mi equipo** (recomendado): "informe de mi equipo"
  -> Informe completo (tickets asignados a cada persona) + resumen
- **Mi informe individual**: "mis tickets" / "mi informe semanal"
- Tickets de una persona concreta: "tickets de [nombre]"

Que necesitas?
```

### Ejecucion para Project Manager

```bash
# Para cada persona a cargo (field=assigned, UNA sola seccion)
./scripts/get_user_tickets.sh <uid_persona> open "" "" assigned

# Cerrados esta semana
./scripts/get_closed_tickets.sh <uid_persona> <lunes> <hoy> assigned

# Estadisticas del mes
./scripts/get_closed_tickets.sh <uid_persona> <primer_dia_mes> <hoy> assigned | python3 scripts/calculate_ticket_stats.py resolution
```
