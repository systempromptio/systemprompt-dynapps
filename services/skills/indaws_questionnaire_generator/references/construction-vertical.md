# Construction Vertical (inBuilt) -- Rules

## Detection

Analyze sale order data looking for construction indicators:

1. **`x_studio_conceptos_proyecto`**: "inBuilt", "construccion", "obra", "certificacion", "BC3", "FIEBDC", "partidas", "mediciones".
2. **`x_studio_toma_de_datos`**: constructora, subcontrata, obra civil, presupuestos de obra, certificaciones, partidas, capitulos.
3. **`x_studio_resumen`**: sector de construccion.

## Flag Behavior

When `es_construccion = true`:

| Template | Effect |
|----------|--------|
| `ventas.md` | Include section "5. Presupuestos de Obra" (between `<!-- VERTICAL: CONSTRUCCION -->` and `<!-- FIN VERTICAL: CONSTRUCCION -->`) |
| `compras.md` | Include section "9. Compras de Obra" |
| `facturacion.md` | Include section "7. Certificaciones de Obra" |
| `proyectos-obra.md` | Generate INSTEAD of `planificacion.md`. If "Planificacion" is in services_ids, use this template. If NOT in services_ids but is construction, generate anyway. |
| `planificacion.md` | NEVER generate if construction |

When `es_construccion = false`:
- Remove all content between `<!-- VERTICAL: CONSTRUCCION -->` and `<!-- FIN VERTICAL: CONSTRUCCION -->` in every template.
- Never generate `proyectos-obra.md`.

## Rules

- ALWAYS execute detection (Paso 1c) BEFORE generating questionnaires.
- If construction detected but uncertain, ASK the user.
- If construction and no Planificacion/Proyectos in services_ids, generate Proyectos y Obras anyway -- it is essential for inBuilt.
