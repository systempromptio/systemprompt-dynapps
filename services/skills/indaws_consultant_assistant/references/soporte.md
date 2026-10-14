# Soporte

## Qué es

La gestión de incidencias y consultas del cliente a lo largo de todo el ciclo de vida del proyecto. Existen dos modalidades diferenciadas según el momento del proyecto.

## Modalidades

### Soporte de proyecto (antes del go-live)

Aplica durante la fase de ejecución (fase 4.6), cuando el cliente empieza a usar el sistema en entorno de pruebas o piloto. Las incidencias se gestionan en el contexto del proyecto activo y suelen estar relacionadas con la configuración, adaptaciones o datos.

- El consultor gestiona las incidencias directamente desde las tareas del proyecto
- Las resoluciones se documentan en el chatter de la tarea correspondiente
- No requiere un canal de helpdesk formal, pero sí seguimiento en Odoo

### Soporte de soporte (post-producción)

Aplica tras el go-live (fase 5), cuando el cliente está operando en producción. En esta fase el canal formal es el helpdesk y el skill principal es `service-desk`.

- **`service-desk` es el skill clave** para gestionar tickets de soporte post-producción
- Requiere conexión API a la base de datos del cliente para poder consultar datos, reproducir incidencias y proponer soluciones contextualizadas
- Los tickets se gestionan desde `helpdesk.ticket` en Odoo

## Qué debe saber el consultor

### Scope del service desk

El skill maneja:
- **Incidencias**: Errores, bugs, comportamientos inesperados
- **Consultas funcionales**: "¿Cómo hago X en Odoo?"
- **Configuración**: Ajustes necesarios post go-live
- **Datos**: Correcciones o importaciones adicionales

### Workflow del ticket

```
1. Cliente envía ticket (helpdesk.ticket)
2. service-desk analiza: investiga código + instancia del cliente via API
3. Diagnóstico: ¿Es incidencia/bug o petición fuera de alcance?
4. Si incidencia → Propone solución en nota interna
5. Si fuera de alcance → Consultor propone alternativa al cliente
```

### Regla fundamental

**NUNCA enviar mensajes directos al cliente desde el skill**. El service desk genera notas internas para el consultor, que decide cómo comunicar al cliente.

### Tipos de resolución

| Tipo | Acción | Quién |
|------|--------|-------|
| Config simple | Ajustar settings/datos | Consultor via odoo-configurator |
| Bug de código | Fix por equipo técnico | Escalación a desarrollo |
| Error de usuario | Explicación funcional | Consultor (nota interna → comunicación) |
| Fuera de alcance | Nueva propuesta comercial | Consultor + PM |
| Datos incorrectos | Corrección de datos | importation-data-assistant o manual |

### Onboarding de soporte

Al inicio de la fase de soporte post-producción:
1. Configurar el cliente en Odoo Helpdesk
2. Definir SLAs y canales de comunicación
3. Explicar al cliente cómo enviar tickets
4. Establecer expectativas de tiempo de respuesta

## Qué debe hacer el consultor

1. Revisar tickets nuevos al inicio del día (incluido en consultant-daily-planner)
2. Ejecutar `service-desk` para cada ticket pendiente
3. Revisar la nota interna generada antes de actuar
4. Comunicar al cliente según el tipo de resolución
5. Cerrar el ticket cuando esté resuelto

## Skills relacionados

- `service-desk`: Skill principal de gestión de tickets (requiere conexión API a la instancia del cliente)
- `odoo-configurator`: Para correcciones de configuración
- `service-desk-analytics` *(futuro)*: Dashboard de análisis de cartera de soporte

## Errores comunes

1. **Enviar mensajes directos al cliente sin revisar**: El skill genera notas internas, no respuestas al cliente
2. **No distinguir incidencia de petición nueva**: Una petición fuera de alcance requiere nueva propuesta, no un fix
3. **No documentar la escalación**: Si se escala a desarrollo, documentar el problema completamente
4. **Ignorar tickets >24h**: El consultant-daily-planner marca como URGENTE los tickets sin responder en 24h
5. **Resolver sin verificar**: Después de corregir, verificar que el problema realmente se resolvió
6. **Iniciar soporte post-producción sin conexión API**: El service-desk necesita acceso a la base de datos del cliente para funcionar correctamente
