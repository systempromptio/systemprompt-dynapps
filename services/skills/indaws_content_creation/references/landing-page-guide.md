# Landing Page Guide — Indaws Content Creation

Paginas de conversion en Odoo Website Builder. Objetivo: capturar leads o dirigir a accion especifica.

## Structure

```
┌────────────────────────────────────────────────────────────────┐
│  HERO SECTION                                                   │
│  - H1: Propuesta de valor clara (max 10 palabras)               │
│  - Subtitulo: Ampliar beneficio (1-2 lineas)                    │
│  - CTA primario: Boton Blue Lilac (#6B68FA)                     │
│  - Imagen/visual: Screenshot Odoo o foto equipo                 │
├────────────────────────────────────────────────────────────────┤
│  PROBLEMA / DOLOR                                                │
│  - 3 pain points del buyer persona                               │
│  - Formato: icono + titulo + descripcion breve                   │
│  - Tono: empatia, reconocer el reto                              │
├────────────────────────────────────────────────────────────────┤
│  SOLUCION / BENEFICIOS                                           │
│  - 3 beneficios principales de la solucion Indaws                │
│  - Formato: icono + titulo + descripcion breve                   │
│  - Conectar cada beneficio con un pain point                     │
├────────────────────────────────────────────────────────────────┤
│  PRUEBA SOCIAL                                                   │
│  - 1-2 casos de exito resumidos (de content-collection)          │
│  - Cifra de impacto + cita del cliente                           │
│  - Logos de clientes (si hay permiso)                            │
├────────────────────────────────────────────────────────────────┤
│  CTA SECUNDARIO                                                  │
│  - Formulario de contacto o booking link                         │
│  - Campos minimos: nombre, email, empresa, telefono              │
│  - Boton: texto especifico ("Solicita tu diagnostico gratuito")  │
├────────────────────────────────────────────────────────────────┤
│  FOOTER                                                          │
│  - Datos de contacto Indaws                                      │
│  - Links legales                                                 │
└────────────────────────────────────────────────────────────────┘
```

## Odoo Website Builder Considerations

- Usar snippets nativos de Odoo cuando sea posible
- Formularios conectados a `crm.lead` para captura automatica
- Paginas SEO-optimizadas (meta title, description, slug limpio)
- Responsive: testear en movil antes de publicar
- Velocidad: optimizar imagenes (WebP, lazy loading)

## Web Channel Writing Guide

| Aspecto | Guia |
|---------|------|
| Longitud | Lo minimo necesario para convertir |
| Tono | Orientado a beneficio, action-driven |
| Estructura | Jerarquia visual clara, F-pattern de lectura |
| CTA | Visible sin scroll. Repetir al final |
| Formularios | Campos minimos (nombre, email, empresa, telefono) |
| Prueba social | Casos de exito, cifras, logos clientes |
| Mobile | Disenar mobile-first, testear antes de publicar |

## Case Study Structure

Formato propio de Indaws: **Reto - Solucion - Resultados**. Pieza clave para MOFU/BOFU.

```
┌────────────────────────────────────────────────────────────────┐
│  HEADER                                                         │
│  - Titulo: "Caso de exito: [Empresa] — [Resultado clave]"      │
│  - Sector + tamano empresa                                      │
│  - Modulos Indaws Core implementados                            │
├────────────────────────────────────────────────────────────────┤
│  RETO (150-200 palabras)                                         │
│  - Situacion inicial del cliente                                 │
│  - Pain points concretos (con datos si es posible)               │
│  - Por que las soluciones anteriores no funcionaban              │
├────────────────────────────────────────────────────────────────┤
│  SOLUCION (200-300 palabras)                                     │
│  - Que se implemento (modulos Odoo + personalizaciones)          │
│  - Proceso de implementacion (fases, duracion)                   │
│  - Decisiones clave de diseno                                    │
├────────────────────────────────────────────────────────────────┤
│  RESULTADOS (150-200 palabras)                                   │
│  - 3-5 metricas cuantificadas (antes vs despues)                 │
│  - ROI o impacto en negocio                                      │
│  - Cita textual del cliente (1-2 frases)                         │
├────────────────────────────────────────────────────────────────┤
│  MODULOS RELACIONADOS                                            │
│  - Lista de modulos Indaws Core utilizados                       │
│  - Link a pagina de cada modulo                                  │
├────────────────────────────────────────────────────────────────┤
│  CTA                                                             │
│  - "Quieres resultados similares? Hablemos de tu caso."         │
│  - Boton a formulario de contacto                                │
└────────────────────────────────────────────────────────────────┘
```

### Results Quantification Example

```
ANTES                              DESPUES
─────                              ──────
Cierre mensual: 5 dias             Cierre mensual: 4 horas
Stock entre tiendas: manual        Stock sincronizado en tiempo real
Contabilidad: hojas de calculo     Contabilidad integrada con ventas
Visibilidad: nula                  Dashboards en tiempo real
```

## Content Repurposing Matrix

```
CASO DE EXITO (pieza madre)
│
├── Blog post largo (1500 palabras)
├── LinkedIn post MOFU (200 palabras)
├── Email en secuencia de nurturing
├── Slide en presentacion de ventas
├── Quote card para social (visual)
├── Dato para infografia
└── Referencia en landing page (prueba social)

BLOG POST (pieza madre)
│
├── 2-3 LinkedIn posts (extractos con angulo diferente)
├── Newsletter highlight
├── Carrusel visual (5-7 slides)
├── Video corto / Reel (resumen 60 seg)
└── Podcast topic / guion

EVENTO (pieza madre)
│
├── 4 LinkedIn posts (pre + post)
├── 3 emails (invitacion, recordatorio, follow-up)
├── Blog post recap
├── Video resumen
├── Fotos para social
└── Caso de exito (si hay resultados medibles)
```
