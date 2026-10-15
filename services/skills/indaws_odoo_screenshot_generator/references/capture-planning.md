# Capture Planning — Document Analysis

## Detección de Placeholders

Buscar marcadores `[SCREENSHOT:identifier]` en el documento.

**Si hay placeholders**: extraer identificadores, nombrar archivos como `{N}_{identifier}.png`, reemplazar al final.

**Si NO hay placeholders**: análisis manual para identificar capturas.

## Caso A: Documentación de Módulo con Placeholders

Secciones clave a analizar:

1. **"5. Objetos de Negocio"** — modelos principales, nuevos vs extendidos
2. **"6. Cambios en UI"** — vistas, smart buttons, ribbons
3. **"3. Flujo de Trabajo"** — estados/fases a mostrar

## Output: Lista de Capturas

```json
{
  "module_name": "indaws_crm_prospection",
  "captures": [
    {
      "id": "capture_01",
      "model": "crm.prospection",
      "view_type": "form",
      "description": "Vista principal en estado Active",
      "record_filter": "__test__campana_q1_industrial",
      "highlight_selectors": "[name=prospection_count],[name=state]"
    }
  ]
}
```

## Estrategia por Modelo

- **Vista Form**: 1-2 capturas (diferentes estados si aplica)
- **Vista List**: 1 captura con varios registros
- **Vista Kanban**: 1 captura (si existe)
- **Vistas específicas**: según menciones en documentación

## Criterio de Resaltado

Resaltar campos NUEVOS o MODIFICADOS por el módulo documentado. No resaltar campos estándar de Odoo.
