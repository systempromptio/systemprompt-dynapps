# Blog Post Guide — Indaws Content Creation

## Structure

```
┌────────────────────────────────────────────────────────────────┐
│  H1: Titulo principal (60-70 caracteres, keyword principal)    │
├────────────────────────────────────────────────────────────────┤
│  Meta description (150-160 caracteres, incluye keyword + CTA)  │
├────────────────────────────────────────────────────────────────┤
│  Intro / Hook (100-150 palabras)                                │
│  - Plantear el reto del lector                                  │
│  - Anticipar la solucion                                        │
│  - Establecer credibilidad (dato, cifra, caso real)             │
├────────────────────────────────────────────────────────────────┤
│  H2: Seccion 1 — Contexto / Problema                           │
│  (200-300 palabras, datos que respaldan el reto)                │
├────────────────────────────────────────────────────────────────┤
│  H2: Seccion 2 — Solucion / Enfoque                            │
│  (300-400 palabras, como se resuelve, paso a paso)              │
├────────────────────────────────────────────────────────────────┤
│  H2: Seccion 3 — Caso practico / Ejemplo real                  │
│  (200-300 palabras, caso de exito Indaws si aplica)             │
├────────────────────────────────────────────────────────────────┤
│  H2: Seccion 4 — Beneficios / Resultados (opcional)            │
│  (150-200 palabras, metricas, ROI, mejoras concretas)           │
├────────────────────────────────────────────────────────────────┤
│  H2: Seccion 5 — Conclusion + CTA                              │
│  (100-150 palabras, resumen, siguiente paso claro)              │
├────────────────────────────────────────────────────────────────┤
│  CTA block: Boton o enlace a demo/guia/contacto                 │
└────────────────────────────────────────────────────────────────┘
```

**Format**: 1000-2000 palabras. Publicado en Odoo Blog (www.indaws.es/blog).

## Channel Writing Guide

| Aspecto | Guia |
|---------|------|
| Longitud | 1000-2000 palabras |
| Tono | Educativo, experto pero accesible |
| Estructura | H2s claros, parrafos cortos (3-4 lineas max), listas |
| SEO | Keyword en H1, primera frase, H2s, meta desc |
| Links internos | Minimo 2-3 links a otras paginas de indaws.es |
| Imagenes | 1 imagen cada 300-400 palabras, alt text descriptivo |
| CTA | Uno principal al final, puede haber inline CTAs suaves |
| Frecuencia | 1-2 posts por semana |

## Odoo Blog Field Mapping

| Campo Odoo | Descripcion | Ejemplo |
|------------|-------------|---------|
| `name` | Titulo visible del post | "Como reducir el cierre contable de 5 dias a 4 horas" |
| `subtitle` | Subtitulo (aparece bajo el titulo) | "Caso real de una cadena de tiendas con Odoo" |
| `seo_name` | Slug URL (sin acentos, guiones) | "reducir-cierre-contable-odoo" |
| `website_meta_title` | Title tag SEO (max 60 chars) | "Reducir cierre contable con Odoo | Indaws" |
| `website_meta_description` | Meta description (max 160 chars) | "Descubre como una cadena de tiendas redujo su cierre mensual de 5 dias a 4 horas con Odoo. Caso real con resultados medibles." |
| `tag_ids` | Tags del blog | ["ERP", "Contabilidad", "Caso de exito"] |
| `content` | Cuerpo HTML del post | HTML con clases Odoo |
| `author_id` | Autor | Partner ID del autor |

## Intro Example

```
Tu equipo de finanzas dedica la primera semana de cada mes a "cerrar".
Facturas pendientes, conciliaciones bancarias, asientos manuales. Y mientras
tanto, la direccion toma decisiones con datos del mes anterior.

No es un problema de personas. Es un problema de procesos y herramientas.

En este articulo te mostramos como una cadena de tres tiendas paso de cerrar
en 5 dias a hacerlo en 4 horas. Sin cambiar de equipo. Solo cambiando como
fluye la informacion entre ventas, inventario y contabilidad.
```

## SEO On-Page Checklist

Apply to EVERY web content piece before publishing:

```
[ ] Title tag: 50-60 caracteres, keyword principal al inicio, " | Indaws" al final
[ ] Meta description: 150-160 caracteres, keyword + propuesta de valor + CTA implicito
[ ] H1: unico por pagina, incluye keyword principal
[ ] H2s: incluyen variaciones de keyword o keywords secundarios
[ ] URL slug: limpio, sin acentos, con guiones, incluye keyword (seo_name en Odoo)
[ ] Primer parrafo: keyword principal en las primeras 100 palabras
[ ] Internal links: minimo 2-3 links a otras paginas relevantes de indaws.es
[ ] External links: 1-2 links a fuentes autoritativas (Odoo.com, estudios, etc.)
[ ] Image alt text: descriptivo, incluye keyword cuando es natural
[ ] Image filename: descriptivo (no IMG_001.jpg)
[ ] Image format: WebP preferido, lazy loading activado
[ ] Content length: minimo 800 palabras para posts, 300 para paginas
[ ] Readability: parrafos max 3-4 lineas, listas, subtitulos cada 200-300 palabras
```

## Keyword Strategy

1. **Investigar antes de escribir**: Consultar seo-manager para datos de DataForSEO
2. **Un keyword principal por pagina**: Evitar canibalizacion
3. **Long-tail para blog**: Frases de 3-5 palabras con intencion informacional
4. **Short-tail para paginas de servicio**: Terminos competitivos con intencion transaccional
5. **Cluster model**: Pagina pilar + posts satelite enlazados

| Tipo | Ejemplo | Uso |
|------|---------|-----|
| Marca | "indaws", "indaws odoo" | Homepage, about |
| Servicio | "consultoria odoo", "implantacion erp" | Paginas servicio |
| Problema | "como automatizar facturacion", "erp para pymes" | Blog posts |
| Comparativa | "odoo vs sap", "mejor erp espana" | Blog posts |
| Caso de uso | "erp fabricacion", "odoo comercio" | Landing pages |

## Content-SEO Integration Process

```
1. Consultar seo-manager para keywords target y volumenes
2. Verificar que no existe contenido similar (content-collection)
3. Seleccionar keyword principal + 2-3 secundarios
4. Escribir contenido siguiendo template + checklist on-page
5. Publicar en Odoo via odoo-pilot
6. Verificar indexacion (Google Search Console)
7. Monitorizar posicion en weekly SEO report
```

## Press Release (Nota de Prensa)

Uso poco frecuente. Para anuncios relevantes (nuevo partner status, expansiones, premios).

```
TITULAR: [Empresa] + [Accion] + [Resultado/Contexto]
SUBTITULO: Ampliacion en 1 linea

PRIMER PARRAFO (Piramide invertida):
- Que, quien, cuando, donde, por que
- Lo mas importante primero

SEGUNDO PARRAFO:
- Contexto y detalles

TERCER PARRAFO:
- Cita del portavoz (CEO o director relevante)

CUARTO PARRAFO:
- Datos adicionales, cifras, background

SOBRE INDAWS (Boilerplate):
- Descripcion estandar de la empresa
- Datos clave: Gold Partner, equipo, clientes, sede

CONTACTO DE PRENSA:
- Nombre, email, telefono
```
