Seguridad y accesos cliente-a · rama master (tarea NNNNN)

Cliente A · Tarea NNNNN · Análisis de la rama MASTER

# Seguridad y accesos: qué piden las matrices, qué hay en Odoo y cómo cerramos la distancia

Auditoría de la instancia real, contraste contra las tres matrices entregadas por el cliente y plan de implantación en siete fases verificables.

Rama master
Base auditada master-auto-20260809 · copia local
Versión Odoo 18.0 EE
Fecha 10-ago-2026

Propuesta para revisión · no se ha modificado ninguna base del cliente

Este documento analiza solo la rama master, donde `clientea_field` no está instalado. Existe un análisis hermano de la rama staging-visor, con Field360 instalado y un reparto de permisos distinto. Los dos entornos no son intercambiables.

## Dónde estamos

16usuarios internos, y todos son administradores de casi todas las apps

29celdas en las que los dos libros de matrices se contradicen

4hallazgos de seguridad activos hoy en la instancia

33/41módulos de la matriz realmente instalados

Las tres matrices describen un sistema maduro: 17 roles internos, tres portales, ámbito por empresa y por obra, segregación de funciones. La instancia real está en el extremo opuesto. Entre lo que se pide y lo que hay no falta un ajuste: falta una capa de seguridad completa.

Es viable y se puede hacer de forma verificable. Pero hay cuatro cosas que decidir antes de tocar nada, y están al final de este documento.

## Lo declarado frente a lo real

Contraste directo entre la portada de las matrices y la base de datos auditada.
| 
| | Punto | La matriz dice | La instancia es

| | Versión | Odoo 19 | Odoo 18.0 (`base 18.0.1.3`)

| | Empresas | Cabecera + 360 Arquitectura / Construcción / Instalaciones / HubVisual, en pirámide | 4 planas y con otros nombres: 360 Estudio, Estudio Idea Espacio, Hub Visual 360, Sostenibilidad 360. `parent_id` vacío en todas

| | Módulos | 41 instalados | 33. Faltan Eventos, Servicio de asistencia, Fabricación, Taller, Calidad, Mantenimiento, Limpieza de datos y Field360

| | Trabajador | Usuario interno con autoservicio | 22 usuarios portal

| | Aprobaciones | 3 tramos con segundo nivel | `one_step`, 5.000 € en las cinco empresas. `approvals` sin instalar

| | Encargado ({Encargado}) | Rol interno | No existe como usuario interno; el único {Encargado} de la instancia es portal

### Lo que sí está bien

Conviene decirlo: las reglas multicompañía nativas están intactas y activas en los quince modelos críticos. El aislamiento entre las cuatro empresas funciona. El problema no es multiempresa — es que dentro de cada empresa nadie está limitado.

### El permiso que hoy comparten los dieciséis

Administrador de Proyecto, de Compras, de Inventario, de Documentos, de Servicio de campo, de Firma electrónica y de Ausencias. Gestionar todos los empleados. Todos los partes de horas. Editor y diseñador del sitio web. Y costes y márgenes avanzados de venta.

En la práctica: el jefe de obra puede editar la web, la persona de facturación puede borrar proyectos y el técnico de proyecto ve el margen de todas las ofertas.

## Cuatro hallazgos que piden corrección

Estos no dependen del resto del proyecto. H1 y H2 se pueden corregir aunque todo lo demás se aplace.

H1
Documentación laboral y médica borrable por cualquier usuario interno
Crítico · RGPD

El modelo Studio `x_prl_y_documentacion` concede a Usuario interno lectura, escritura, creación y borrado. Ahí viven el APTO médico, el DNI, los contratos y el consentimiento de imagen de cada empleado: datos de categoría especial.

H2
Dos reglas de seguridad nativas desactivadas
Crítico

`hr_timesheet.timesheet_line_rule_portal_user` y `project.project_task_rule_portal_project_sharing` están con `active = false`. Son exactamente las que limitan a los usuarios portal en partes de horas y en tareas compartidas. Con 22 trabajadores en portal, la exposición es real.

H3
Los 55 proyectos abiertos al portal
Medio

Todos tienen `privacy_visibility = 'portal'`, lo que contradice el principio número uno del propio libro de portales: denegación por defecto. Cualquier usuario portal seguidor de un proyecto ve sus tareas.

H4
Un usuario portal con conmutador de empresas
Medio

La cuenta `{cuenta-prueba}` tiene `base.group_multi_company`, un grupo que no debería tener ningún usuario externo.

## Las matrices se contradicen entre sí

Los tres libros se leen sin desalineación: 41 módulos × 20 roles y 17 apps × 17 roles. Pero al cruzar los dos que se solapan aparecen 29 discrepancias sobre 238 celdas comparables, el 12 %. Varias caen sobre Contabilidad, Empleados y Asistencias.

Muestra de las discrepancias en módulos sensibles. La lista completa va en `discrepancias.md`, con una columna en blanco para la decisión del cliente.
| 
| | Rol | Módulo | Hoja 3 dice | Matriz de módulos dice

| | D. Operaciones | Contabilidad | C consulta | R responsable

| | D. Operaciones | Empleados | C consulta | O+ operador ampliado

| | D. Operaciones | Asistencias | C consulta | O+ operador ampliado

| | Dirección Económica | Empleados | O+ | R

| | Dirección General | Ajustes | C | R

| | Administración | Contabilidad | O | O+

### Cómo se resuelven mientras tanto

- El nivel por módulo lo manda la matriz de módulos, que es la más granular.

- El ámbito G/E/P lo manda la hoja 3, que es la única que lo declara.

- Tesorería, Contratación y Contabilidad analítica se toman de la hoja 3: no tienen módulo equivalente.

- En empate sobre un módulo sensible gana el menor privilegio, y la celda queda marcada como pendiente.

El punto 4 no cierra el asunto. Sobre nómina y tesorería, equivocarse por exceso no es simétrico a equivocarse por defecto, y no nos corresponde decidirlo.

## La arquitectura que proponemos

Un usuario recibe un grupo de rol, y ese grupo implica todo lo demás. Nadie recibe grupos sueltos, porque así es exactamente como se llega al estado actual: cada incidencia se resolvió añadiendo un grupo y nadie los quitó nunca.

cliente-a · Roles (17)
lo único que se asigna a una persona

implied_ids

Bloques funcionales (4)
lo transversal, sin repetir

Grupos nativos de Odoo
niveles O · O+ · R · A

Grupos de consulta (13)
propios · nivel C

ir.model.access
qué operaciones

ir.rule
sobre qué registros

La jerarquía no es una pirámide lineal: la matriz da a Dirección General nivel de consulta en Empleados y Nómina mientras RRHH tiene responsabilidad. Se encadena solo donde la matriz lo respalda — Encargado ⊂ Producción ⊂ Jefatura de Obra — y lo transversal se agrupa en bloques.

### Cómo se traduce cada nivel de la matriz

| 
| | Nivel | Qué significa | Cómo se implementa en Odoo

| | — | Sin acceso | Ninguna ACL. El menú no aparece

| | C | Consulta | Grupo propio de solo lectura que no implica el grupo nativo de usuario, con ACL `1,0,0,0` y acceso explícito al menú

| | O | Operador, sus documentos | Grupo nativo de usuario + `ir.rule` que filtra por `user_id`

| | O+ | Operador ampliado | Grupo nativo de usuario, sin regla adicional: la multicompañía nativa ya lo limita a su empresa

| | R | Responsable | Grupo nativo de administrador de la app

| | A | Administrador técnico | Administrador + `base.group_system`, solo el rol IT

Por qué el nivel «consulta» es la parte cara. En Odoo las reglas de registro de grupo se unen, no se restan: no se puede convertir a solo-lectura a alguien que ya tiene el grupo de usuario de la app añadiéndole una regla restrictiva. Y salvo Contabilidad, Odoo 18 no trae grupos de solo lectura. Hay que fabricarlos, uno por módulo.

Se fabrican para los 13 módulos sensibles o de uso diario. Para los de sensibilidad baja — Fabricación, Taller, Código de barras, Eventos, Encuestas, Sitio web — se acepta el grupo nativo como aproximación, y queda anotado como desviación consciente, no escondido.

### Ámbito de obra: el campo que hoy no existe

La matriz habla de «los proyectos u obras que tiene asignados», y en la instancia no hay ningún campo que lo exprese: `user_id` es solo el responsable. Se añade `member_ids` a `project.project` y de ahí cuelgan las reglas.

<!-- Ámbito P · Jefatura de obra, Producción, Calidad, Encargado -->
<record id="rule_project_ambito_obra" model="ir.rule">
<field name="domain_force">
['|', ('user_id', '=', user.id), ('member_ids', 'in', [user.id])]
</field>
</record>

# project.task
['|', '|',
('user_ids', 'in', [user.id]),
('project_id.user_id', '=', user.id),
('project_id.member_ids', 'in', [user.id])]

# Ámbito G · escrito ya para la pirámide del día que exista
['|', ('company_id', '=', False),
('company_id', 'parent_of', company_ids)]
```

Hoy, sin `parent_id` en ninguna empresa, `parent_of` se comporta igual que `in`. Se escribe así para no reescribir ni volver a probar las reglas cuando la Cabecera exista.

### Y la regla transversal de las ACL

id,name,model_id:id,group_id:id,perm_read,perm_write,perm_create,perm_unlink
# Nivel C — Jefatura de obra en Contabilidad: solo lectura
access_account_move_consulta,...,account.model_account_move,group_contabilidad_consulta,1,0,0,0
# Nivel O+ — Producción en Proyecto: crea y edita, no borra
access_project_task_pr,...,project.model_project_task,group_rol_pr,1,1,1,0
# Nivel R — Dirección Económica en Contabilidad: control total
access_account_move_de,...,account.model_account_move,group_rol_de,1,1,1,1
```

Borrar solo en los niveles R y A. Ningún operador elimina registros.

## Lo que no se resuelve con permisos

| 
| | Requisito de la matriz | Por qué no es un permiso | Qué haría falta

| | Umbrales de aprobación 3k / 15k / >15k | Es un circuito, no un acceso | Módulo `approvals`. Los importes están marcados «(H)»: hipótesis sin validar por Dirección

| | «Quien registra no aprueba, quien aprueba no paga» | Es segregación de funciones sobre un flujo | Reglas de aprobación más una comprobación de que nadie acumula los dos grupos. Esto sí es automatizable como test

| | Aceptar → RGPD → firmar → descargar | Es una máquina de estados | Odoo Sign más desarrollo. La propia matriz lo marca como coste a presupuestar

| | El cliente colabora en el acta de obra | Modelo y flujo nuevos | Field360 (`clientea_field`), que ni siquiera está instalado

| | Conformidades liberadas una a una | Visibilidad por registro individual | Field360 más regla por `partner_id`

Portales y Field360 son, con honestidad, otro proyecto. Entran en el diseño de este documento; no en su implantación.

## Cómo se ejecuta

Siete fases, cada una con una puerta mecánica: no avanza hasta que su comando de verificación sale con código cero. Y como `odoo-bin` devuelve cero incluso con tests en error, la verificación va envuelta en un guion que lee la línea de resumen del log. Una puerta que no puede cerrarse no es una puerta.

Auditoría reproducibleverify.sh auditLa auditoría de este documento convertida en script idempotente, y las tres matrices normalizadas a un CSV canónico que será el fixture de todos los tests.

Módulo y rolesTestRoles`clientea_security` instalable, con los 17 grupos de rol y los 4 bloques funcionales. Todavía no se asigna a nadie.

ACL y grupos de consultaTestAclLos 13 grupos de solo lectura y el `ir.model.access.csv` completo. Es la fase de mayor riesgo del plan.

Reglas de registroTestRulesÁmbito por grupo, por empresa y por obra, más el campo `member_ids`. Con usuarios impersonados que comprueban el aislamiento.

Vistas y remediaciónTestViewsMárgenes, nómina, PRL y tesorería ocultos a quien no debe verlos. Y los cuatro hallazgos H1–H4 corregidos desde el módulo, no a mano.

Asignación de rolesTestAssignmentUn rol por persona y purga de los grupos heredados, con diff revisado antes de aplicarse.

QA de matriz y runbookTestMatrixUn test que recorre las tres matrices celda a celda, más el procedimiento de despliegue con su vuelta atrás.

Nada de esto toca la instancia del cliente. Todo se construye y se verifica sobre la copia local del dump. Staging y producción reciben un runbook, no una ejecución. Y ninguna corrección se hace a mano en base de datos: o va dentro del módulo, o no se hace.

## Lo que necesitamos que decida cliente-a

No se escribe una línea de código hasta que estas cinco estén respondidas. Las cuatro primeras son bloqueantes.

- Las 29 discrepancias entre los dos libros, al menos las que caen en módulos de sensibilidad alta: Contabilidad, Empleados y Asistencias.

- Qué rol tienen {Usuario A} y {Usuario B}, que hoy son administradores y no aparecen en ninguna de las tres matrices.

- Si el Encargado debe existir como usuario interno. Hoy no existe, y la matriz le asigna rol.

- Si se acepta mantener a los 22 trabajadores en portal. Convertirlos a internos cumple la hoja 4 al completo y cuesta 22 licencias Enterprise; mantenerlos deja cinco de sus ocho funciones sin cubrir.

- Confirmación de que la pirámide de cinco empresas queda para cuando la asesoría cierre el mapa jurídico.

Plan completo, decisiones de diseño y anexos: `plans/draft/2026-08-permisos-cliente-a/`
Indaws · Dynapps Spain para Cliente A · 10-ago-2026
