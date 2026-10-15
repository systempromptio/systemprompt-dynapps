# Tareas Genéricas Obligatorias

Este documento define las tareas que SIEMPRE deben incluirse en toda planificación de proyecto Odoo, independientemente de los módulos implementados.

---

## Tareas Obligatorias

### 1. Configuraciones Generales

**Nombre de tarea**: `Configuraciones Generales`

**Descripción**:
Incluye todas las configuraciones iniciales del proyecto que no pertenecen a ningún módulo específico pero son necesarias para poner en marcha el sistema.

**Tareas específicas incluidas:**
- Alta de servidor de Odoo
- Alta de bases de datos (producción + staging)
- Configuración de copias de seguridad
- Configuración de servidor de correos de salida
- Prueba de envío de email
- Configuración de correos de entrada
- Prueba de entrada de email
- Alta de usuarios iniciales y permisos
- Configuración de idiomas
- Alta de compañía(s)
- Reglas entre compañías (si multicompañía)

**Reglas de Cálculo de Horas:**

| Situación | Horas Asignadas | Justificación |
|-----------|-----------------|---------------|
| **Estándar** | 12h | Proyecto sin complejidades técnicas |
| **Multicompañía** | 16h | +4h por configuración de reglas inter-compañía |
| **Multi-idioma (>2)** | 14h | +2h por configuración adicional de idiomas |
| **Desarrollo de servidor custom** | 20h | +8h si el servidor requiere configuración especial |
| **Integraciones con correo complejo** | 14h | +2h si hay múltiples buzones o reglas complejas |

**Default**: Si no hay indicaciones especiales en el análisis, usa **12 horas**.

**Posición en cronograma**: Siempre **al inicio del proyecto** (primera(s) semana(s)).

**Dependencias**: Ninguna (es la primera tarea)

**Equipo**: Técnico + Consultor líder

---

### 2. Soporte en la Salida a Producción

**Nombre de tarea**: `Soporte en Salida a Producción`

**Descripción**:
Soporte intensivo durante el primer mes de operación en producción. Incluye resolución de incidencias urgentes, ajustes menores de configuración, formación adicional sobre casos no previstos y cierre de dudas operativas.

**Reglas de Cálculo de Horas:**

Basado en la **bolsa de horas total del proyecto**:

| Bolsa de Horas Total | Horas de Soporte | Justificación |
|---------------------|------------------|---------------|
| **Hasta 100h** | 18h | Proyecto pequeño, pocas complejidades |
| **100h - 300h** | 24h | Proyecto mediano |
| **400h - 700h** | 32h | Proyecto grande |
| **Más de 700h** | 48h | Proyecto muy grande, alta complejidad |

**Distribución temporal**:
- **Semana 1 post go-live**: 40% de las horas (soporte muy intensivo)
- **Semana 2 post go-live**: 30% de las horas
- **Semana 3 post go-live**: 20% de las horas
- **Semana 4 post go-live**: 10% de las horas (soporte residual)

**Posición en cronograma**: Siempre **después del go-live**, durante el **último mes del proyecto**.

**Dependencias**: Requiere que el go-live haya ocurrido.

**Equipo**: Consultor funcional + Desarrollador (standby para incidencias técnicas)

---

### 3. Reuniones / Formaciones Semanales

**Nombre de tarea**: `Reuniones / Formaciones Semanales`

**Descripción**:
Reuniones de seguimiento semanal con el cliente para validar avances, resolver dudas, ajustar prioridades y dar formación continua sobre los módulos que se van configurando.

**Reglas de Cálculo de Horas:**

**Fórmula base**: `3 horas por semana × número de semanas del proyecto`

**Ejemplo:**
- Proyecto de 12 semanas: 3h × 12 = **36 horas**
- Proyecto de 24 semanas (6 meses): 3h × 24 = **72 horas**

**Ajustes según contexto:**

| Situación | Ajuste | Horas/Semana |
|-----------|--------|--------------|
| **Cliente muy disponible** | -0.5h/semana | 2.5h |
| **Cliente con poca disponibilidad** | +0.5h/semana | 3.5h |
| **Proyecto con muchos stakeholders** | +1h/semana | 4h |
| **Proyecto remoto 100%** | +0.5h/semana | 3.5h (más tiempo de coordinación) |

**Default**: Si no hay indicaciones especiales, usa **3 horas por semana**.

**Distribución de las 3 horas semanales:**
- 1h: Reunión de seguimiento con cliente (estado del proyecto, próximos pasos)
- 1h: Sesión de formación/validación de módulos configurados
- 1h: Resolución de dudas operativas y refinamiento de requisitos

**Posición en cronograma**: **Durante TODO el proyecto**, desde el inicio hasta el cierre (incluyendo el mes de soporte post go-live).

**Dependencias**: Ninguna (es transversal a todo el proyecto).

**Equipo**: Consultor funcional (líder del proyecto)

---

## Cálculo Rápido de Tareas Genéricas

### Plantilla de Cálculo

Para facilitar el cálculo rápido, usa esta plantilla:

```markdown
**Tareas Genéricas:**
1. Configuraciones generales: 12h (estándar)
2. Soporte post go-live: [18h / 24h / 32h / 48h según bolsa]
3. Reuniones/formaciones: [3h × número de semanas]

**Total tareas genéricas**: [suma] horas
```

### Ejemplos Completos

#### Ejemplo 1: Proyecto Pequeño (100h, 12 semanas)

```
1. Configuraciones generales: 12h
2. Soporte post go-live: 18h (bolsa < 100h)
3. Reuniones/formaciones: 3h × 12 semanas = 36h

Total: 66h (66% de la bolsa se va en tareas genéricas)
Disponible para módulos: 34h
```

**Nota**: En proyectos pequeños, las tareas genéricas consumen una proporción muy alta de la bolsa.

#### Ejemplo 2: Proyecto Mediano (300h, 20 semanas)

```
1. Configuraciones generales: 12h
2. Soporte post go-live: 24h (bolsa 100-300h)
3. Reuniones/formaciones: 3h × 20 semanas = 60h

Total: 96h (32% de la bolsa)
Disponible para módulos: 204h
```

#### Ejemplo 3: Proyecto Grande (600h, 6 meses = 24 semanas)

```
1. Configuraciones generales: 12h (o 16h si multicompañía)
2. Soporte post go-live: 32h (bolsa 400-700h)
3. Reuniones/formaciones: 3h × 24 semanas = 72h

Total: 116h (19% de la bolsa)
Disponible para módulos: 484h
```

**Nota**: En proyectos grandes, las tareas genéricas tienen un peso porcentual menor, dejando más margen para módulos y desarrollos.

---

## Reglas de Validación

### Coherencia de Tareas Genéricas

1. **Reuniones/formaciones no pueden ser < 20% de la bolsa total** en proyectos pequeños
   - Si calculas que las reuniones son <20% de la bolsa, probablemente el proyecto está mal dimensionado o la bolsa es insuficiente.

2. **Reuniones/formaciones no deben ser > 30% de la bolsa total** en proyectos grandes
   - Si calculas que las reuniones son >30%, considera reducir la frecuencia (2.5h/semana en lugar de 3h).

3. **Soporte post go-live debe ser proporcional a la complejidad**
   - Un proyecto de 600h con solo 18h de soporte es insuficiente.
   - Un proyecto de 80h con 48h de soporte es excesivo.

---

## Casos Especiales

### Proyecto con Go-Live Escalonado

Si el go-live se hace por fases (ej. primero Ventas, después Inventario, luego Contabilidad):

- **Multiplica el soporte post go-live**: Añade 50% de las horas por cada fase adicional.
- **Ejemplo**: Go-live normal = 32h, Go-live en 2 fases = 32h + 16h = 48h

### Proyecto Remoto 100%

- **Aumenta reuniones en +0.5h/semana**: Más tiempo de coordinación y validación a distancia.
- **Aumenta soporte post go-live en +20%**: Menos eficiencia en resolución remota de incidencias.

### Cliente con Equipo Técnico Interno

- **Reduce soporte post go-live en -30%**: El cliente puede resolver incidencias menores.
- **Mantiene reuniones estándar**: La coordinación sigue siendo necesaria.

---

## Notas Finales

- **Estas tareas NO son negociables**: Siempre deben incluirse en la planificación.
- **Son overhead del proyecto**: No aportan funcionalidad directa pero son críticas para el éxito.
- **Validar con el usuario**: Si las tareas genéricas consumen >50% de la bolsa en un proyecto pequeño, comunícalo al usuario - probablemente la bolsa es insuficiente.
