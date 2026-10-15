# Kick-Off Presentation - Rules and Validations

## Rules

### Contenido
- PERSONALIZAR TODO: Cada slide debe contener datos reales del proyecto, nunca placeholders genericos
- NO mostrar horas ni importes al cliente en la presentacion. Los datos economicos son internos
- NO usar modulos OCA ni software de terceros. Solo Odoo estandar e Indaws Core
- Tono: Profesional pero cercano. Transmitir liderazgo sin arrogancia. Usar "tu"
- Compromisos: Ser firme pero constructivo. No amenazar, sino mostrar transparencia
- Vision del proyecto: Siempre mostrar "antes vs despues" personalizado con datos de la toma de datos
- Fechas: NUNCA usar expresiones temporales relativas. Siempre fecha concreta (DD de Mes de YYYY)
- Alcance: Incluir siempre "El alcance se confirma al finalizar el analisis"

### Visual (Presentacion HTML)
- Colores: Exclusivamente paleta Indaws (Blue Space, Blue Lilac, Warm Yellow, Light Sky, blanco)
- Tipografia: Montserrat (400/600/700) con Verdana fallback
- Slides oscuras: Title, Section Dividers, Closing (fondo Blue Space o gradiente)
- Slides claras: Content, Data (fondo blanco o Light Sky suave)
- Sin emojis en la presentacion
- Sin animaciones excesivas. Solo fade/appear sutil

### Knowledge Article
- Estilos inline obligatorios (Odoo editor no soporta CSS externo)
- Estructura de documento, no de presentacion. Es para leer, no para proyectar
- Incluir toda la informacion del kick-off como referencia consultable
- Tablas HTML con estilos para compromisos, calendario, alcance

### Guion
- Tiempos realistas: Total 60-90 minutos
- Preguntas anticipadas: Minimo 2-3 por bloque tematico
- Respuestas sugeridas: Concretas, no evasivas. Si no sabemos algo, decir "lo validaremos en el analisis"
- Transiciones naturales: Frases puente entre slides, no saltos bruscos

## Validaciones

### Interactivas (obligatorias)
1. Paso 0: Usuario proporciona todos los datos obligatorios
2. Paso 2: Usuario aprueba la estructura de 20 slides

### Automaticas (antes de generar)
3. Todos los campos del sale.order leidos correctamente
4. Todos los empleados del equipo encontrados en hr.employee
5. Datos del cliente (res.partner) completos
6. Las 20 slides tienen contenido personalizado (sin placeholders)
7. El timeline es coherente (kick-off < inicio analisis < go-live)
8. La presentacion HTML es valida (tags balanceados, CSS inline correcto)
9. El knowledge article tiene estilos inline compatibles con Odoo
