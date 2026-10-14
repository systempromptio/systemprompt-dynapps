# Consultant Assistant - Mode GENERAR

## Workflow

### Paso 1: Escanear skills

```bash
bash scripts/generate_skill_catalog.sh {directorio_base_skills}
```

El script busca todos los SKILL.md, extrae frontmatters y genera un JSON con el catalogo.

### Paso 2: Leer fuentes

Lee en paralelo:
- `references/metodologia_indaws.md`
- `references/skill_catalog.md`
- `references/estandares_proyecto.md`
- `references/knowledge_structure.md`
- `templates/guia_consultoria.html`

### Paso 3: Generar HTML

1. Usa el template como base
2. Rellena los placeholders (`{{DATE}}`, `{{SKILL_COUNT}}`, `{{STATS}}`)
3. Regenera los tabs 1-3 con los datos actuales de skills
4. Genera los tabs 4-6 desde los references
5. Guarda el resultado en `output/guia_consultoria_{fecha}.html`

## HTML Rules

- El HTML debe ser autonomo (CSS inline, sin dependencias externas excepto Google Fonts)
- Usar template `templates/guia_consultoria.html` como base
