# Workflow Detallado - HR Employee Lookup

**CONFIDENTIAL - HR TEAM ONLY**
**Document ID:** HR-SKILL-006-REF-002

Bloques de ejecucion y formatos de presentacion para el skill `hr-employee-lookup`.

---

## Convencion de ejecucion (cwd)

Todos los bloques Python se ejecutan **desde la raiz de la suite** (`ai-claude-hr/`,
el directorio raiz del plugin). La libreria compartida vive en `lib/`, por lo que se
inyecta con `sys.path.insert(0, 'lib')`. No usar rutas absolutas de maquina.

Si el cwd no es la raiz del plugin, ajustar la ruta relativa a `lib/` en consecuencia
(p. ej. `../../lib` si se ejecuta desde dentro del propio skill).

---

## Paso 0: Recopilacion de Informacion (DETENTE)

PREGUNTAR al usuario antes de cualquier ejecucion:

1. **Tipo de busqueda** (obligatorio): por nombre (parcial/completo), por email
   corporativo, o por departamento.
2. **Termino de busqueda** (obligatorio): el nombre, email o departamento a buscar.
3. **Nivel de detalle** (opcional, default: ficha basica):
   - Ficha basica: nombre, puesto, departamento, manager, email.
   - Ficha completa: basica + ubicacion, fecha incorporacion, telefono oficina.
   - Con reporting line: ficha + cadena jerarquica (max 3 niveles).
   - Con historial: ficha + cambios de puesto/departamento.

> **DETENTE** y espera la respuesta del usuario antes de continuar.

---

## Paso 1: Verificacion de Conexion

```bash
python3 -c "import sys; sys.path.insert(0,'lib'); from hibob_client import HiBobClient; c=HiBobClient(); ok=c.test_connection(); print('Conexion HiBob:', 'OK' if ok else 'FALLO')"
```

Si falla:
- Verificar `HIBOB_API_TOKEN` y `HIBOB_SERVICE_USER` definidos.
- Comprobar que `HIBOB_API_URL` apunta a `https://api.hibob.com/v1`.
- Informar al usuario y detener.

---

## Paso 2: Ejecucion de la Busqueda

**Por nombre:**
```bash
python3 -c "import sys, json; sys.path.insert(0,'lib'); from hibob_client import HiBobClient; c=HiBobClient(); print(json.dumps(c.search_people([{'fieldPath':'root.fullName','operator':'contains','values':['<NOMBRE>']}]), indent=2, ensure_ascii=False))"
```

**Por email:**
```bash
python3 -c "import sys, json; sys.path.insert(0,'lib'); from hibob_client import HiBobClient; c=HiBobClient(); print(json.dumps(c.get_person_by_email('<EMAIL>'), indent=2, ensure_ascii=False))"
```

**Por departamento:**
```bash
python3 -c "import sys, json; sys.path.insert(0,'lib'); from hibob_client import HiBobClient; c=HiBobClient(); print(json.dumps(c.search_people([{'fieldPath':'work.department','operator':'equals','values':['<DEPARTAMENTO>']}]), indent=2, ensure_ascii=False))"
```

**Multiples resultados:** presentar lista numerada (nombre, puesto, departamento) y
pedir seleccion. Si >20 resultados, pedir refinar criterios. Si search_people pagina,
recorrer todas las paginas antes de afirmar "no hay mas".

**Sin resultados:** sugerir variaciones del nombre u otros criterios; ofrecer listar
departamentos disponibles.

---

## Paso 2.5: Validacion (DETENTE)

Resumen de resultados:
```
Resultados encontrados: {N} empleado(s)
1. {nombre_completo} - {puesto} ({departamento})
2. ...
```

Ficha de un empleado concreto:
```
Ficha de Empleado
=================
Nombre:          {fullName}
Puesto:          {title}
Departamento:    {department}
Manager:         {manager_name}
Ubicacion:       {site / city}
Fecha de alta:   {startDate}   (DD/MM/YYYY)
Email:           {email_corporativo}
Telefono:        {phone_office}
```

> **DETENTE** si se pidio reporting line o historial: pregunta si continuar con esos
> datos adicionales.

---

## Paso 3: Datos Adicionales (si se solicitaron)

**Linea de reporte (max 3 niveles):**
```bash
python3 -c "import sys; sys.path.insert(0,'lib'); from hibob_client import HiBobClient; c=HiBobClient(); emp=c.get_person('<EMPLOYEE_ID>'); chain=[emp]; cur=emp
for _ in range(3):
    mid=(cur.get('work') or {}).get('reportsTo',{}).get('id')
    if not mid: break
    m=c.get_person(mid); chain.append(m); cur=m
for i,p in enumerate(chain):
    pre=('  '*i)+('-> ' if i>0 else ''); print(pre+(p.get('fullName') or 'No disponible')+' ('+((p.get('work') or {}).get('title') or 'No disponible')+')')"
```

Presentacion:
```
Linea de Reporte
================
{empleado} ({puesto})
  -> {manager_nivel_1} ({puesto})
    -> {manager_nivel_2} ({puesto})
      -> {manager_nivel_3} ({puesto})
```

**Historial laboral:** extraer del campo `lifecycle` del empleado los cambios de
puesto, departamento y tipo de empleo. Presentar cronologicamente:
```
Historial Laboral
=================
{fecha} - {tipo_cambio}: {descripcion}
{fecha} - Alta en la empresa ({puesto_inicial}, {departamento_inicial})
```

Si `lifecycle` o `reportsTo` vienen vacios, ver GOTCHAS en SKILL.md.

---

## Notas de presentacion

- Fechas siempre `DD/MM/YYYY`.
- Campo vacio o inexistente -> "No disponible" (nunca en blanco).
- Procesamiento en batch -> `report_utils.sanitize_employee_data()`.
- Busqueda por departamento -> solo nombre + puesto + email corporativo.
