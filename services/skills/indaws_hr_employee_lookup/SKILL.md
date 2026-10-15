---
name: indaws-hr-employee-lookup
description: >
  Consulta rapida de empleados en HiBob. Busqueda por nombre, email o departamento.
  Muestra ficha de empleado, linea de reporte y historial laboral. GDPR-compliant.
  Usar cuando el usuario pida: buscar empleado, ficha empleado, quien es,
  reporting line, datos de contacto, historial laboral, directorio de personas.
version: 1.1.1
model: sonnet
effort: low
---

# HR Employee Lookup

**CONFIDENTIAL - HR TEAM ONLY**
**Document ID:** HR-SKILL-006

Consulta de SOLO LECTURA de empleados en HiBob. Busca por nombre, email o departamento
y devuelve ficha profesional, linea de reporte (max 3 niveles) e historial laboral.
Cumple RGPD en todo momento. Conecta via la libreria compartida `lib/hibob_client.py`;
no necesita scripts propios.

**Herramientas**: [Bash, Read, Grep] (solo lectura). No modifica datos en HiBob.

---

## Prerequisitos

- **HiBob API**: `HIBOB_API_URL`, `HIBOB_API_TOKEN`, `HIBOB_SERVICE_USER` en entorno.
- **Python 3** + libreria compartida `lib/hibob_client.py` (en la raiz de la suite).
- **RGPD**: leer `references/data_privacy.md` antes de mostrar cualquier dato.

---

## Workflow (resumen)

Bloques de ejecucion y formatos completos en `references/workflow.md`. Los bloques
Python se invocan desde la raiz de la suite con `sys.path.insert(0,'lib')` (sin rutas
absolutas de maquina).

0. **Recopilar (DETENTE)**: tipo de busqueda (nombre/email/departamento), termino, y
   nivel de detalle (basica / completa / + reporting line / + historial). Esperar respuesta.
1. **Verificar conexion**: `HiBobClient().test_connection()`. Si falla, informar y parar.
2. **Buscar**: `search_people()` por nombre/departamento, `get_person_by_email()` por email.
   Si >20 resultados -> pedir refinar. Multiples -> lista numerada y pedir seleccion.
3. **Validar (DETENTE)**: presentar resumen/ficha. Si se pidio reporting line o historial,
   preguntar antes de continuar.
4. **Datos adicionales**: linea de reporte (recorrer `work.reportsTo`, max 3 niveles) e
   historial laboral (campo `lifecycle`). Formatos en `references/workflow.md`.

---

## Agent Routing

| Phase | Model | Runs as |
|-------|-------|---------|
| Lookup / ficha (read-only) | sonnet | Inline (turno principal) |
| Busqueda batch o multi-empleado read-only | sonnet | Subagent (Explore) opcional |

Skill read-only de bajo esfuerzo: por defecto inline en sonnet; delegar a subagente
solo para barridos amplios sin escritura.

---

## GOTCHAS

- **`lifecycle` / `reportsTo` vacios**: HiBob deja estos campos sin rellenar para muchos
  empleados. Si `reportsTo.id` falta, la cadena de managers se corta: mostrar lo obtenido
  y advertir "linea de reporte incompleta (dato no disponible en HiBob)". No inventar niveles.
- **Busqueda por nombre devuelve >20**: nombres comunes saturan resultados. Pedir refinar
  (nombre completo, departamento) en vez de volcar la lista entera.
- **Email personal vs corporativo**: `email`/`work.phone` (corporativo) PERMITIDOS;
  email y telefono personales PROHIBIDOS. No confundirlos al presentar la ficha.
- **Paginacion de `search_people`**: la API pagina; recorrer todas las paginas antes de
  afirmar "no hay mas resultados" o un conteo total.
- **Datos RGPD prohibidos**: si un JSON de la API trae NIF, salario, IBAN, salud, etc.,
  NUNCA mostrarlos aunque vengan en la respuesta. Ver `references/data_privacy.md`.

---

## NEVER

- NUNCA modificar datos en HiBob (skill de SOLO LECTURA).
- NUNCA mostrar datos sensibles RGPD: NIF/NIE, salario, IBAN, salud, estado civil,
  religion, orientacion sexual, afiliacion sindical, email/telefono personal, foto.
- NUNCA mostrar linea de reporte de mas de 3 niveles.
- NUNCA volcar listados masivos con datos personales: por departamento, solo nombre +
  puesto + email corporativo.

## ALWAYS

- SIEMPRE leer `references/data_privacy.md` antes de presentar datos.
- SIEMPRE fechas en formato `DD/MM/YYYY`.
- SIEMPRE "No disponible" para campos vacios/inexistentes (nunca en blanco).
- SIEMPRE `report_utils.sanitize_employee_data()` para procesamiento en batch.
- Ante solicitud de dato protegido, responder: "Estos datos estan protegidos por RGPD
  y no pueden mostrarse a traves de esta herramienta" (texto completo en data_privacy.md).

---

## Delegates to

| Skill | Cuando |
|-------|--------|
| None | Skill autonomo; no delega en otros skills. |

## Consumed by

| Skill | Uso |
|-------|-----|
| hr-onboarding | Resolver ficha/manager del empleado a incorporar o desvincular. |
| hr-people-report | Resolver nombres/reporting line al detallar headcount. |

(Reutilizan el patron de lookup; no hay invocacion directa programatica.)

---

## REFERENCES

- `references/workflow.md` — bloques de ejecucion y formatos de presentacion.
- `references/data_privacy.md` — clasificacion de campos RGPD, roles y protocolo.
- `../../lib/hibob_client.py` — cliente REST de HiBob (search_people, get_person, etc.).
- `../../security/gdpr_guidelines.md` — guia RGPD de la suite.
- `common:references/shared/` — N/A: esta suite usa HiBob via `lib/`, no la conexion Odoo compartida.

---

## Success Criteria

- [ ] Conexion a HiBob verificada.
- [ ] Busqueda ejecutada segun criterio (nombre/email/departamento).
- [ ] Ficha presentada solo con campos permitidos; max 3 niveles en reporting line.
- [ ] Historial cronologico (si se pidio); ningun dato sensible expuesto.
- [ ] Cumplimiento RGPD verificado contra `references/data_privacy.md`.
