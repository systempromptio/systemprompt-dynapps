# Fases de Implantación Odoo

Este documento define el modelo estándar de implantación de Odoo en Indaws, basado en una metodología Waterfall/Híbrida adaptada a proyectos ERP.

---

## Modelo de Implantación

**Metodología**: Waterfall/Híbrido con solapamientos controlados

**Principio**: Las fases tienen dependencias secuenciales, pero algunas tareas pueden ejecutarse en paralelo cuando hay equipos multidisciplinares (consultores + desarrolladores).

---

## Fases del Proyecto

### Fase 0: Inicio y Configuraciones Generales

**Duración típica**: 1-2 semanas
**Posición**: Siempre al inicio del proyecto

**Tareas incluidas:**
- Alta de servidor y bases de datos (producción + staging)
- Configuración de copias de seguridad
- Configuración de servidores de correo (entrada/salida)
- Alta de usuarios y permisos iniciales
- Configuración de idiomas
- Alta de compañías (si multicompañía)
- Reglas entre compañías (si aplica)

**Equipo**: Técnico + Consultor líder

**Output**: Entorno Odoo operativo y accesible para el equipo y cliente

---

### Fase 1: Configuración de Módulos (CONFIG + STUDIO/AUTO)

**Duración típica**: 40-60% del tiempo total del proyecto
**Posición**: Después de Configuraciones Generales

**Tareas incluidas:**
- Configuración de maestros (productos, tarifas, clientes, proveedores)
- Configuración de flujos de negocio (ventas, compras, inventario)
- Campos personalizados (Odoo Studio)
- Acciones automatizadas básicas
- Configuración de permisos y vistas

**Orden de módulos (recomendado):**
1. **Primero**: Maestros y configuraciones base (Ventas, Compras)
2. **Después**: Módulos operativos (Inventario, Fabricación)
3. **Al final**: Módulos financieros (Facturación, Contabilidad)

**Justificación del orden:**
- Los maestros (clientes, productos) son fundamentales y se usan en todos los módulos
- Los flujos operativos deben configurarse antes que los financieros
- Contabilidad requiere que los flujos anteriores estén definidos

**Equipo**: Consultor funcional + Cliente (validación constante)

**Output**: Módulos configurados y validados por el cliente

---

### Fase 2: Desarrollos a Medida (DEV) - En Paralelo

**Duración típica**: Variable según complejidad (puede abarcar 50-70% del proyecto)
**Posición**: Puede solaparse con Fase 1

**Tareas incluidas:**
- Desarrollo de módulos personalizados
- Herencias de modelos estándar
- Informes QWeb complejos
- Integraciones con APIs externas
- Lógica de negocio específica

**Principio de paralelización:**
- Los desarrollos pueden ejecutarse **en paralelo** con las configuraciones porque:
  - Equipo técnico independiente del equipo de consultoría
  - No bloquean las pruebas de funcionalidad estándar
  - Permiten aprovechar mejor los recursos

**Dependencias críticas:**
- Los desarrollos que modifican flujos estándar deben coordinarse con la configuración
- Los desarrollos que dependen de maestros deben esperar a que estos estén definidos

**Equipo**: Desarrolladores + Consultor técnico (especificaciones)

**Output**: Módulos personalizados desarrollados y en staging para pruebas

---

### Fase 3: Migraciones de Datos (MIGR)

**Duración típica**: 10-15% del tiempo total
**Posición**: Después de la configuración del módulo correspondiente

**Tareas incluidas:**
- Migración de maestros (clientes, productos, proveedores)
- Migración de datos históricos (facturas, pedidos)
- Migración de saldos contables iniciales
- Carga de stock inicial

**Orden de migración (obligatorio):**
1. **Maestros primero**: Clientes, Productos, Proveedores
2. **Dependencias**: Tarifas, Categorías, Impuestos
3. **Transacciones**: Pedidos, Facturas, Movimientos de stock
4. **Saldos**: Contabilidad (último, requiere todo lo anterior)

**Principio de secuencialidad:**
- NO puedes migrar pedidos si no existen clientes y productos
- NO puedes migrar saldos contables si no existen partners

**Equipo**: Consultor funcional + Cliente (preparación de datos)

**Output**: Datos migrados y validados en Odoo

---

### Fase 4: Pruebas y Validación (TEST)

**Duración típica**: 10-15% del tiempo total
**Posición**: Después de completar Configuración + Desarrollos + Migración de cada módulo

**Tareas incluidas:**
- Pruebas de flujos completos end-to-end
- Validación de cálculos (tarifas, impuestos, comisiones)
- Pruebas de integraciones
- Validación de informes y PDFs
- UAT (User Acceptance Testing)

**Tipos de testing:**
- **Testing funcional**: El consultor valida que la configuración cumple requisitos
- **UAT**: El cliente final valida que el sistema resuelve sus necesidades reales
- **Testing técnico**: El equipo técnico valida desarrollos e integraciones

**Equipo**: Consultor + Cliente (usuarios clave) + Desarrolladores (si hay DEV)

**Output**: Módulos validados y listos para producción

---

### Fase 5: Formaciones (TRAIN)

**Duración típica**: 5-10% del tiempo total
**Posición**: Puede solaparse con Testing y Go-Live

**Tareas incluidas:**
- Formación a usuarios finales por rol
- Formación a administradores del sistema
- Formación específica de módulos complejos
- Sesiones de Q&A

**Principio**: La formación es **continua durante el proyecto**:
- Formaciones tempranas para usuarios clave (alpha testers)
- Formaciones masivas cerca del go-live
- Formación de administradores al final (requieren visión completa)

**Equipo**: Consultor funcional

**Output**: Usuarios capacitados para usar Odoo

---

### Fase 6: Salida a Producción (Go-Live)

**Duración típica**: 1 día (evento puntual)
**Posición**: 1 mes antes de la finalización del proyecto

**Tareas incluidas:**
- Migración final de datos (datos frescos desde sistema anterior)
- Sincronización de maestros actualizados
- Validación final de configuraciones
- Arranque oficial del sistema en producción
- Soporte intensivo el día del go-live

**Preparación crítica:**
- Todos los módulos deben estar validados antes del go-live
- Los usuarios deben estar formados
- Plan de rollback preparado (por si falla el go-live)

**Equipo**: Todo el equipo (consultor + técnico + soporte)

**Output**: Sistema en producción operando

---

### Fase 7: Soporte Post Go-Live

**Duración típica**: 1 mes después del go-live
**Posición**: Último mes del proyecto

**Tareas incluidas:**
- Resolución de incidencias urgentes
- Ajustes menores de configuración
- Formación adicional sobre casos no previstos
- Optimización de flujos basada en uso real
- Cierre de dudas operativas

**Intensidad**: Alta al inicio (primeros días), decreciente después

**Equipo**: Consultor funcional + Desarrollador (si hay incidencias técnicas)

**Output**: Sistema estabilizado y equipo del cliente autónomo

---

## Reglas de Solapamiento

### ✅ Solapamientos Permitidos

| Fase A | Fase B | Condición |
|--------|--------|-----------|
| Configuración de Módulos | Desarrollos | Siempre (equipos distintos) |
| Configuración de Módulos | Migraciones | Si el módulo ya está configurado |
| Pruebas | Formaciones | Usuarios clave pueden formarse mientras se prueba |
| Todo | Reuniones/Formaciones | Las reuniones semanales son transversales |

### ❌ Solapamientos NO Permitidos

| Fase A | Fase B | Razón |
|--------|--------|-------|
| Configuración | Pruebas del mismo módulo | No puedes probar lo que no está configurado |
| Migraciones | Pruebas del mismo módulo | No puedes probar sin datos |
| Desarrollos | Pruebas del mismo desarrollo | No puedes probar código no terminado |
| Cualquiera | Go-Live | El go-live es un evento único y puntual |

---

## Dependencias Críticas

### Diagrama de Dependencias

```
[Config Generales] ──┐
                     ├──> [Config Módulos] ──┐
                     │                       ├──> [Migraciones] ──> [Pruebas] ──> [Go-Live] ──> [Soporte]
[Desarrollos] ───────┘                       ┘

[Formaciones] ───────────────────────────────────────────> (transversal)
[Reuniones] ─────────────────────────────────────────────> (transversal)
```

### Reglas de Secuencia

1. **Config Generales** debe completarse antes de cualquier otra tarea
2. **Maestros** (clientes, productos) deben existir antes de configurar flujos que los usen
3. **Configuración de módulo** debe completarse antes de migrar datos de ese módulo
4. **Migraciones** deben completarse antes de probar flujos end-to-end
5. **Pruebas** deben completarse antes del go-live
6. **Go-Live** debe ocurrir 1 mes antes del fin de proyecto (para tener tiempo de soporte)

---

## Hitos del Proyecto

Define hitos claros para tracking de progreso:

| Hito | Descripción | % Proyecto |
|------|-------------|-----------|
| **M1: Entorno Listo** | Configuraciones generales completadas | 10% |
| **M2: Maestros Configurados** | Clientes, productos, tarifas en Odoo | 25% |
| **M3: Flujos Operativos OK** | Ventas, compras, inventario funcionando | 50% |
| **M4: Desarrollos Completados** | Todos los módulos custom en staging | 65% |
| **M5: Migraciones OK** | Todos los datos históricos en Odoo | 75% |
| **M6: UAT Aprobado** | Cliente valida que todo funciona | 85% |
| **M7: Go-Live** | Sistema en producción | 90% |
| **M8: Cierre de Proyecto** | Soporte finalizado, proyecto cerrado | 100% |

---

## Notas Finales

- **Flexibilidad controlada**: Aunque es Waterfall, se permiten iteraciones dentro de cada fase
- **Comunicación constante**: Las reuniones semanales son críticas para mantener alineación
- **Validación incremental**: No esperes al final para validar - valida módulo por módulo
- **Plan de contingencia**: Reserva tiempo para imprevistos (10-15% de la bolsa total)
