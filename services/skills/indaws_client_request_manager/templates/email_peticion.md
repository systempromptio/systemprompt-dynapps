# Template: Email de Peticion Inicial

## Estructura del email

### Asunto
```
[{nombre_proyecto}] {titulo_peticion}
```

### Cuerpo

**IMPORTANTE**: Este template es una GUIA, no un molde fijo. Cada email debe parecer escrito por un humano, con variedad natural en la redaccion. NUNCA copies la misma estructura ni las mismas frases en emails consecutivos.

```
Saludo → Contexto (que estamos haciendo y por que necesitamos esto) → Peticion concreta → Deadline con razon practica → Cierre
```

### Ejemplo (solo como referencia, NO copiar literalmente)

```html
<p>Buenos dias,</p>

<p>Estamos configurando los usuarios en Odoo y necesitamos los siguientes datos para poder darlos de alta:</p>

<ul>
  <li>...</li>
</ul>

<p>En cuanto tengamos esta informacion procederemos con el alta, por lo que seria importante poder tenerla <strong>antes del 27 de marzo</strong> para que los usuarios puedan ir conectandose y probando los flujos.</p>

<p>Cualquier cosa nos decis.</p>

<p>Un saludo,</p>
<p>Juan De la Rubia<br/>
Equipo de Implantacion Indaws</p>
```

---

## Reglas de naturalidad (CRITICO)

### Apertura: VARIAR SIEMPRE
El primer parrafo debe ser diferente en cada email. Conecta con lo que se esta haciendo en ese momento del proyecto:

**Ejemplos validos** (usar como inspiracion, no copiar):
- "Buenos dias," / "Buenas tardes,"
- "Hola {nombre},"
- "Hola de nuevo," (si es un email reciente al mismo contacto)

**Contexto inicial** — redactar de forma natural segun la peticion concreta:
- "Estamos configurando {area} en Odoo y necesitamos..."
- "De cara a la puesta en marcha de {modulo}, nos faltaria..."
- "Para poder avanzar con {tarea_concreta}, necesitariamos que nos facilitarais..."
- "Siguiendo con la implantacion, estamos en el punto de {fase} y nos haria falta..."
- "Os escribo porque estamos preparando {lo_que_sea} y necesitamos un dato vuestro:"

**PROHIBIDO**: Usar siempre la misma frase de apertura como "Dentro del proyecto de implantacion de X, necesitamos vuestra colaboracion en el siguiente punto". Esta frase es generica, repetitiva y suena a plantilla automatica.

### Deadline: CONTEXTUALIZAR
No usar frases genericas como "Seria ideal recibir esta informacion antes del X". En su lugar, explicar POR QUE es importante esa fecha en terminos practicos:

**Ejemplos validos**:
- "En cuanto tengamos esto lo daremos de alta, por lo que seria importante tenerlo antes del {fecha} para que podais ir probando"
- "Necesitariamos tenerlo para el {fecha} ya que esa semana empezamos con la configuracion de {area}"
- "Si nos lo podeis enviar antes del {fecha} podemos tenerlo listo para la formacion del viernes"
- "Lo ideal seria tenerlo antes del {fecha}, asi lo dejamos configurado antes de pasar a {siguiente_fase}"
- "Nos vendria bien tenerlo para el {fecha} — si necesitais mas tiempo avisadnos y ajustamos"

### Cierre: VARIAR
No terminar siempre con la misma frase. Alternar entre:

- "Cualquier cosa nos decis."
- "Si teneis alguna duda, aqui estamos."
- "Quedamos a vuestra disposicion para lo que necesiteis."
- "Si necesitais que os echemos una mano con esto, decidnos."
- "No dudeis en escribirnos si necesitais aclarar algo."
- "Si teneis cualquier pregunta, respondednos a este email."

### Firma
```html
<p>{nombre_consultor}<br/>
Equipo de Implantacion Indaws</p>
```

O variantes:
- "Un saludo," / "Gracias," / "Muchas gracias,"

---

## Reglas de personalizacion por tipo

### information
Tono conversacional, con preguntas concretas. Explicar para que se usara la informacion.

### data_file
Especificar formato deseado (Excel, CSV). Ofrecer ayuda para el export:
```
Si necesitais ayuda para sacarlo de vuestro sistema actual, podemos hacer una llamada rapida y os guiamos.
```

### validation
Hacer referencia a lo que debe validar y por que:
```
Hemos configurado {lo_configurado} segun lo que comentamos en el analisis. Necesitamos que le echeis un vistazo y nos confirmeis que esta correcto antes de seguir adelante.
```

### decision
Presentar las opciones claras y ofrecer reunion si es complejo:
```
Necesitamos que decidais sobre este punto. Las opciones serian:
- Opcion A: ...
- Opcion B: ...
Si quereis que os lo expliquemos con mas detalle, organizamos una llamada rapida.
```

### access
Indicar exactamente que datos tecnicos se necesitan. Ofrecer alternativa de sesion compartida:
```
Necesitariamos las credenciales de {servicio}. Si preferis no compartirlas por email, podemos hacer una sesion en la que lo configuremos juntos.
```

---

## Reglas estrictas

- **NUNCA** mencionar horas presupuestadas, costes o informacion interna del proyecto
- **NUNCA** mencionar herramientas internas (odoo-pilot, skills, checklists internos)
- **NUNCA** usar tono de urgencia excesivo o presion
- **NUNCA** repetir la misma estructura o frases entre emails consecutivos
- **NUNCA** usar frases que suenen a plantilla automatica o a bot
- **SIEMPRE** dar contexto practico de por que se necesita la informacion
- **SIEMPRE** contextualizar el deadline con una razon concreta
- **SIEMPRE** que el email parezca escrito por una persona real, con variedad natural
- **SIEMPRE** adaptar el nivel de formalidad al momento del proyecto (mas formal al inicio, mas cercano cuando ya hay confianza)
