Seguridad y accesos cliente-a · rama staging-visor (tarea NNNNN)

Cliente A · Tarea NNNNN · Análisis de la rama STAGING-VISOR

# Field360 sí existe aquí: qué cambia eso en la implantación de las matrices de permisos

Auditoría en solo lectura del entorno `field.cliente-a.example`, contraste con las tres matrices y con el análisis de la rama master, y plan de implantación en siete fases.

Rama staging-visor
Base staging-visor · Odoo 18.0+e
Método XML-RPC solo lectura
Fecha 10-ago-2026

Solo lectura · no se ha escrito nada en el entorno

Documento hermano del análisis de la rama master. No son intercambiables: los dos entornos difieren en 186 módulos, 4 usuarios internos, 20 usuarios portal y en si Field360 está instalado o no. Un test verde en uno no dice nada del otro.

## Dónde estamos

17de 20 usuarios internos comparten el mismo bloque de administrador

18de 20 no tienen ningún grupo de Field360, que es el módulo central de este entorno

12módulos de la matriz sin instalar: casi un tercio no es implementable

4hallazgos de seguridad, dos compartidos con master

Las conclusiones estructurales del análisis de master se confirman: los usuarios internos son administradores de casi todo, la matriz se contradice consigo misma en 29 celdas y las empresas siguen sin pirámide.

Pero tres cosas cambian el trabajo. Field360 existe aquí —v18.0.1.36.0, 25 modelos, portal de subcontratistas funcionando—, lo que hace verificable la columna SUBCONTRATA que en master no lo era. Este entorno es más permisivo que master en finanzas y personas. Y faltan doce módulos de la matriz, cuatro más que allí y no los mismos.

## Los dos entornos han divergido

Esta tabla es la razón por la que existen dos análisis y no uno. No es la misma base con distinto nombre.

Auditados ambos el 10-ago-2026: master desde el dump restaurado en local, staging-visor en vivo por XML-RPC.
| 
| |   | master | staging-visor

| | Módulos instalados | 505 | 319

| | Usuarios internos activos | 16 | 20

| | Usuarios portal activos | 22 (trabajadores reales) | 2 (subcontrata de prueba)

| | Proyectos | 55 | 8, casi todos demo

| | Grupos · Reglas · ACL | 147 · 642 · 1.924 | 113 · 461 · 1.414

| | `clientea_field` | no instalado | sí v18.0.1.36.0, 25 modelos

| | Documentos, Nómina, Flota, Servicio de campo, eLearning | sí instalados | no instalados

| | Con `base.group_system` | 5 usuarios de negocio | 2 cuentas técnicas

### Dónde es más permisivo cada uno

| 
| | Grupo | master | staging-visor

| | Contabilidad · Administrador | 4 | 17

| | Empleados · Administrador | 5 | 17

| | Selección · Administrador | 5 | 17

| | Gastos · Administrador | 4 | 17

| | Partes de horas · Administrador | 4 | 17

| | Field360 · Usuario o Responsable | módulo ausente | 2, ambas técnicas

Staging-visor es sensiblemente más permisivo en el bloque financiero y de personas; master lo es en el control del sistema. Ninguno de los dos se parece a la matriz.

### La población de usuarios tampoco coincide

`{usuario-1}@` no existe aquí, aunque en master sí y la matriz le asigna rol. `{usuario-2}@` e `{usuario-3}@` están activos aquí e inactivos allí. Y aparecen tres cuentas técnicas —`{tecnica-1}@`, `{tecnica-2}@`, `{tecnica-3}@`— que no son personas de la matriz y no reciben rol.

## Hallazgos

H1
Documentación laboral y médica borrable por cualquier usuario interno
Crítico · RGPD

`x_prl_y_documentacion` concede a Usuario interno lectura, escritura, creación y borrado. Ahí viven el APTO médico, el DNI y los contratos de cada empleado. Idéntico a master: es configuración heredada, no del entorno.

H2
Dos reglas de seguridad nativas desactivadas
Crítico

`hr_timesheet.timesheet_line_rule_portal_user` y `project.project_task_rule_portal_project_sharing` están con `active = false`. También idéntico a master. Corregir H1 y H2 sirve para los dos entornos a la vez.

H5
La regla de incidencias de Field360 no filtra nada
Alto · bloquea la matriz

`clientea_field.rule_field_issue_internal` tiene dominio `[(1, '=', 1)]`, atada a `base.group_user` con permisos de lectura, escritura, creación y borrado. Conviene precisar qué es y qué no: no es una fuga hacia el portal —los usuarios portal no están en `base.group_user` y su propia regla los filtra bien—, pero anula cualquier ámbito por obra para los usuarios internos. Es el bloqueante para implantar la columna Field360 de la matriz.

H3
Los 8 proyectos abiertos al portal
Bajo aquí

Mismo principio que en master, pero el volumen es trivial y casi todo es demo. Cuidado al corregirlo: `{Proyecto DEMO} — DEMO Field360` es el proyecto sobre el que se está demostrando el visor.

### Lo que está bien, y es más que en master

- Las reglas multicompañía nativas están intactas y activas.

- Field360 trae 15 reglas globales multicompañía correctamente escritas sobre sus propios modelos.

- La regla de portal de subcontrata hace exactamente lo que pide la matriz: `[('assigned_company_id.commercial_partner_id', '=', user.commercial_partner_id.id)]`, y solo concede lectura.

- Ningún usuario portal tiene `base.group_multi_company`. El hallazgo H4 de master no se reproduce aquí.

## Qué cambia en la implantación

La arquitectura de seguridad es la misma que en el análisis de master —un rol es un grupo, bloques funcionales, grupos de consulta propios porque las reglas de grupo se suman en lugar de restarse, ámbito de obra por `member_ids`— y no se repite aquí. Lo que cambia es esto.

### Field360 entra en alcance, respetando lo que ya existe

No se reescribe su seguridad: el módulo es de otro equipo y las tareas 19851 y 19852 lo están tocando ahora mismo. Se hacen tres cosas.

- Los roles pasan a implicar sus grupos. Dirección de Proyectos toma `group_field360_manager`; Jefatura de Obra, Producción, Calidad, PMO y BIM toman `group_field360_user` con ámbito ampliado; Encargado, Técnicos y HubVisual con ámbito propio; Dirección General, Comunicación y Logística reciben un grupo de consulta nuevo.

- Se sustituye la regla de incidencias, desactivándola en lugar de borrarla.

- Se retira el permiso de borrado donde la matriz no lo concede: hoy `group_field360_user` puede borrar en ocho modelos.

# Hoy — grupo base.group_user, permisos R1 W1 C1 U1
[(1, '=', 1)]

# Propuesto — ámbito de obra, roles Jefatura / Producción / Calidad / Encargado
['|', '|',
('assigned_user_id', '=', user.id),
('project_id.user_id', '=', user.id),
('project_id.member_ids', 'in', [user.id])]

# Ámbito de empresa y de grupo — Técnicos, HubVisual, BIM, D. Proyectos, PMO, D. General:
# sin regla adicional; basta la global rule_field_issue_company, que ya existe y es correcta
```

El riesgo real de este cambio es el visor. `indaws_api` y los modelos `field.viewer.api` y `field.viewer.link` consultan `field.issue`. Restringir por obra puede dejar el visor sin datos si lee con un usuario de servicio. El plan incluye un test que ejerce ese camino antes de dar la fase por buena.

Y si se rompe, la salida no es devolver el dominio `[(1, '=', 1)]`: es darle al visor su propia regla o su propio ámbito.

### Casi un tercio de la matriz no es implementable aquí

Doce módulos no están instalados: Documentos, Nómina, Flota, Servicio de campo, eLearning, Marketing por correo, Encuestas, Eventos, Servicio de asistencia, Fabricación, Taller y Limpieza de datos. Sus columnas se marcan como desviación y aparecen como fallo esperado en el informe de pruebas. No desaparecen: si un día se instalan, el test las reclama solo.

### A cambio, aquí sí se puede verificar el portal de subcontrata

| 
| | Celda de la matriz | master | staging-visor

| | Columna Field360, doce roles | módulo ausente | implementable

| | Columna SUBCONTRATA EXTERNA | módulo ausente | implementable y verificable

| | Ámbito por obra sobre incidencias | no aplica | se implementa

| | Columna CLIENTE | módulo ausente | sigue sin existir

Los modelos de anclaje del portal de cliente sí existen ya —`field.meeting` para las actas de reunión de obra, `field.monthly.report` para el informe mensual a la propiedad—, pero el flujo aceptar → RGPD → firmar → descargar sigue siendo desarrollo nuevo.

## Cómo se ejecuta

Las mismas siete fases que en master, cada una con su puerta mecánica, más dos condiciones propias de esta rama.

Auditoría reproducibleverify.sh auditLa auditoría remota convertida en script de solo lectura por ORM, no por SQL: aquí no hay acceso a psql.

Módulo y rolesTestRolesLos 17 grupos de rol y los 4 bloques, ahora con la integración de los tres grupos de Field360.

ACL y grupos de consultaTestAclDiez grupos de solo lectura —tres menos que en master, porque faltan módulos— más los 25 modelos de Field360.

Reglas de registroTestRulesÁmbito por grupo, empresa y obra, más la sustitución de la regla de incidencias y la comprobación del visor.

Vistas y remediaciónTestViewsMárgenes, PRL y tesorería ocultos a quien no debe verlos, y los hallazgos corregidos desde el módulo.

Asignación de rolesTestAssignmentUn rol por persona. Las tres cuentas técnicas quedan excluidas explícitamente, no fallando.

QA de matriz y runbookTestMatrixTest celda a celda, incluida la columna SUBCONTRATA, con un usuario portal real que comprueba que ve solo las incidencias de su empresa.

Nada de esto escribe en `staging-visor`. El acceso que se nos dio es de administrador, pero el encargo era leer: toda la auditoría se hizo con `search_read`, `search_count` y `authenticate`, sin un solo `create`, `write` ni `unlink`. El módulo se construye sobre una copia local y el entorno remoto recibe un runbook.

Y no se usa como banco de pruebas: es un entorno compartido con las tareas 19851 y 19852.

## Lo que necesitamos que decida cliente-a

Las cinco del análisis de master siguen en pie. Esta rama añade dos, y la primera es la que más dinero mueve de las siete.

- ¿Cuál de los dos entornos va a ser producción? Si es staging-visor, el análisis de master estudia un entorno que va a desaparecer. Si no lo es, todo el trabajo de Field360 no llega a producción por esta vía.

- ¿Quién es el dueño de la seguridad de `clientea_field`? Las tareas 19851 y 19852 lo están tocando. Proponemos no reescribirla y limitarnos a ajustar lo que la matriz contradice, pero hay que acordarlo antes de la fase 4.

- Las 29 discrepancias entre los dos libros de matrices, al menos las de módulos sensibles.

- Qué rol tienen {Usuario A} y {Usuario B}, que hoy son administradores y no aparecen en ninguna matriz.

- Si el Encargado debe existir como usuario interno, y qué pasa con `{usuario-1}@`, que la matriz nombra y aquí no existe.

Plan completo, decisiones y anexos: `plans/draft/2026-08-permisos-cliente-a-staging-visor/`
Análisis hermano de la rama master: `plans/draft/2026-08-permisos-cliente-a-master/`
Indaws · Dynapps Spain para Cliente A · 10-ago-2026
