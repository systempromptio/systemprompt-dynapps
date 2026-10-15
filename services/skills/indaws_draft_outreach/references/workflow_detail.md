# Draft Outreach - Workflow Detail

## Paso 0: Recopilacion de Informacion

PREGUNTAR al usuario:
1. **A quien va dirigido** (nombre, empresa, email, LinkedIn)
2. **Tipo de outreach**: cold, warm, re-engagement, post-evento, post-demo
3. **Contexto** (por que contactar ahora, referencia, evento, etc.)
4. **Objetivo** (demo, llamada, reunion, respuesta)

> **DETENTE** y espera respuesta del usuario.

## Paso 1: Investigacion (OBLIGATORIO)

### 1.1 Buscar en Odoo (via odoo-pilot)

```
Modelo: crm.lead
Buscar por: email, partner_name, contact_name
Extraer: historial de oportunidades, ultima interaccion, etapa, notas
```

```
Modelo: res.partner
Buscar por: email, name
Extraer: historial de pedidos, notas, interacciones
```

### 1.2 Buscar en Clay (si disponible)

- Buscar contacto por nombre/email/empresa
- Extraer: cargo, empresa, tamano, sector, tech stack, senales recientes
- Identificar puntos de conexion personalizados

### 1.3 Sintetizar hallazgos

```
FICHA DE PROSPECTO:
Nombre:      {nombre}
Cargo:       {cargo}
Empresa:     {empresa} ({tamano}, {sector})
Historial:   {resumen_interacciones_previas_o_"primera_vez"}
Tech stack:  {tecnologias_actuales}
Senal:       {por_que_ahora - funding, contratacion, cambio, etc.}
Punto hook:  {dato_personal_o_empresarial_para_personalizar}
```

## Paso 2: Redactar con Framework AIDA

Para cada tipo de outreach, seguir el framework AIDA adaptado:

**A - ATENCION** (linea de asunto / primera frase):
- Personalizada con dato real del prospecto
- Maximo 6 palabras en asunto
- Sin clickbait, sin mayusculas, sin exclamaciones

**I - INTERES** (primer parrafo):
- Conectar con su situacion actual
- Mencionar dato especifico (tech stack, crecimiento, sector)
- Mostrar que has investigado

**D - DESEO** (segundo parrafo):
- Beneficio concreto para su empresa
- Caso similar o referencia relevante
- Sin hablar de producto, hablar de resultado

**A - ACCION** (cierre):
- Peticion clara y facil (15 min, no "reunion")
- Dar opciones de horario
- Sin presion

## Paso 3: Generar Outputs

PRESENTAR al usuario:

```
=============================================
  OUTREACH - {tipo}
  Para: {nombre} ({empresa})
=============================================

--- INVESTIGACION ---
{ficha_de_prospecto}

--- EMAIL ---
Asunto: {asunto_6_palabras_max}

{nombre},

{parrafo_atencion_interes - 2 frases max}

{parrafo_deseo - 2 frases max, beneficio concreto}

{parrafo_accion - peticion clara}

{firma}

--- LINKEDIN (alternativa) ---
{Mensaje mas corto, 3 frases max, tono mas cercano}

--- SECUENCIA FOLLOW-UP (3 toques) ---

Touch 2 (dia +3):
Asunto: Re: {asunto_original}
{2 frases recordatorio + dato adicional}

Touch 3 (dia +7):
Asunto: {asunto_alternativo}
{Angulo diferente - caso de exito o insight del sector}

Touch 4 - Break-up (dia +14):
Asunto: {ultimo_intento}
{Mensaje de cierre respetuoso, dejar puerta abierta}
=============================================
```

## Plantillas por Tipo

### Cold (primera vez, sin relacion previa)
- Hook: dato especifico del prospecto o su empresa
- Beneficio: resultado medible de un caso similar
- CTA: "15 minutos para explorar si tiene sentido"

### Warm (referencia, evento, conexion)
- Hook: mencionar la conexion o referencia
- Beneficio: por que la referencia/conexion es relevante
- CTA: "Me gustaria contarte como ayudamos a {referencia}"

### Re-engagement (lead dormido, oportunidad perdida)
- Hook: "Ha pasado tiempo desde nuestra ultima conversacion"
- Beneficio: novedad relevante (nueva funcionalidad, caso de exito, cambio en su sector)
- CTA: "Ha cambiado algo en {su_area}?"

### Post-evento (webinar, conferencia, feria)
- Hook: referencia al evento especifico
- Beneficio: profundizar en tema del evento
- CTA: "Me gustaria ampliar lo que comentamos sobre {tema}"

### Post-demo (seguimiento tras demostracion)
- Hook: referencia a momento concreto de la demo
- Beneficio: resumen de como resuelve su problema especifico
- CTA: siguiente paso concreto (propuesta, POC, referencia)

## Formato del Email

- **Texto plano** (sin HTML, sin markdown, sin negritas, sin listas)
- **Espanol profesional** (no coloquial, no formal en exceso)
- **Maximo 100 palabras** (cold) / 150 palabras (warm/follow-up)
- **Sin emojis**
- **Sin adjuntos** en primer contacto
- **Sin "Estimado/a"** ni "Quedo a su disposicion" ni "Espero que se encuentre bien"
- Parrafos cortos (2 frases max por parrafo)
- Un solo CTA por email

## Success Criteria

- [ ] Investigacion completada (Odoo + Clay si disponible)
- [ ] Ficha de prospecto con datos reales
- [ ] Email personalizado con dato especifico del prospecto
- [ ] Formato texto plano, maximo palabras respetado
- [ ] Framework AIDA aplicado correctamente
- [ ] Variante LinkedIn generada
- [ ] Secuencia follow-up con 3 toques + break-up
- [ ] Cada touch usa angulo diferente
