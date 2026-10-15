# Content Guidelines — Indaws Core Documenter

Guía de formato, tono y estructura HTML para cada uno de los 4 campos que genera este skill. Incluye ejemplos reales extraídos de paquetes ya documentados.

---

## 1. Descripción para Ventas (description_sale)

### Formato
- **Tipo de campo**: `text` (texto plano, sin HTML)
- **Longitud**: 2-3 líneas (100-250 caracteres)
- **Aparece en**: presupuestos y pedidos de venta

### Tono
- Profesional, directo, orientado al beneficio del cliente
- Sin jerga técnica (nada de "módulo", "modelo", "campo", "herencia")
- **NUNCA empieces con una pregunta retórica**
- Empieza directamente describiendo lo que hace el paquete o el beneficio principal

### Estructura
1. Qué hace el paquete (descripción directa del beneficio)
2. Resultado para el cliente

### Ejemplo real: [CORE] Gestión de Descuentos Múltiples
```
Gestiona múltiples descuentos en cascada (ej. 10+5+2) en tus ventas, compras y facturas. Además, potencia tus tarifas para aplicar promociones globales sobre tus precios ya negociados, dándote una flexibilidad total para gestionar tus ofertas.
```

### Ejemplo real: [CORE] Framework de API
```
La base tecnológica para construir integraciones de API personalizadas, seguras y potentes. Abre las puertas de tu Odoo para que hable con cualquier otra aplicación de forma controlada.
```

### Ejemplo real: [CORE] Utilidades de facturación
```
Mejora tu proceso de facturación para que sea más claro, profesional y eficiente. Más control sobre tus documentos contables, mejor organización de la información y comunicación automática con tus clientes.
```

---

## 2. Notas Internas (description)

### Formato
- **Tipo de campo**: `html`
- **Tags permitidos**: `<p>`, `<ul>`, `<li>`, `<ol>`, `<b>`, `<i>`
- **Sin clases CSS ni estilos inline**

### Audiencia
Comerciales y consultores de Indaws. Conocen Odoo, pueden entender nombres técnicos.

### Estructura obligatoria

```html
<p></p>
<p><b>Cliente Ideal:</b> {descripción del tipo de empresa, 1-2 frases}</p>
<p></p>
<p><b>Resumen de Módulos:</b></p>
<ul>
  <li><b>{Nombre Comercial del Módulo} ({nombre_tecnico}):</b>
    <ul>
      <li><b>Qué es:</b> {una frase}</li>
      <li><b>Funcionalidad:</b> {descripción detallada: campos que añade, comportamiento, integración con otros módulos}</li>
    </ul>
  </li>
  <!-- Repetir para cada módulo -->
</ul>
<p></p>
<p><b>Argumento de Venta:</b> {2-4 frases con el argumento comercial principal}</p>
```

### Ejemplo real: [CORE] Utilidades de facturación

```html
<p></p>
<p><b>Cliente Ideal:</b> <i>Cualquier</i> cliente que use Odoo Contabilidad (account). Especialmente útil para empresas que emiten facturas complejas (que se benefician de las secciones) o que tienen un alto volumen de facturación (donde fusionar borradores y automatizar el CC/BCC ahorra mucho tiempo).</p>
<p></p>
<p><b>Resumen de Módulos:</b></p>
<ul>
  <li><b>Secciones Inteligentes para Facturas (indaws_account):</b>
    <ul>
      <li><b>Qué es:</b> Potencia las secciones de las facturas.</li>
      <li><b>Funcionalidad:</b> 1) Añade botones "Añadir Sección/Nota" a las líneas de factura (mejora de UI). 2) Añade un asistente para aplicar cambios (impuestos, analítica) a todas las líneas debajo de una sección.</li>
    </ul>
  </li>
  <li><b>Asistente para Fusionar Facturas (indaws_account_move_merge):</b>
    <ul>
      <li><b>Qué es:</b> Una herramienta para consolidar facturas borrador.</li>
      <li><b>Funcionalidad:</b> Añade una acción de menú en la lista de facturas. Permite seleccionar múltiples borradores (con filtros de seguridad: mismo cliente, moneda, etc.) y los fusiona en uno nuevo, cancelando los originales.</li>
    </ul>
  </li>
  <!-- ... más módulos ... -->
</ul>
<p></p>
<p><b>Argumento de Venta:</b> Es el paquete de "calidad de vida" para el departamento de contabilidad. Resuelve cuatro de las quejas más comunes sobre la rigidez de la facturación en Odoo. Ahorra clics, reduce errores manuales y asegura la coherencia de los documentos.</p>
```

### Reglas para notas internas
- En "Funcionalidad" incluye detalles técnicos: nombres de modelos, campos, comportamientos
- Si un módulo depende de otro del mismo paquete, menciona la relación
- El argumento de venta debe identificar qué carencia del estándar resuelve

---

## 3. Página Web E-commerce (website_description)

### Formato
- **Tipo de campo**: `html`
- **Estructura**: Snippet `s_text_block` de Odoo website

### Template HTML obligatorio

```html
<section class="s_text_block pt40 pb40 o_colored_level" data-snippet="s_text_block" data-name="Texto" style="background-image: none;">
    <div class="container s_allow_columns">
        <!-- Módulo 1 -->
        <p><b>1) {Nombre Comercial del Módulo}</b></p>
        <p>Nombre técnico: {nombre_tecnico}</p>
        <p>{Descripción general de 2-3 frases. Sin jerga técnica. Enfocada en el beneficio.}</p>
        <ul>
            <li><b>{Nombre Funcionalidad}:</b> {Descripción clara de lo que hace, 1-2 frases}</li>
            <li><b>{Nombre Funcionalidad}:</b> {Descripción clara}</li>
            <li><b>{Nombre Funcionalidad}:</b> {Descripción clara}</li>
        </ul>

        <!-- Módulo 2 -->
        <p><b>2) {Nombre Comercial del Módulo}</b></p>
        <p>Nombre técnico: {nombre_tecnico}</p>
        <p>{Descripción}</p>
        <ul>
            <li>...</li>
        </ul>

        <!-- Repetir para cada módulo -->
    </div>
</section>
```

### Reglas para la web
- **Numerar** los módulos: 1), 2), 3)...
- **Nombre técnico** siempre debajo del título: `<p>Nombre técnico: {x}</p>`
- **Descripción introductoria** del módulo: 2-3 frases que expliquen el beneficio principal
- **Lista de funcionalidades**: 3-5 items con nombre en **negrita** + descripción
- **Sin jerga técnica** en los nombres de funcionalidades (usar nombres de negocio)
- **Sin estilos inline** en los elementos internos (Odoo gestiona el CSS)

### Ejemplo real (extracto): [CORE] Utilidades de facturación

```html
<p><b>1) Secciones Inteligentes para Facturas</b></p>
<p>Nombre técnico: indaws_account</p>
<p>Deja de usar las secciones de Odoo solo como un separador visual. Esta herramienta las convierte en carpetas de gestión, permitiéndote aplicar cambios en bloque y agilizar la creación de facturas complejas.</p>
<ul>
    <li><b>Organización Fácil:</b> Añade botones de "Añadir Sección" y "Añadir Nota" directamente en las líneas de la factura, para que no tengas que escribir comandos manuales.</li>
    <li><b>Gestión en Bloque:</b> Utiliza la nueva "configuración de sección" para aplicar impuestos o cuentas analíticas a <i>todas</i> las líneas que estén debajo de esa sección con un solo clic, ahorrando una enorme cantidad de tiempo.</li>
</ul>

<p><b>2) Asistente para Fusionar Facturas</b></p>
<p>Nombre técnico: indaws_account_move_merge</p>
<p>¿Tienes varios albaranes o pedidos del mismo cliente y Odoo te ha creado múltiples facturas en borrador? Deja de molestarlo con varios documentos y unifícalos.</p>
<ul>
    <li><b>Consolidación en un Clic:</b> Selecciona múltiples facturas borrador desde el listado y usa la nueva herramienta para fusionarlas automáticamente en una sola factura consolidada.</li>
    <li><b>Proceso Limpio y Seguro:</b> El sistema crea un nuevo documento con todas las líneas de las facturas originales y anula las antiguas, manteniendo tu contabilidad limpia y ordenada.</li>
</ul>
```

---

## 4. Descripción E-commerce (description_ecommerce)

### Formato
- **Tipo de campo**: `html`
- **Contenido**: Texto DIFERENTE a `description_sale`, envuelto en `<p>...</p>`
- **Aparece en**: ficha del producto en la tienda web (e-commerce)

### Tono
- Comercial, orientado a captar atención del visitante web
- **PUEDE incluir preguntas retóricas** y apelaciones directas al lector (a diferencia de description_sale)
- Enfocado en el problema que resuelve + beneficio

### IMPORTANTE
`description_sale` y `description_ecommerce` son textos **DISTINTOS** con enfoques diferentes:
- `description_sale` → profesional y directo (aparece en documentos de venta)
- `description_ecommerce` → comercial con gancho (aparece en la web)

### Ejemplo
`description_sale`:
```
Permite registrar las referencias, descripciones y precios específicos de cada cliente, y usarlos automáticamente en tus pedidos de venta.
```

`description_ecommerce`:
```html
<p>¿Tus clientes te piden que uses sus propios códigos de producto y descripciones en los pedidos y facturas? Este paquete te permite almacenar las referencias de producto de cada cliente y usarlas automáticamente en tus ventas, eliminando confusiones y agilizando la gestión.</p>
```

---

## Glosario de Traducción Técnico → Comercial

Usa esta tabla para traducir conceptos técnicos a lenguaje de negocio:

| Técnico | Comercial (web/ventas) |
|---------|----------------------|
| `_inherit` / herencia | "Potencia", "Amplía", "Mejora" |
| campo / field | "información", "dato" |
| modelo / model | "documento", "registro", "ficha" |
| vista / view | "pantalla", "interfaz" |
| wizard / asistente | "herramienta", "asistente" |
| cron / acción planificada | "ejecución automática", "programada" |
| constraint | "validación", "regla de seguridad" |
| compute | "se calcula automáticamente" |
| onchange | "se actualiza en tiempo real" |
| many2one | "vinculado a", "asociado a" |
| one2many | "lista de", "historial de" |
| ir.actions.server | "acción automática" |
| mail.thread | "seguimiento por correo" |
| ACL / ir.model.access | "permisos", "control de acceso" |
