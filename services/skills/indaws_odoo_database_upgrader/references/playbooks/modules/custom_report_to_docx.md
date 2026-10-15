# Playbook: Report custom (QWeb PDF) → `report_docx`

Módulo custom con reports PDF complejos en QWeb que en v18 requieren reescritura
significativa (BS5, `address_layout`, `tax_totals`, etc.). Alternativa:
**migrar a `report_docx` / `indaws_report_docx`** (plantillas .docx).

## Requisitos del contexto
- `features.reports_docx: true` (requiere `indaws_report_docx` instalado en destino)
- `to_version`: 17.0+

## Aplicabilidad
- Report QWeb con mucho HTML/CSS que sufre BS5 regression
- Cliente prefiere editar plantillas en Word/LibreOffice
- Hay licencia para `report_docx` o módulo OCA equivalente

## Flujo
1. **Decisión consultor + cliente** (categoría 3)
2. Para cada report:
   a. Generar `.docx` template equivalente (manualmente o con IA)
   b. Crear `ir.actions.report` con `report_type='docx'` y `report_name='<modulo>.<key>_docx'`
   c. Sustituir el `report_action` invocado en menús/botones
3. **Probar PDFs** generados contra muestras de v15
4. **Marcar el report viejo** como `active=false` o eliminarlo del `ir_act_report_xml`

## Helpers a invocar
- Skill externa: `indaws-data-export` puede importar templates si están listos
- Sin step técnico estándar (cada report es ad-hoc)

## Pitfalls
- `report_docx` no soporta inline CSS — usar estilos del template Word
- Numeración de páginas: depende del motor (LibreOffice headless vs `docxtpl + libreoffice`)
- Logo / firma digital: re-incrustar en .docx
