# Workflow detallado: Onboarding & Offboarding

**CONFIDENTIAL - HR TEAM ONLY**

Flujo operativo completo. SKILL.md resume; aqui esta el detalle de cada paso,
los comandos de scripts y las plantillas de validacion.

---

## Paso 0: Recopilacion de Informacion (DETENTE)

PREGUNTAR al usuario:

1. **Tipo de operacion** (obligatorio):
   - `onboarding` - Nueva incorporacion
   - `offboarding` - Desvinculacion / salida
   - `status` - Estado de onboardings en curso

2. **Datos del empleado** (obligatorio):
   - Nombre completo
   - Departamento (Desarrollo, Comercial, Consultoria, Soporte, Administracion)
   - Puesto / rol
   - Ubicacion (oficina o remoto)
   - Fecha de incorporacion o fecha de baja

3. **Para onboarding** (opcional): Manager asignado, buddy/mentor, equipo IT
   necesario (portatil, monitor, perifericos), accesos especiales (VPN, servidores,
   entornos Odoo).

4. **Para offboarding** (opcional): Motivo de baja (voluntaria, fin contrato,
   despido, jubilacion), fecha ultimo dia efectivo, proyectos en curso que
   requieren transferencia, entrevista de salida programada (si/no).

> **DETENTE** y espera la respuesta antes de continuar.

---

## Paso 1: Extraccion de Datos

EJECUTAR `scripts/fetch_onboarding_data.py`:

```bash
# Onboarding - wizards y tareas del empleado
python3 scripts/fetch_onboarding_data.py --mode onboarding \
  --employee-name "<nombre>" --department "<departamento>" --start-date "<fecha>"

# Offboarding - estado actual del empleado
python3 scripts/fetch_onboarding_data.py --mode offboarding \
  --employee-name "<nombre>" --last-day "<fecha>"

# Status - todos los onboardings en curso
python3 scripts/fetch_onboarding_data.py --mode status
```

Extrae:
1. **HiBob**: onboarding wizards activos, tareas pendientes/completadas, datos del empleado.
2. **Odoo**: registro hr.employee, departamento, manager, historial.
3. **Estado**: porcentaje de completitud del onboarding wizard.

**Verificar**:
- Que el empleado existe en al menos uno de los sistemas.
- Que el departamento es valido y tiene checklist asociado.
- Si es offboarding, que el empleado esta activo.

---

## Paso 2: Calculo (OBLIGATORIO - nunca manual)

EJECUTAR `scripts/onboarding_tracker.py`:

```bash
# Checklist de onboarding personalizado
python3 scripts/onboarding_tracker.py --mode generate-checklist \
  --input onboarding_data.json --department "<dep>" --role "<rol>" \
  --location "<ubicacion>" > checklist_result.json

# Checklist de offboarding
python3 scripts/onboarding_tracker.py --mode generate-offboarding \
  --input onboarding_data.json --reason "<motivo>" > offboarding_result.json

# Estado de onboardings en curso
python3 scripts/onboarding_tracker.py --mode track-status \
  --input onboarding_data.json > status_result.json
```

> NUNCA generes checklists manualmente. El script aplica las reglas de departamento,
> rol y ubicacion de `onboarding_checklist.md` y `offboarding_checklist.md`.

**Calcula**: checklist personalizado (tareas, responsables, plazos); plan
Day-1/Week-1/Month-1 por departamento; completitud por categoria (IT, Accesos,
Formacion...); dias hasta objetivo (30 por defecto); alertas de tareas vencidas.

---

## Paso 2.5: Validacion (DETENTE)

PRESENTAR resumen al usuario.

**Onboarding:**
```
Onboarding: {nombre} | {departamento} | {rol} | {ubicacion}
Fecha incorporacion: {fecha} | Manager: {manager} | Buddy: {buddy}
Checklist: {total} tareas
  IT Setup: {n} ({completadas}/{total}) | Accesos: {n} | Formacion: {n}
  Documentacion: {n} | Bienvenida: {n}
Plan Day-1: {n} | Week-1: {n} | Month-1: {n} objetivos
```

**Offboarding:**
```
Offboarding: {nombre} | Motivo: {motivo} | Ultimo dia: {fecha} | Antiguedad: {x}
Checklist: {total} tareas
  Equipos a devolver: {n} | Accesos a revocar: {n}
  Transferencia conocimiento: {n} | Documentacion: {n}
  Entrevista de salida: {programada_si_no}
```

> **DETENTE** y espera aprobacion antes de generar documentos. El usuario puede
> pedir ajustes (anadir tareas, cambiar responsables, modificar plazos).

---

## Paso 3: Generacion del Informe

> SOLO tras la aprobacion del Paso 2.5.

**Onboarding** — LEER plantillas y generar:
1. **Checklist** (`templates/onboarding_report.md`): tabla de tareas con responsable,
   plazo y estado, agrupada por categoria.
2. **Plan Day-1/Week-1/Month-1**: Day-1 (bienvenida, setup equipo, accesos basicos,
   presentacion); Week-1 (formacion inicial, reunion manager, primeras tareas);
   Month-1 (objetivos periodo prueba, feedback, evaluacion 30 dias).
3. **Email de bienvenida** (`templates/welcome_email.md`): personalizado con nombre,
   departamento, manager, buddy; info practica (horario, oficina, contactos).
4. **Informe para manager** (`templates/onboarding_report.md`): resumen del plan,
   hitos clave, responsabilidades del manager.

**Offboarding** — LEER plantillas y generar:
1. **Checklist** (`offboarding_checklist.md`): equipos a devolver, accesos a revocar
   (sistemas, correo, VPN), transferencia de conocimiento, documentacion pendiente.
2. **Entrevista de salida** (`templates/offboarding_report.md`): preguntas estandar
   adaptadas al motivo.
3. **Notificaciones email**: a IT (revocacion accesos), a manager (transferencia),
   a RRHH (resumen de baja).

**Envio de emails**: la suite envia via SMTP/Gmail (`lib/email_service.py`), NO via
Odoo. Es el comportamiento esperado.

GUARDAR los documentos generados en el directorio de trabajo.

---

## Reglas de negocio

1. Checklists SIEMPRE via `onboarding_tracker.py`, nunca manual.
2. No generar documentos sin aprobacion del usuario (Paso 2.5).
3. Nunca incluir datos sensibles (NIF/NIE, salario, banco, salud) — RGPD.
4. Nunca inventar datos: todo viene de HiBob y Odoo.
5. Checklists personalizados por departamento + rol + ubicacion, nunca generico.
6. Plazo onboarding: 30 dias (`ONBOARDING_COMPLETE_TARGET_DAYS` en `lib/config.py`).
7. Periodo de prueba Espana: 180 dias (`PROBATION_PERIOD_DAYS`).
8. Offboarding: revocacion de accesos el ultimo dia efectivo, no antes.
9. Entrevista de salida voluntaria: no forzar si el empleado declina.
10. Email de bienvenida: minimo 3 dias habiles antes de la incorporacion.
11. Buddy: mismo departamento, minimo 6 meses de antiguedad.
12. HiBob es la fuente primaria. Si hay discrepancia con Odoo, prevalece HiBob.
