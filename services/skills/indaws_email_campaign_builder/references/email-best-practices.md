# Mejores Prácticas de Email Marketing — Indaws

## Subject Lines

### Reglas
- **Máximo 50 caracteres** (idealmente 30-40 para móvil)
- **Personalización:** Usar `{{name}}` o `{{company}}` cuando sea relevante
- **Sin spam words:** Evitar GRATIS, OFERTA, URGENTE, DESCUENTO, ¡¡¡, $$$
- **Sin MAYÚSCULAS completas:** Solo para una palabra de énfasis máximo
- **Sin emojis excesivos:** Máximo 1 emoji, y solo si encaja con el tono
- **Preguntas:** Funcionan bien para engagement ("¿Tu equipo pierde tiempo en...?")
- **Números:** Concretos y creíbles ("3 señales de que...", "en 2 semanas")

### Patrones Efectivos
```
Personalización: "{{name}}, esto es para ti"
Curiosidad: "Lo que nadie te cuenta sobre {{tema}}"
Dolor/Reto: "¿Tu {{proceso}} necesita evolucionar?"
Social proof: "Cómo {{empresa}} consiguió {{resultado}}"
Directo: "¿Hablamos esta semana, {{name}}?"
Recurso: "Tu guía gratuita: {{tema}}"
```

### Patrones a Evitar
```
❌ "¡¡¡OFERTA INCREÍBLE!!!"
❌ "ABRE ESTO AHORA"
❌ "No te lo vas a creer"
❌ "Última oportunidad (por tercera vez)"
❌ Líneas de asunto genéricas sin valor
```

## Preheader

- **Máximo 100 caracteres** (lo visible en la mayoría de clientes)
- **Complementa** el subject, no lo repite
- **Nunca dejarlo vacío** — el cliente mostrará el inicio del body
- Ejemplo: Subject "¿Tu ERP necesita evolucionar?" → Preheader "5 señales que indican que es momento de dar el siguiente paso"

## Estructura del Email

### Longitud
- **Emails de campaña:** 150-300 palabras (lecturable en 1-2 minutos)
- **Newsletter:** 400-600 palabras (varias secciones)
- **Evento:** 100-200 palabras (claridad sobre la acción)

### Anatomía
1. **Preheader** (hidden text, 100 chars)
2. **Header** (logo Indaws)
3. **Saludo personalizado** ("Hola {{name}},")
4. **Apertura enganchante** (1-2 frases que conectan con el reto/interés)
5. **Body** (valor principal — educativo, caso, oferta)
6. **CTA principal** (botón prominente)
7. **Cierre** (firma personal o del equipo)
8. **Footer** (legal + unsubscribe)

### CTAs
- **Un solo CTA principal por email** (puede haber secundarios en texto)
- **Botón visible:** Mínimo 44x44px clickable area (móvil)
- **Texto del botón:** Verbos de acción, 2-5 palabras
  - ✅ "Reservar mi consulta"
  - ✅ "Ver caso de éxito"
  - ✅ "Descargar la guía"
  - ❌ "Click aquí"
  - ❌ "Más información"
  - ❌ "Enviar"
- **Color:** Blue Lilac (#6B68FA) o Warm Yellow (#E5B92B) según contexto

## Timing de Envío

### Mejores Horarios (España, B2B)
- **Martes a Jueves:** Mayor engagement
- **9:00-10:00:** Inicio de jornada, revisan inbox
- **14:00-15:00:** Post-comida, revisan pendientes
- **Evitar:** Lunes temprano (inbox saturado), Viernes tarde (ya están "de fin de semana")

### Spacing entre Emails
- **Mínimo:** 2 días entre emails de la misma secuencia
- **Óptimo:** 3-5 días (permite procesar sin olvidar)
- **Máximo:** 7 días (más allá se pierde el hilo)
- **Newsletter:** Frecuencia fija (semanal, quincenal o mensual)

## Deliverability

### Evitar Filtros de Spam
- Ratio texto/imagen: mínimo 60% texto
- No usar imágenes como único contenido
- Incluir siempre versión texto plano (Odoo lo genera automáticamente)
- Enlace de unsubscribe visible y funcional
- Dirección física en el footer (requerimiento legal)
- "From" name reconocible (Indaws, no "noreply")
- Reply-to funcional (que alguien lea las respuestas)

### Técnicas
- Autenticar dominio (SPF, DKIM, DMARC) — configuración de infraestructura
- Warm up listas nuevas: empezar con contactos más engaged
- Limpiar lista: eliminar bounces y contactos sin apertura en 6+ meses

## Personalización

### Merge Fields Disponibles en Odoo
```
${object.name}           — Nombre completo
${object.email}          — Email
${object.company_name}   — Nombre empresa
${object.phone}          — Teléfono
${object.title.name}     — Título (Sr., Sra.)
${user.name}             — Nombre del comercial asignado
```

### Reglas de Personalización
- **Nombre en subject:** Aumenta open rate ~20%
- **Nombre en saludo:** Siempre ("Hola {{name}},")
- **Empresa en body:** Cuando sea relevante para el contexto
- **Fallback:** Siempre tener un default ("Hola," si no hay nombre)

## Mobile

- **60%+ de emails se leen en móvil**
- Ancho máximo: 600px
- Font size body: mínimo 14px (idealmente 16px)
- Font size subject (móvil): visible ~35 caracteres
- Botones: mínimo 44px alto, full-width en móvil
- Imágenes: responsive, max-width 100%
- Texto: sin columnas múltiples en móvil

## Métricas de Referencia (B2B España)

| Métrica | Bueno | Excelente |
|---------|-------|-----------|
| Open Rate | >20% | >35% |
| Click Rate | >2.5% | >5% |
| Bounce Rate | <2% | <0.5% |
| Unsubscribe | <0.5% | <0.1% |
| Reply Rate | >1% | >3% |

## Vocabulario Indaws para Emails

Recordatorio del vocabulario de valor (ver `dynapps-brand/references/verbal-identity.md`):

| Evitar | Usar |
|--------|------|
| gasto, coste, precio | **inversión, valoración** |
| problema | **reto** |
| soporte | **acompañamiento técnico** |
| mantenimiento | **evolución continua** |
| presupuesto | **propuesta de valor** |
| técnico | **especialista** |
| cliente | **colaborador** |

Tutear siempre. Tono "Sabio Cercano": experto pero accesible.
