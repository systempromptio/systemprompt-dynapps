# Guia de Redaccion de Tareas para Devs

## Principio

**Una buena tarea: el dev la completa sin volver a preguntar.** El redactor guia, no da codigo.

El dev debe saber:
- POR QUE (contexto)
- QUE (objetivo)
- DONDE (modelos/vistas)
- QUE PATRON (referencia)
- CUANDO TERMINO (criterios)
- QUE NO (restricciones)

**Test rapido**: dev senior entiende en <5 min. Dev junior puede empezar sin preguntar.

---

## Las 7 Secciones de una Tarea

### 1. Contexto de Negocio (POR QUE)

La parte mas importante y la mas ignorada. El dev que entiende el problema decide mejor.

**Patron**: "{Quien} necesita {que} para {por que}. Actualmente {situacion}. Con esto {mejora}."

| Mal | Bien |
|-----|------|
| "Anadir campo margin_percent en sale.order.line" | "El director comercial necesita ver el margen en cada linea para aprobar descuentos. Hoy lo calcula en Excel, lo cual es lento y propenso a errores." |

### 2. Objetivo (QUE, no COMO)

Describir comportamiento esperado, no implementacion.

**Patron**: "En {donde}, {que se ve/hace}. {Como se comporta}. {Cuando se actualiza}."

| Mal | Bien |
|-----|------|
| "Crear campo Float computed con depends en price_unit..." | "En cada linea del pedido, mostrar el % de margen. Se actualiza al cambiar precio o producto." |

### 3. Alcance (incluido + excluido)

Lo excluido es tan importante como lo incluido — evita scope creep.

Patron: dos listas explicitas. **Incluido**: `[...]` / **Excluido**: `[...]`

### 4. Guia de Implementacion (DONDE y QUE PATRON)

Indicar:
- Que modelos/vistas tocar
- Que patron seguir (referencia a modulo existente)
- Que documentacion consultar
- Que trampas evitar

**NUNCA dar codigo.** Adaptar detalle al nivel del dev (ver `level-scaffolding.md`).

### 5. Criterios de Aceptacion

Verificables, tipo "Cuando X, entonces Y". Nunca "funciona bien".

| Mal | Bien |
|-----|------|
| "El campo funciona correctamente" | "Cuando creo linea con coste 50€ y precio 100€, margen muestra 50%. Cuando cambio precio a 75€, recalcula a 33.3%." |

### 6. Restricciones (que NO + POR QUE)

Cada restriccion lleva justificacion. Sin POR QUE, el dev la ignora o la cumple a medias.

| Mal | Bien |
|-----|------|
| "No usar sudo()" | "No usar sudo(): el campo muestra costes, debe respetar permisos del grupo 'Ver costes'." |

### 7. Preguntas Abiertas

Si hay >3 preguntas abiertas, la tarea no esta lista. El redactor resuelve antes de asignar.

---

## Patrones por Tipo de Tarea

### Anadir campo
Contexto + tipo campo + donde mostrarlo + reglas calculo + ref campo similar

### Nuevo flujo
Contexto + diagrama estados + modelos + ref flujo estandar similar + casos edge

### Integracion API
Contexto + direccion (Odoo↔Externo) + frecuencia + docs API + mapping campos + gestion errores

### Bug fix
Pasos reproducir + comportamiento actual vs esperado + donde buscar causa + logs relevantes

### Migracion modulo
Version origen/destino + APIs deprecated + archivos por prioridad + tests que deben pasar
(ver `migration-task-requirements.md` para los 5 gates)

---

## Checklist del Redactor antes de Asignar

- [ ] La tarea tiene contexto de negocio (POR QUE)?
- [ ] El objetivo describe comportamiento, no implementacion?
- [ ] El alcance lista lo excluido explicitamente?
- [ ] La guia esta adaptada al nivel del dev asignado?
- [ ] Los criterios son verificables (Cuando X, entonces Y)?
- [ ] Las restricciones explican el POR QUE?
- [ ] Hay menos de 3 preguntas abiertas?
- [ ] La estimacion es razonable para el nivel del dev? (Junior: x1.3-1.5 del tiempo teorico)
- [ ] Hay un timer de ayuda definido?
- [ ] **Cero rutas locales?** `grep -nE '(/opt/|/home/|~/|[A-Z]:\\)'` sobre el markdown y sobre los campos que suben a Odoo. Cada recurso: **repo + rama + ruta relativa**, o **adjunto de la tarea** si no está en git. El entorno se describe por lo que hace falta (versión, addons, skill que lo levanta), no por dónde está el tuyo.
- [ ] **La tarea permite tocar código?** Ninguna tarea lo prohíbe — tampoco una revisión o auditoría: para sacar el informe hay que tocar el código, así que prohibirlo no evita el trabajo, solo impide que se entregue. El PR lleva el código; informe y evidencias son el respaldo. Si algo no debe cambiarse todavía, se nombra caso por caso con su motivo, y la fase de arreglo se deja **abierta en horas**, nunca prohibida.
- [ ] **Todo documento citado que no esté en un repo, adjuntado ya?** (specs del cliente, ficheros de ejemplo, planes de prueba). Si algo no cabe por API, ¿está dicho en la tarea a quién pedírselo?

---

## Anti-patrones generales

### Por exceso de detalle (Junior tratado como robot)

| Error | Consecuencia | Solucion |
|-------|-------------|----------|
| Dar el codigo de la solucion | Copia-pega sin entender | Apuntar al patron, no a la solucion |
| Dictar cada linea a escribir | No aprende a decidir | Dar el "que" del paso, no el "como" |
| Especificar hasta el nombre de variable | Micromanagement | Solo nombrar modelo/campo, no variables internas |

### Por falta de detalle (Junior tratado como senior)

| Error | Consecuencia | Solucion |
|-------|-------------|----------|
| "Implementa rappel" sin mas | 8 horas perdidas explorando | Dar contexto + modelo ref + pasos |
| Sin referencia a codigo existente | Reinventa la rueda | Apuntar a modulo similar |
| Sin timer de ayuda | Lleva 6h atascado sin preguntar | Establecer "pide ayuda si >Xh" |

### Por nivel incorrecto

| Error | Consecuencia | Solucion |
|-------|-------------|----------|
| Tarea de Senior con guia de Junior | Se siente micromanageado | Problema + contexto, no pasos |
| Tarea de Junior con guia de Senior | Paralizado, no sabe por donde empezar | Pasos concretos con referencias |
| Siempre tareas "safe" para Mid | Estanca, no crece | Alternar comfort + stretch tasks |
| Solo tareas triviales para Junior | Se aburre, pierde motivacion | Tareas reales con scaffolding |
