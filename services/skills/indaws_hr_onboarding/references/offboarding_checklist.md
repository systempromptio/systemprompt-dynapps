# Checklist de Offboarding

**CONFIDENTIAL - HR TEAM ONLY**

Proceso estandar de desvinculacion de empleados en Indaws. Cubre devolucion de equipos,
revocacion de accesos, transferencia de conocimiento y entrevista de salida.

---

## Timeline de Offboarding

| Fase | Cuando | Responsable |
|------|--------|-------------|
| Comunicacion interna | Dia de notificacion | RRHH + Manager |
| Planificacion de transicion | Semana 1 tras notificacion | Manager |
| Transferencia de conocimiento | Periodo de preaviso | Empleado + Manager |
| Devolucion de equipos | Ultimo dia | IT + Empleado |
| Revocacion de accesos | Ultimo dia (fin jornada) | IT |
| Entrevista de salida | Ultima semana | RRHH |
| Baja administrativa | Ultimo dia + 1 | RRHH |

---

## 1. Devolucion de Equipos (Responsable: IT)

### Inventario de Equipos a Devolver

| Item | Verificar | Notas |
|------|-----------|-------|
| Portatil corporativo | Numero de serie, estado fisico | Borrado seguro de datos |
| Monitor adicional | Si fue asignado | Incluir cables |
| Teclado / raton corporativo | Si fue asignado | - |
| Auriculares / webcam | Si fueron asignados | - |
| Telefono movil corporativo | Si fue asignado | SIM devuelta |
| Llave / tarjeta de acceso oficina | Siempre | Desactivar en sistema |
| Tarjeta de parking | Si fue asignada | - |
| Material de papeleria/oficina | Items significativos | A criterio de IT |

### Procedimiento para Empleados Remotos
- Coordinar envio por mensajeria prepagada
- Proporcionar etiquetas de envio
- Confirmar recepcion de todos los items
- Plazo maximo: 5 dias habiles tras el ultimo dia

---

## 2. Revocacion de Accesos (Responsable: IT)

> **CRITICO**: Todos los accesos deben revocarse el ultimo dia efectivo, al final de la jornada.
> NO revocar antes del ultimo dia salvo instruccion expresa de RRHH.

### Accesos a Revocar

| Sistema | Accion | Prioridad |
|---------|--------|-----------|
| Google Workspace (@indaws.es) | Suspender cuenta, transferir datos | CRITICA |
| Slack | Desactivar usuario | CRITICA |
| HiBob | Marcar como inactivo | CRITICA |
| Odoo (todos los modulos) | Desactivar usuario | CRITICA |
| VPN corporativa | Revocar certificado | CRITICA |
| GitHub / GitLab | Remover de organizacion | ALTA |
| Gestor de contrasenas (1Password) | Revocar acceso, rotar secretos compartidos | CRITICA |
| CI/CD (Jenkins / GitHub Actions) | Revocar tokens | ALTA |
| Herramienta de tickets (Helpdesk) | Desactivar usuario | ALTA |
| Banca online | Revocar acceso | CRITICA |
| Herramienta de gestion proyectos | Desactivar usuario | MEDIA |
| Sentry / Monitoring | Remover usuario | MEDIA |
| Notion / dynbrain | Revocar acceso | MEDIA |
| Google Drive compartido | Revocar acceso a carpetas sensibles | ALTA |

### Post-Revocacion
- Verificar que no quedan sesiones activas
- Rotar credenciales compartidas que el empleado conocia
- Actualizar listas de distribucion de email
- Remover de grupos de Slack departamentales

---

## 3. Transferencia de Conocimiento (Responsable: Manager + Empleado)

### Documentacion a Generar

| Documento | Descripcion | Destinatario |
|-----------|-------------|--------------|
| Estado de proyectos activos | Lista de proyectos, estado, proximos hitos | Manager / Sucesor |
| Contactos clave de clientes | Personas de contacto, contexto de relacion | Manager / Sucesor |
| Procedimientos no documentados | Tareas recurrentes, procesos especificos del rol | Equipo |
| Credenciales de servicio | Cuentas de servicio gestionadas (NO personales) | IT |
| Backlog / tareas pendientes | Lista de tareas abiertas, prioridad | Manager |
| Documentacion tecnica pendiente | Codigo, configuraciones, arquitectura | Equipo tecnico |

### Sesiones de Transferencia
- Programar minimo **3 sesiones** de transferencia durante el periodo de preaviso
- Cada sesion de **1-2 horas** con el sucesor o el equipo
- Grabar las sesiones (con consentimiento) para referencia futura
- Documentar las preguntas que surjan y sus respuestas

---

## 4. Entrevista de Salida (Responsable: RRHH)

> **IMPORTANTE**: La entrevista de salida es VOLUNTARIA. Respetar la decision del empleado si declina.

### Logistica
- Programar en la **ultima semana** del empleado
- Duracion: 30-45 minutos
- Formato: presencial preferido, videollamada si remoto
- Conducida por RRHH (nunca por el manager directo)
- Confidencial: resultados agregados, nunca atribuidos individualmente

### Areas a Cubrir
1. **Motivo de salida**: Razon principal, factores contribuyentes
2. **Experiencia general**: Satisfaccion con el trabajo, equipo, empresa
3. **Relacion con manager**: Comunicacion, apoyo, desarrollo profesional
4. **Cultura y ambiente**: Valores, inclusion, bienestar
5. **Compensacion y beneficios**: Percepcion de equidad (sin preguntar cifras)
6. **Desarrollo profesional**: Oportunidades de crecimiento, formacion
7. **Sugerencias de mejora**: Recomendaciones concretas
8. **Disposicion a recomendar**: NPS del empleado saliente

---

## 5. Documentacion Administrativa (Responsable: RRHH)

| Tarea | Plazo | Notas |
|-------|-------|-------|
| Comunicacion formal de baja | Segun tipo | Carta de baja, finiquito |
| Calculo de finiquito | Ultimo dia | Vacaciones pendientes, pagas extras |
| Baja en Seguridad Social | Ultimo dia + 1 | Via asesoria laboral |
| Certificado de empresa | Ultimo dia + 3 | Para el empleado |
| Carta de recomendacion | Si solicitada | Opcional, a criterio de la empresa |
| Actualizacion de organigrama | Ultimo dia + 1 | HiBob + comunicacion interna |
| Reasignacion de funciones | Antes del ultimo dia | Temporal o permanente |

---

## 6. Comunicacion Interna (Responsable: RRHH + Manager)

| Comunicacion | Cuando | A quien | Contenido |
|-------------|--------|---------|-----------|
| Notificacion a IT | Dia de notificacion | Equipo IT | Fecha ultimo dia, accesos a revocar |
| Notificacion al equipo | Segun manager | Equipo directo | Comunicacion personal del manager |
| Anuncio general | Antes del ultimo dia | Empresa | Si es apropiado, mensaje de despedida |
| Notificacion a clientes | Si aplica | Clientes afectados | Nuevo punto de contacto |

---

## Adaptaciones por Motivo de Salida

### Baja Voluntaria
- Periodo de preaviso: 15 dias (convenio)
- Entrevista de salida: ofrecer siempre
- Tono: agradecimiento por la contribucion

### Fin de Contrato Temporal
- Preaviso: segun duracion del contrato
- Entrevista de salida: ofrecer
- Posibilidad de renovacion: comunicar decision con antelacion

### Despido
- Seguir procedimiento legal estricto
- Revocacion de accesos inmediata tras comunicacion
- No programar entrevista de salida
- Asesor laboral gestiona documentacion

### Jubilacion
- Celebracion / reconocimiento (si el empleado lo desea)
- Transferencia de conocimiento extendida
- Entrevista de salida: ofrecer con enfoque en legado
