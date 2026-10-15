# Cuestionario: Recursos Humanos

## Seccion 1: Gestion de Empleados (hr)

### Datos Maestros del Empleado

1. **Como gestionan actualmente la informacion de los empleados? (Software especifico de RRHH, ERP, hojas de calculo, carpetas fisicas)**

2. **Ademas de los datos estandar de Odoo (nombre, contacto, departamento, puesto), que otra informacion especifica necesitan registrar para cada empleado? (Ej: Tallas de uniforme, personas de contacto en caso de emergencia, n de hijos, certificaciones con fecha de caducidad, datos bancarios para nomina, tipo de permiso de trabajo y su validez, etc.)**

3. **Necesitan gestionar una estructura organizativa jerarquica clara (organigrama)? Como se define esta jerarquia (quien reporta a quien)?**

4. **Como se gestionan los diferentes puestos de trabajo y departamentos en la empresa? Hay descripciones de puestos asociadas?**

5. **Necesitan almacenar y gestionar documentos adjuntos al perfil del empleado? (Ej: DNI/NIE, contrato firmado, titulaciones, certificados medicos, evaluaciones de desempeno). Con que politica de acceso?**

### Migracion de Datos (Empleados)

6. **Necesitan migrar la base de datos de empleados actual a Odoo? Aproximadamente cuantos empleados (activos e historicos si aplica)?**

7. **En que formato se encuentran los datos a migrar (Excel, CSV, otro sistema)?**

8. **Que campos de informacion son imprescindibles migrar?**

## Seccion 2: Gestion de Asistencias (hr_attendance)

### Registro de Asistencia

9. **Como registran actualmente la asistencia los empleados? (Fichaje manual en papel, hoja de calculo, sistema de fichaje con tarjeta/huella, app movil)**

10. **Que metodos de fichaje necesitan en Odoo? (Ej: Manual por el empleado desde su portal, modo Kiosko en un dispositivo comun, integracion con lectores de tarjetas/biometricos, fichaje por geolocalizacion desde app movil, restricciones por IP)**

11. **Se debe registrar la entrada y salida para descansos (ej. comida)?**

12. **Hay diferentes horarios de trabajo o turnos? Como se asignan a los empleados? Necesitan gestionar horarios flexibles?**

### Validacion y Control

13. **Quien es responsable de revisar y validar los registros de asistencia?**

14. **Cual es el proceso para corregir errores u olvidos en los fichajes? Requiere aprobacion?**

15. **Como se gestionan las horas extra? Requieren autorizacion previa o posterior? Como se calculan (factor de compensacion, tipo de hora extra)?**

16. **Necesitan alertas por fichajes incompletos, retrasos o ausencias no justificadas?**

### Informes y Analisis (Asistencias)

17. **Que informes de asistencia son necesarios? (Ej: Horas trabajadas por empleado/periodo, informe de puntualidad, horas extra realizadas, resumen de absentismo laboral no planificado)**

## Seccion 3: Gestion de Gastos de Empleados (hr_expense)

### Tipos y Registro de Gastos

18. **Que tipos de gastos pueden reportar los empleados? (Ej: Kilometraje, dietas, transporte publico, alojamiento, material de oficina, comidas con clientes, peajes, parking)**

19. **Como reportan los gastos actualmente los empleados? (Formulario en papel, email con adjuntos, Excel, otra herramienta)**

20. **Necesitan que los empleados puedan crear gastos desde una app movil, adjuntando fotos de los recibos?**

21. **Existen politicas de gastos definidas? (Ej: Limites por tipo de gasto, necesidad de adjuntar justificante a partir de cierto importe, gastos no permitidos)**

### Flujo de Aprobacion

22. **Describan el proceso actual de aprobacion de gastos. Quien revisa y aprueba los gastos (manager directo, departamento de RRHH, departamento financiero)?**

23. **El flujo de aprobacion varia segun el importe o tipo de gasto?**

24. **Una vez aprobados, como se procede al reembolso al empleado? (Incluido en nomina, transferencia bancaria independiente)**

### Informes y Analisis (Gastos)

25. **Que informes de gastos necesitan? (Ej: Gastos por empleado, gastos por departamento, gastos por tipo, gastos pendientes de aprobacion/reembolso, comparacion con presupuestos)**

## Seccion 4: Gestion de Ausencias y Vacaciones (hr_holidays)

### Tipos de Ausencias

26. **Que tipos de ausencias gestionan? (Ej: Vacaciones anuales, dias de asuntos propios, baja por enfermedad comun, accidente laboral, maternidad/paternidad, permiso de matrimonio, mudanza, horas medicas, ausencias no remuneradas)**

27. **Algunos de estos tipos de ausencia requieren adjuntar documentacion justificativa?**

### Solicitud y Aprobacion

28. **Como solicitan actualmente los empleados sus ausencias?**

29. **Describan el flujo de aprobacion de las solicitudes de ausencia. Quien aprueba (manager directo, RRHH)? Hay multiples niveles de aprobacion?**

### Informes y Analisis (Ausencias)

30. **Que informes de ausencias son necesarios? (Ej: Saldos de vacaciones por empleado, dias disfrutados y pendientes, planificacion de ausencias del equipo/departamento, informe de absentismo planificado)**

## Seccion 5: Gestion de Flota de Vehiculos (fleet)

### Datos de los Vehiculos

31. **Gestionan una flota de vehiculos de empresa? Cuantos vehiculos aproximadamente?**

32. **Que informacion necesitan registrar para cada vehiculo? (Ej: Matricula, marca, modelo, numero de bastidor (VIN), tipo de combustible, fecha de compra/matriculacion, fecha de proxima ITV, poliza de seguro y su vencimiento, contrato de leasing/renting y sus condiciones)**

33. **Como se asignan los vehiculos a los empleados (conductor habitual, uso compartido)?**

### Migracion de Datos (Flota)

34. **Necesitan migrar la base de datos de vehiculos actual? Que informacion es clave?**

## Seccion General de RRHH (Roles, Permisos, Automatizaciones)

### Roles y Permisos

35. **Definan los diferentes roles que interactuaran con los modulos de RRHH y que nivel de acceso/edicion deberian tener para cada seccion (Ej: Empleado (autoservicio), Manager de Departamento, Especialista de RRHH, Administrador de RRHH, Responsable de Flota):**

### Automatizaciones

36. **Identifican tareas repetitivas en sus procesos de RRHH actuales que les gustaria automatizar con Odoo?**

### Formacion

37. **Cuantos usuarios aproximadamente utilizaran las diferentes funcionalidades de RRHH?**
