# Especificacion de Campos del Formulario de Cualificacion

> CONFIDENTIAL - SALES TEAM ONLY
> Documento ID: SALES-SKILL-013-REF

## Mapeo de campos por seccion

### 1. Actividad del Negocio

| Campo | Fuente primaria | Fuente secundaria | Campo Odoo |
|-------|----------------|-------------------|------------|
| Sector/Actividad | eInforma `cnae_description` | Keywords emails | `industry_id` |
| Codigo CNAE | eInforma `cnae_code` | — | — |

### 2. Dimensionamiento

| Campo | Fuente primaria | Fuente secundaria | Campo Odoo |
|-------|----------------|-------------------|------------|
| Facturacion | eInforma `turnover_eur` | `expected_revenue` | `expected_revenue` |
| Empleados | eInforma `employees` | — | `res.partner.employee` |
| Usuarios ERP | Keywords emails | — | — |
| Provincia | eInforma `province` | — | `res.partner.state_id` |
| Ciudad | eInforma `city` | — | `res.partner.city` |
| Forma juridica | eInforma `legal_form` | — | — |
| Fundada | eInforma `founded_year` | — | — |

### 3. Sistema Actual

| Valor | Keywords deteccion |
|-------|-------------------|
| Odoo | odoo, openerp |
| Excels | excel, hoja de calculo, spreadsheet |
| Varios sistemas | varios sistema, multiples herramienta, diferentes programa |
| ERP tercero | sap, dynamics, navision, sage, a3, holded, factusol, contaplus, oracle, netsuite, primavera |

Subcampos (si Odoo): edicion (Community/Enterprise), version, partner actual.

### 4. Pain Points

10 pain points predefinidos con id, emoji, titulo y descripcion:

| ID | Emoji | Keywords principales |
|----|-------|---------------------|
| dispersos | 🗂️ | disperso, multiple, herramienta, separado |
| manuales | 🔁 | manual, error, duplica, repetir |
| reportes | 📊 | reporte, informe, dato, fiable |
| caro | 💸 | caro, coste, licencia, precio |
| escala | 🧱 | escala, limite, crecimiento, obsoleto |
| integracion | 🔌 | integracion, conectar, api, ecommerce |
| dependencia | 🧑‍💻 | dependencia, proveedor, consultor |
| inventario | 📦 | inventario, stock, almacen, rotura |
| experiencia | 👥 | lento, dificil, complejo, interfaz |
| visibilidad | 🚫 | visibilidad, seguimiento, trazabilidad |

### 5. Modulos de Interes

15 modulos Odoo detectables por keywords:

CRM, Ventas, Compras, Inventario, Contabilidad, Facturacion,
Fabricacion (MRP), Proyectos, SAT (Servicio Posventa), Suscripciones,
Recursos Humanos, Control Horario, eCommerce, Punto de Venta (POS),
Marketing / Emailing.

Ver `config/qualification_fields.json` para keywords completos por modulo.

### 6. Presupuesto & Timeline

| Estado | Keywords |
|--------|---------|
| Inversion definida | presupuesto, inversion, aprobado, asignado |
| Fase comparativa | comparar, evaluar, alternativa, propuesta |

Timeline keywords: urgente, plazo, fecha, trimestre, semestre, mes.

### 7. Mapa de Decisores

Keywords de roles: director, gerente, ceo, cto, cfo, responsable, jefe.
Roles BANT: Decision Maker, Champion, Influencer, Blocker.
Max 5 decisores por oportunidad.

## Badges de confianza

| Badge | Color | Significado |
|-------|-------|-------------|
| AUTO | #28a745 (verde) | Dato verificado de API (eInforma) |
| PARCIAL | #ffc107 (amarillo) | Dato inferido de emails por keywords |
| PENDIENTE | #dc3545 (rojo) | Dato no disponible, requiere input manual |

## Branding

| Elemento | Color |
|----------|-------|
| Primary (headers, labels) | #714B67 |
| Secondary (chips, next step) | #00A09D |
| Background seccion | #F3F0F5 |
| Border | #D9D0E3 |
| Pain point background | #FFF8F0 |
| Pain point border | #E8A020 |
| Next step background | #E8F7F7 |
