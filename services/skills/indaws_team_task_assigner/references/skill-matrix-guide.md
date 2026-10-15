# Skill Matrix

Gestiona competencias del equipo para mejorar asignaciones.

## Setup inicial

```bash
# desde el directorio de esta skill

# 1. Sincronizar empleados desde Odoo
python3 scripts/skill_matrix.py sync

# 2. Asignar skills (nivel 1-5)
python3 scripts/skill_matrix.py set <emp_id> python 4
python3 scripts/skill_matrix.py set <emp_id> javascript 2
python3 scripts/skill_matrix.py set <emp_id> integrations 5

# 3. Ver matriz
python3 scripts/skill_matrix.py show

# 4. Informe completo (gaps, bus factor)
python3 scripts/skill_matrix.py report

# 5. Buscar candidato por skills requeridos
python3 scripts/skill_matrix.py match '{"python": 3, "integrations": 4}'
```

**Niveles**: 1=Basico, 2=Intermedio, 3=Avanzado, 4=Experto, 5=Referente

**Taxonomia**: python, javascript, xml_views, odoo_core, integrations, devops,
database, testing, functional, frontend, indaws_core, studio, accounting,
stock, manufacturing, ecommerce, hr, project

**Integracion en snapshot.py**: El algoritmo de asignacion (Phase 6) infiere
skills requeridos del nombre de la tarea y cruza con la matriz.
Si la matriz esta vacia, usa score neutral (50) como antes.
