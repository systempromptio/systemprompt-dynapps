---
name: indaws-demo-builder
description: >
  Prepara demos de Odoo completas con datos realistas a partir de oportunidades CRM
  o descripciones. Configura instancias e inyecta datos en diferentes estados de flujo
  para que la demo sea impactante. 12 guias de apps, 12 perfiles de sector.
  Usar cuando el usuario pida: preparar demo, montar demo, demo builder, crear datos
  de demo, configurar instancia demo, demo para [cliente].
version: 1.0.1
model: opus
effort: high
---

# Demo Builder

Prepara demos de Odoo completas con datos realistas a partir de oportunidades CRM o descripciones. Configura instancias e inyecta datos en diferentes estados de flujo para que la demo sea impactante.

## Prerequisitos

- **odoo-pilot**: Skill de `ai-claude-common` para conexion a Odoo via API
- **Credenciales CRM Indaws** (opcional): Para leer oportunidades directamente
- **Credenciales instancia demo** (obligatorio): URL, DB, API Key de la instancia destino
- See references/shared/odoo-connection.md for connection pattern.

## Workflow

> Detalle completo de cada paso en `references/workflow_detail.md`

- **Paso 0 - Recopilar info**: Desde CRM (oportunidad ID), archivo/descripcion manual, o web del cliente. Investigar sector. **VALIDACION 1**: confirmar cliente + sector + resumen.
- **Paso 1 - Identificar apps**: Deducir apps necesarias del sector y necesidades. 12 apps soportadas. **VALIDACION 2**: confirmar lista de apps.
- **Paso 2 - Planificar demo**: Leer `references/app_*.md` de cada app + `references/sector_knowledge.md` + `references/flow_stages_guide.md`. Generar plan con datos maestros + transaccionales por estado. **VALIDACION 3**: aprobar plan completo.
- **Paso 3 - Conectar e instalar**: Configurar credenciales demo, autenticar via odoo-pilot, instalar modulos, configurar empresa y features.
- **Paso 4 - Inyectar datos maestros**: Contactos (empresas + hijos), categorias, productos/servicios, empleados, y otros maestros segun apps. Registrar IDs para referencias cruzadas.
- **Paso 5 - Inyectar datos transaccionales**: Crear documentos en diferentes estados (80% historicos / 20% actuales). Orden: historico (2-6 meses) -> actual -> futuro. Avanzar estados via API. Anadir contexto (chatter, actividades).
- **Paso 6 - Verificar y presentar**: Contar registros por modelo, verificar estados, presentar resumen al comercial con puntos clave para la demo.

> **3 paradas obligatorias** (Validaciones 1, 2, 3): NUNCA avanzar sin aprobacion del usuario.

## Rules

1. **3 validaciones obligatorias** antes de ejecutar: cliente (Paso 0), apps (Paso 1), plan (Paso 2).
2. **Nombres realistas** del sector siempre. NUNCA "Cliente 1", "Producto A".
3. **Consultar sector_knowledge.md** para precios, cantidades y nombres del sector.
4. **Distribucion de estados** segun flow_stages_guide.md. NUNCA todo en el mismo estado.
5. **Leer app_*.md** de cada app seleccionada antes de planificar.
6. **Datos historicos obligatorios** (2-6 meses atras) para dar profundidad.
7. **Registrar IDs** creados para referencias cruzadas entre documentos.
8. Precios y cantidades coherentes con el sector (un particular no compra 500 palets).
9. Dependencias primero: si ventas necesita productos, crear productos antes.
10. Si la conexion falla, avisar al usuario inmediatamente. No instalar modulos innecesarios.

## Gotchas

1. `product.template` type changed: v18 usa `"consu"` por defecto, `"product"` requiere `stock` instalado.
2. `res.config.settings` requiere `create` -> `write` -> `execute` (no write directo).
3. Fechas pasadas en documentos: algunos campos computed se recalculan; usar `write` post-creacion.
4. `button_validate` en pickings puede lanzar wizard (immediate.transfer); capturar el return action.
5. `_create_invoices` es metodo de sale.order, no de account.move. Requiere pedido confirmado + entregado.
6. Instalar modulos en orden: dependencias base primero (sale antes de sale_management).
7. CRM stages son por equipo; crear equipo de ventas antes de mover oportunidades entre stages.

## Agent Routing

**Model**: opus (multi-phase data injection, sector reasoning, realistic data generation)

### Agent() condicional — Pattern A "Scan First" (solo si config avanzada)

Si la instancia demo requiere configuracion avanzada (multi-company, localizacion, contabilidad compleja):

```
Agent({
  description: "Configure demo instance",
  subagent_type: "general-purpose",
  model: "sonnet",
  prompt: "Use consultant-skills:odoo-configurator to configure the demo instance at {demo_url}. Apps to configure: {app_list}. Country: Spain. Apply standard Indaws configuration."
})
```

**NO es Agent()**: odoo-pilot (Bash inline — create_record.sh, update_record.sh, execute_method.sh).

## Delegates to

| Skill | Suite | How | Agent()? |
|-------|-------|-----|----------|
| odoo-pilot | common | Bash inline (CRUD scripts) | No |
| odoo-configurator | consultant | Config avanzada de instancia | **Condicional — Pattern A** |

## Consumed by

- **demo-script-generator** (sales) - opcional, usa datos de demo como contexto para guion

## References

| File | Content |
|------|---------|
| `references/workflow_detail.md` | Pasos 0-6 detallados, validaciones, code blocks, estructura archivos, success criteria |
| `references/app_ventas.md` | Sales: config + datos + flujos |
| `references/app_compras.md` | Purchases: config + datos + flujos |
| `references/app_inventario.md` | Inventory: config + datos + flujos |
| `references/app_contabilidad.md` | Accounting: config + datos + flujos |
| `references/app_crm.md` | CRM: config + datos + flujos |
| `references/app_fabricacion.md` | Manufacturing: config + datos + flujos |
| `references/app_proyecto.md` | Project: config + datos + flujos |
| `references/app_rrhh.md` | HR: config + datos + flujos |
| `references/app_helpdesk.md` | Helpdesk: config + datos + flujos |
| `references/app_website_ecommerce.md` | Website/eCommerce: config + datos + flujos |
| `references/app_punto_de_venta.md` | Point of Sale: config + datos + flujos |
| `references/app_suscripciones.md` | Subscriptions: config + datos + flujos |
| `references/sector_knowledge.md` | Conocimiento de sectores para datos realistas |
| `references/flow_stages_guide.md` | Guia de distribucion de datos por estado |
