# Guia de Distribucion de Datos por Estado para Demos

Este documento define como distribuir datos transaccionales en una demo de Odoo para que sea realista e impactante.

---

## Principio Fundamental

Una demo necesita ver **TODOS los estados del flujo**, no solo datos creados. El objetivo es que el cliente vea el sistema "en uso", como si llevara semanas funcionando.

---

## Regla 80/20

- **80%** de datos en estados **intermedios** (donde el usuario interactuara durante la demo)
- **20%** en estados **finales** (para demostrar que el flujo se completa)

---

## Distribucion Temporal

- **40%** historico (2-6 meses atras): demuestra que el sistema tiene vida
- **40%** actual (ultimo mes): datos con los que interactuar
- **20%** futuro/planificado: demuestra planificacion y prevision

---

## Distribucion por Tipo de Documento

### Ventas (sale.order)
| Estado | % | Descripcion | API |
|--------|---|-------------|-----|
| Presupuesto (draft) | 30% | Pendientes de enviar/aprobar | Crear con state=draft |
| Pedido confirmado | 25% | Confirmados, pendiente de entregar | action_confirm |
| Parcialmente entregado | 15% | En proceso de entrega | Confirmar + validar picking parcial |
| Totalmente entregado | 10% | Listos para facturar | Confirmar + validar picking completo |
| Facturado | 10% | Con factura emitida | _create_invoices + action_post |
| Pagado | 10% | Flujo completo | Registrar pago en factura |

### Compras (purchase.order)
| Estado | % | Descripcion | API |
|--------|---|-------------|-----|
| Solicitud de presupuesto (draft) | 20% | Pendientes de enviar | Crear con state=draft |
| PO confirmada | 30% | Pedidas al proveedor | button_confirm |
| Parcialmente recibida | 15% | En recepcion | Confirmar + validar picking parcial |
| Totalmente recibida | 20% | Pendiente de factura | Confirmar + validar picking completo |
| Facturada y pagada | 15% | Flujo completo | Crear factura + pago |

### Inventario (stock.picking)
| Estado | % | Descripcion | API |
|--------|---|-------------|-----|
| Borrador | 15% | Planificados | Crear con state=draft |
| En espera | 20% | Esperando disponibilidad | action_confirm |
| Preparado | 30% | Listo para transferir | action_assign |
| Hecho | 35% | Completados | button_validate |

### Contabilidad (account.move)
| Estado | % | Descripcion | API |
|--------|---|-------------|-----|
| Borrador | 20% | Pendientes de validar | Crear con state=draft |
| Publicada | 40% | Validadas, pendiente de pago | action_post |
| Pagada parcialmente | 15% | En proceso de cobro | Registrar pago parcial |
| Pagada | 25% | Cobradas/pagadas | Registrar pago total |

### CRM (crm.lead)
| Estado | % | Descripcion | API |
|--------|---|-------------|-----|
| Nuevo | 15% | Leads recien llegados | Crear en stage_id primer stage |
| Calificado | 20% | Contacto realizado | Mover a stage calificacion |
| Propuesta | 25% | Presupuesto enviado | Mover a stage propuesta |
| Negociacion | 20% | En negociacion activa | Mover a stage negociacion |
| Ganado | 15% | Cerrados con exito | action_set_won |
| Perdido | 5% | Para estadisticas | action_set_lost |

### Fabricacion (mrp.production)
| Estado | % | Descripcion | API |
|--------|---|-------------|-----|
| Borrador | 15% | Planificadas | Crear con state=draft |
| Confirmada | 20% | Material reservado | action_confirm |
| En progreso | 35% | En fabricacion | button_start (si workorders) |
| Hecha | 30% | Completadas | button_mark_done |

### Proyecto (project.task)
| Estado | % | Descripcion | API |
|--------|---|-------------|-----|
| Nuevo | 15% | Recien creadas | Crear en stage_id primer stage |
| En progreso | 40% | Trabajando activamente | Mover a stage en progreso |
| En revision | 15% | Pendiente de validacion | Mover a stage revision |
| Hecho | 25% | Completadas | Mover a stage done |
| Cancelado | 5% | Para contexto | Mover a stage cancelado |

### RRHH - Ausencias (hr.leave)
| Estado | % | Descripcion | API |
|--------|---|-------------|-----|
| Borrador | 20% | Pendientes de solicitar | Crear con state=draft |
| Confirmada | 30% | Solicitadas | action_confirm |
| Validada | 40% | Aprobadas | action_validate |
| Rechazada | 10% | Para ver el flujo completo | action_refuse |

### Helpdesk (helpdesk.ticket)
| Estado | % | Descripcion | API |
|--------|---|-------------|-----|
| Nuevo | 20% | Recien recibidos | Crear en stage_id primer stage |
| En progreso | 35% | Siendo atendidos | Mover a stage en progreso |
| En espera cliente | 15% | Pendiente de respuesta | Mover a stage esperando |
| Resuelto | 25% | Solucionados | Mover a stage resuelto |
| Cerrado | 5% | Archivados | Mover a stage cerrado |

### Punto de Venta (pos.order)
| Estado | % | Descripcion | API |
|--------|---|-------------|-----|
| Pagado | 90% | Ventas completadas | Crear via sesion POS |
| Facturado | 10% | Con factura asociada | Crear con factura |

### Suscripciones (sale.order con recurrence)
| Estado | % | Descripcion | API |
|--------|---|-------------|-----|
| Borrador | 10% | Pendientes de activar | Crear con state=draft |
| En progreso | 50% | Activas | action_confirm |
| Pausada | 10% | Temporalmente detenida | set_paused |
| Cerrada | 20% | Finalizadas normalmente | set_close |
| Cancelada | 10% | Canceladas | set_cancel |

---

## Datos Maestros Recomendados

### Contactos
- **Clientes**: 8-15 (mezcla B2B y B2C segun sector)
- **Proveedores**: 3-6 (del sector relevante)
- Al menos 2 clientes con historial completo (pedidos, facturas, pagos)
- Al menos 1 cliente nuevo (sin historial, para crear en demo)

### Productos
- **Productos fisicos**: 10-20 (con variantes si aplica)
- **Servicios**: 3-5 (si aplica al sector)
- Al menos 2 productos con stock disponible
- Al menos 1 producto con regla de reabastecimiento
- Categorias bien organizadas (3-5 categorias)
- Imagenes en los productos principales (si hay Website)

### Empleados (si RRHH activo)
- 5-10 empleados en 2-3 departamentos
- Al menos 1 manager por departamento
- Algunos con ausencias registradas

---

## Historico Recomendado

Para que la demo sea creible, crear:

1. **Ultimos 3-6 meses**: 2-3 pedidos de venta completados por mes
2. **Ultimo mes**: mayor actividad (5-8 documentos en varios estados)
3. **Proximas semanas**: 2-3 documentos planificados/programados

### Fechas
- Usar dates relativas a la fecha actual
- No usar fechas fijas (se quedan obsoletas rapidamente)
- Crear registros con create_date en el pasado usando write directo

---

## Tips para Demo Impactante

1. **Variedad de importes**: No todos los presupuestos del mismo valor. Usar rangos realistas del sector.
2. **Nombres reales**: Usar nombres de empresas y productos del sector, no "Cliente 1" o "Producto A".
3. **Actividades programadas**: Crear actividades pendientes para mostrar el planificador.
4. **Adjuntos**: Adjuntar algun PDF de ejemplo a pedidos clave (simula documentos reales).
5. **Notas internas**: Anadir notas en el chatter de 2-3 registros (simula uso real).
6. **Dashboard**: Asegurar que los dashboards de cada app tengan datos suficientes para ser visuales.
7. **Filtros y agrupaciones**: Tener datos variados para que los filtros muestren resultados interesantes.
