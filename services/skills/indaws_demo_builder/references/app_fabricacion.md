# Fabricacion (Manufacturing) - Guia de Configuracion y Datos para Demo

## Modulos

| Modulo | Descripcion | Cuando instalar |
|--------|-------------|-----------------|
| mrp | Fabricacion base | Siempre (si app seleccionada) |
| mrp_workorder | Ordenes de trabajo | Si tiene centros de trabajo diferenciados |
| mrp_account | Valoracion de produccion | Si necesita costeo de fabricacion |
| mrp_subcontracting | Subcontratacion | Si externaliza parte de la produccion |
| quality_control | Control de calidad | Si tiene procesos de QC |
| maintenance | Mantenimiento | Si gestiona maquinaria |

## Configuracion

### Ajustes Principales
- **Ordenes de trabajo**: Activar si hay centros de trabajo (maquinas, estaciones)
- **Subcontratacion**: Activar si externaliza operaciones
- **Control de calidad**: Activar si hay puntos de control QC
- **Planificacion por capacidad**: Activar para demo de planificacion avanzada
- **Lotes/Series**: Activar si necesita trazabilidad

### Features Opcionales
- Mantenimiento preventivo de equipos
- PLM (Product Lifecycle Management) para gestion de versiones de BoM
- Kit de materiales (sin MO, despiece automatico en entrega)

## Datos Maestros

### Listas de Materiales (BoM) - 3-5 BoMs
- Al menos 2 BoMs de fabricacion (type=normal)
- 1 BoM tipo kit si aplica (type=phantom)
- Componentes con stock disponible
- Al menos 1 BoM multinivel (componente que a su vez tiene BoM)
- Cantidades y unidades de medida correctas

### Centros de Trabajo (si workorders activos)
- 2-3 centros de trabajo (ej: Corte, Ensamblaje, Acabado)
- Capacidad y horarios configurados
- Costes por hora

### Operaciones (si workorders activos)
- 2-3 operaciones por BoM
- Tiempos estimados realistas
- Asignadas a centros de trabajo

### Productos
- Productos terminados (con BoM asociada)
- Componentes/materias primas (con stock)
- Al menos 1 semielaborado (con su propia BoM)

## Flujo de Negocio - Distribucion por Estado

### Borrador - 15% de MOs
**Crear**: mrp.production con state=draft
**Para la demo**: Ordenes planificadas, pendientes de confirmar

### Confirmada - 20%
**Crear**: mrp.production + action_confirm
**Para la demo**: Material reservado, lista para producir

### En Progreso - 35%
**Crear**: MO confirmada + button_start (si workorders)
**Para la demo**: En fabricacion activa, mostrar progreso

### Hecha - 30%
**Crear**: MO + button_mark_done (con produce wizard)
**Para la demo**: Produccion completada, stock actualizado

## Tips para Demo Impactante
- Tener 1 MO en progreso para completar en vivo durante la demo
- Mostrar BoM multinivel y como se planifican automaticamente las MOs dependientes
- Si hay centros de trabajo, mostrar la vista de planificacion Gantt
- Crear 1 MO con variacion de cantidad (producir mas o menos de lo planificado)
- Tener componentes con stock justo para que al crear MO se vea la reserva parcial
- Si hay QC, tener 1 punto de control para mostrar en demo

## Comandos API (odoo-pilot)

```bash
# Crear lista de materiales
create_record.sh mrp.bom '{"product_tmpl_id": TMPL_ID, "product_qty": 1, "type": "normal", "bom_line_ids": [[0, 0, {"product_id": COMP_ID, "product_qty": 2}]]}'

# Crear orden de fabricacion
create_record.sh mrp.production '{"product_id": PROD_ID, "product_qty": 5, "bom_id": BOM_ID}'

# Confirmar MO
execute_method.sh mrp.production MO_ID action_confirm

# Completar MO
execute_method.sh mrp.production MO_ID button_mark_done
```
