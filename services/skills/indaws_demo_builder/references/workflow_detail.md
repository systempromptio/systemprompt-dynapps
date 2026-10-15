# Demo Builder - Workflow Detail

Reference con los pasos detallados del workflow de preparacion de demos.

---

## Paso 0: Recopilacion de Informacion

PREGUNTAR al usuario la **fuente de informacion** del cliente:

1. **ID de oportunidad CRM**: Lee directamente del CRM de Indaws
   - Necesita credenciales del CRM de Indaws (ODOO_URL, ODOO_DB, ODOO_KEY)
   - Ejecutar `scripts/fetch_opportunity.sh --opp-id ID` que extrae:
     - Campos de la oportunidad (name, description, expected_revenue, partner_id, etc.)
     - Mensajes del chatter (mail.message) - contexto de la conversacion comercial
     - Adjuntos (ir.attachment) - PDFs con informacion del cliente
   - Si hay PDFs adjuntos, descargar via la URL proporcionada y leer contenido

2. **Archivo o descripcion manual**: El usuario proporciona un archivo o texto describiendo al cliente

**Informacion adicional a obtener**:
- **Web de la empresa**: Usar WebFetch para investigar la empresa (productos, servicios, sector)
- **Sector**: Deducir del analisis o preguntar al usuario
- **Credenciales de la instancia demo**: URL, DB, API Key del Odoo donde preparar la demo

**VALIDACION 1**: Presentar al usuario:
```
Cliente: {nombre}
Sector detectado: {sector}
Web: {url}
Revenue esperado: {amount} EUR
Resumen: {breve_descripcion_de_lo_que_necesita}

¿Es correcto? ¿Algun ajuste?
```

> **DETENTE** aqui y espera la respuesta del usuario antes de continuar.

---

## Paso 1: Identificacion de Aplicaciones

ANALIZAR la informacion recopilada para identificar las apps de Odoo necesarias:

1. **Deducir apps** basandose en:
   - Descripcion de la oportunidad y notas del chatter
   - Sector del cliente (cada sector tiene apps tipicas en `sector_knowledge.md`)
   - Informacion de la web de la empresa
   - Productos/servicios que ofrece el cliente

2. **Presentar lista de apps** al usuario con justificacion:
```
Apps recomendadas para la demo:
1. Ventas - {razon}
2. Inventario - {razon}
3. CRM - {razon}
...

¿Deseas anadir o quitar alguna app?
```

**VALIDACION 2**: Esperar confirmacion del usuario. El usuario puede anadir/quitar apps.

> **DETENTE** aqui y espera la respuesta del usuario antes de continuar.

---

## Paso 2: Planificacion de la Demo

CONSULTAR las referencias necesarias:
1. `app_*.md` de **cada app seleccionada** (leer TODOS los archivos correspondientes)
2. `sector_knowledge.md` para datos realistas del sector
3. `flow_stages_guide.md` para distribucion de datos por estado

GENERAR el plan usando `templates/demo_plan.md`:

El plan debe incluir:
- **Modulos a instalar** (con justificacion)
- **Features a activar** (solo las necesarias)
- **Configuracion de empresa** (nombre, direccion, moneda, idioma, fiscalidad)
- **Datos maestros**: Contactos (clientes + proveedores), productos/servicios, empleados, etc.
  - Nombres REALISTAS del sector (NUNCA "Cliente 1", "Producto A")
  - Precios dentro de rangos del sector
  - Cantidades coherentes
- **Datos transaccionales**: Por cada app, documentos en diferentes estados
  - Seguir la distribucion de `flow_stages_guide.md`
  - Incluir historico (2-6 meses atras), actual y futuro
- **Puntos clave de la demo**: Momentos "wow" y flujo principal a mostrar

**VALIDACION 3**: Presentar el plan completo al usuario.
```
Plan de demo generado para {cliente}:
- {N} apps / {N} modulos
- {N} clientes + {N} proveedores
- {N} productos/servicios
- {N} documentos transaccionales
- Puntos clave: {lista_breve}

[Mostrar plan detallado con tablas]

¿Apruebas el plan? ¿Algun cambio?
```

> **DETENTE** aqui y espera la aprobacion del usuario antes de continuar.
> El usuario puede pedir ajustes (mas/menos datos, cambiar productos, etc.)

---

## Paso 3: Conexion e Instalacion

> **SOLO** ejecutar tras la aprobacion del plan (Paso 2).

1. **Configurar credenciales** de la instancia demo:
```bash
export ODOO_URL="<url_demo>"
export ODOO_DB="<db_demo>"
export ODOO_KEY="<key_demo>"
```

2. **Autenticar** via odoo-pilot (auth.sh)

3. **Instalar modulos** segun el plan:
```bash
# Para cada modulo del plan
execute_method.sh ir.module.module 0 search_read \
  '[["name","=","MODULE_NAME"]]' '["state"]'

# Si no esta instalado:
execute_method.sh ir.module.module MODULE_ID button_immediate_install
```

4. **Configurar empresa**:
   - Actualizar res.company con nombre, direccion, moneda, logo
   - Configurar idioma (base.language.install si necesario)
   - Aplicar configuracion fiscal (posiciones fiscales, impuestos)

5. **Activar features** necesarias via res.config.settings:
```bash
# Leer configuracion actual
execute_method.sh res.config.settings 0 create '{}'
# Actualizar y ejecutar
update_record.sh res.config.settings SETTINGS_ID '{"feature_name": true}'
execute_method.sh res.config.settings SETTINGS_ID execute
```

**Informar al usuario** del progreso de instalacion.

---

## Paso 4: Datos Maestros

Crear los datos maestros segun el plan aprobado:

1. **Contactos** (clientes + proveedores):
```bash
create_record.sh res.partner '{"name": "...", "is_company": true, "city": "...", "email": "...", "phone": "..."}'
```
   - Crear primero las empresas, luego los contactos hijos
   - Asignar posiciones fiscales, terminos de pago, listas de precios

2. **Categorias de producto**:
```bash
create_record.sh product.category '{"name": "Categoria"}'
```

3. **Productos/Servicios**:
```bash
create_record.sh product.template '{"name": "...", "type": "product", "list_price": 100, "categ_id": CAT_ID}'
```
   - Tipo "product" para almacenables, "service" para servicios, "consu" para consumibles
   - Asignar proveedores (product.supplierinfo)
   - Ajustar stock inicial (stock.quant)

4. **Empleados** (si RRHH activo):
```bash
create_record.sh hr.employee '{"name": "...", "department_id": DEPT_ID, "job_id": JOB_ID}'
```

5. **Otros maestros** segun apps:
   - Almacenes y ubicaciones (stock)
   - Equipos de ventas (crm.team)
   - Proyectos y etapas (project)
   - Equipos de helpdesk (helpdesk.team)
   - Centros de trabajo y BoMs (mrp)
   - Planes de suscripcion
   - Configuracion POS
   - etc.

**Registrar los IDs creados** para usarlos en los datos transaccionales.

---

## Paso 5: Datos Transaccionales (Flujos)

Por cada app del plan, crear datos en los diferentes estados siguiendo la distribucion aprobada.

**IMPORTANTE**: Respetar el orden de creacion:
1. Primero los documentos mas antiguos (historico, 2-6 meses atras)
2. Luego los del mes actual
3. Finalmente los futuros/planificados

**Para cada documento**:
1. **Crear** el registro en estado borrador
2. **Avanzar** al estado deseado usando los metodos API correspondientes:
   - Ventas: `action_confirm` -> picking `button_validate` -> `_create_invoices` -> `action_post`
   - Compras: `button_confirm` -> picking `button_validate` -> factura proveedor
   - Inventario: `action_confirm` -> `action_assign` -> `button_validate`
   - Contabilidad: `action_post` -> registrar pago
   - CRM: mover stage_id, `action_set_won`, `action_set_lost`
   - Fabricacion: `action_confirm` -> `button_mark_done`
   - Proyecto: mover stage_id, registrar horas (account.analytic.line)
   - RRHH: `action_confirm` -> `action_validate`
   - Helpdesk: mover stage_id
   - Suscripciones: `action_confirm`, `set_paused`, `set_close`

3. **Ajustar fechas** para crear historico realistico:
   - Usar `write` para modificar create_date, date_order, etc. a fechas pasadas
   - Distribuir temporalmente segun el plan

4. **Anadir contexto**:
   - Notas en el chatter de 2-3 registros clave (mail.message)
   - Actividades programadas en documentos actuales (mail.activity)
   - Adjuntos en pedidos importantes (opcional)

**Informar al usuario** del progreso de creacion de datos.

---

## Paso 6: Verificacion y Resumen

1. **Verificar** que todos los datos estan correctos:
   - Contar registros creados por modelo
   - Verificar que hay datos en todos los estados previstos
   - Comprobar que los dashboards de cada app tienen contenido

2. **Presentar resumen** al usuario:
```
Demo preparada para {cliente} ({sector})
Instancia: {url}

Resumen de datos creados:
- {N} contactos (clientes + proveedores)
- {N} productos/servicios
- {N} empleados
- {N} presupuestos/pedidos de venta
- {N} pedidos de compra
- {N} facturas
- {N} oportunidades CRM
- ...
Total: {N} registros

Apps configuradas: {lista}
```

3. **Sugerir puntos clave** para la demo:
```
Puntos clave para la demo:
1. {punto_1}: navegar a {ruta}, mostrar {funcionalidad}
2. {punto_2}: ...
3. {punto_3}: ...

Documentos preparados para interactuar:
- Presupuesto {nombre} (draft): confirmar en vivo
- Oportunidad {nombre}: mover en pipeline
- ...
```

---

## Estructura de Archivos

```
demo-builder/
├── SKILL.md                              # Skill principal (resumen)
├── references/
│   ├── workflow_detail.md                # Este archivo - pasos detallados
│   ├── app_ventas.md                     # Sales: config + datos + flujos
│   ├── app_compras.md                    # Purchases: config + datos + flujos
│   ├── app_inventario.md                 # Inventory: config + datos + flujos
│   ├── app_contabilidad.md               # Accounting: config + datos + flujos
│   ├── app_crm.md                        # CRM: config + datos + flujos
│   ├── app_fabricacion.md                # Manufacturing: config + datos + flujos
│   ├── app_proyecto.md                   # Project: config + datos + flujos
│   ├── app_rrhh.md                       # HR: config + datos + flujos
│   ├── app_helpdesk.md                   # Helpdesk: config + datos + flujos
│   ├── app_website_ecommerce.md          # Website/eCommerce: config + datos + flujos
│   ├── app_punto_de_venta.md             # Point of Sale: config + datos + flujos
│   ├── app_suscripciones.md              # Subscriptions: config + datos + flujos
│   ├── sector_knowledge.md               # Conocimiento de sectores para datos realistas
│   └── flow_stages_guide.md              # Guia de distribucion de datos por estado
├── templates/
│   └── demo_plan.md                      # Template del plan de demo
└── scripts/
    └── fetch_opportunity.sh              # Extrae oportunidad del CRM
```

---

## Success Criteria

- [ ] Informacion del cliente recopilada y validada (Paso 0)
- [ ] Apps identificadas y aprobadas por el usuario (Paso 1)
- [ ] Plan de demo generado y aprobado por el usuario (Paso 2)
- [ ] Modulos instalados y features configuradas (Paso 3)
- [ ] Datos maestros creados (contactos, productos, empleados) (Paso 4)
- [ ] Datos transaccionales creados en todos los estados previstos (Paso 5)
- [ ] Verificacion completada y resumen presentado (Paso 6)
- [ ] Nombres y datos realistas del sector (no genericos)
- [ ] Distribucion temporal: historico + actual + futuro
