# Plan de Demo - {EMPRESA_CLIENTE}

## Informacion del Cliente

| Campo | Valor |
|-------|-------|
| **Empresa** | {nombre_empresa} |
| **Sector** | {sector} |
| **Contacto** | {contacto_nombre} |
| **Web** | {web_empresa} |
| **Origen** | {fuente: oportunidad CRM / descripcion manual} |
| **Revenue esperado** | {expected_revenue} EUR |

## Resumen de la Demo

{descripcion_breve_de_que_necesita_el_cliente_y_que_se_va_a_mostrar}

---

## 1. Aplicaciones y Modulos

### Modulos a Instalar

| App | Modulos | Justificacion |
|-----|---------|---------------|
| {app_1} | {modulo_1, modulo_2} | {razon_por_la_que_se_incluye} |
| {app_2} | {modulo_3, modulo_4} | {razon} |
| ... | ... | ... |

### Features a Activar

| Feature | App | Razon |
|---------|-----|-------|
| {feature_1} | {app} | {por_que_activar} |
| ... | ... | ... |

---

## 2. Configuracion de la Instancia

### Datos de Empresa
- **Nombre**: {nombre_empresa_cliente}
- **Direccion**: {direccion}
- **NIF/CIF**: {nif_ficticio}
- **Moneda**: EUR
- **Idioma**: Espanol
- **Localizacion fiscal**: {l10n_es / otra}

### Configuracion por App

{Para cada app seleccionada, listar los ajustes especificos que se van a configurar, basandose en las guias de references/app_*.md}

---

## 3. Datos Maestros

### Contactos

#### Clientes ({N} contactos)

| # | Nombre | Tipo | Ciudad | Sector/Nota |
|---|--------|------|--------|-------------|
| 1 | {nombre_cliente_1} | Empresa | {ciudad} | {nota} |
| 2 | {nombre_cliente_2} | Empresa | {ciudad} | {nota} |
| ... | ... | ... | ... | ... |

#### Proveedores ({N} contactos)

| # | Nombre | Ciudad | Productos que suministra |
|---|--------|--------|------------------------|
| 1 | {nombre_proveedor_1} | {ciudad} | {productos} |
| ... | ... | ... | ... |

### Productos / Servicios

#### Productos ({N} productos)

| # | Nombre | Tipo | Categoria | Precio Venta | Precio Compra | Stock Inicial |
|---|--------|------|-----------|-------------|---------------|---------------|
| 1 | {nombre_producto_1} | Almacenable | {cat} | {precio} EUR | {precio} EUR | {qty} |
| 2 | {nombre_producto_2} | Servicio | {cat} | {precio} EUR | - | - |
| ... | ... | ... | ... | ... | ... | ... |

### Empleados (si RRHH activo)

| # | Nombre | Departamento | Puesto | Manager |
|---|--------|-------------|--------|---------|
| 1 | {nombre_1} | {dept} | {puesto} | {manager} |
| ... | ... | ... | ... | ... |

### Otros Maestros

{Listar otros datos maestros necesarios segun las apps: almacenes, equipos de venta, proyectos, etapas CRM, etc.}

---

## 4. Datos Transaccionales

### Distribucion por App

{Para cada app, listar los documentos a crear con su estado y cantidad, siguiendo la distribucion de references/flow_stages_guide.md}

#### {App 1: Ventas}

| Documento | Estado | Cantidad | Periodo | Detalle |
|-----------|--------|----------|---------|---------|
| Presupuesto | Borrador | {N} | Actual | {descripcion} |
| Pedido | Confirmado | {N} | Actual | {descripcion} |
| Pedido | Entregado | {N} | Historico | {descripcion} |
| Factura | Pagada | {N} | Historico | {descripcion} |

#### {App 2: ...}

| Documento | Estado | Cantidad | Periodo | Detalle |
|-----------|--------|----------|---------|---------|
| ... | ... | ... | ... | ... |

---

## 5. Puntos Clave de la Demo

### Momentos "Wow"
1. {momento_wow_1: funcionalidad que impresionara al cliente}
2. {momento_wow_2}
3. {momento_wow_3}

### Flujo Principal a Mostrar
1. {paso_1_del_flujo_principal}
2. {paso_2}
3. {paso_3}
4. ...

### Preguntas Anticipadas
- {pregunta_probable_1}: {respuesta_preparada}
- {pregunta_probable_2}: {respuesta_preparada}

---

## 6. Resumen de Volumen

| Categoria | Cantidad |
|-----------|----------|
| Apps/Modulos | {N} |
| Contactos (clientes) | {N} |
| Contactos (proveedores) | {N} |
| Productos/Servicios | {N} |
| Empleados | {N} |
| Documentos transaccionales | {N} |
| Total registros estimados | {N} |

---

> **NOTA**: Este plan requiere aprobacion del usuario antes de proceder con la configuracion e inyeccion de datos.
