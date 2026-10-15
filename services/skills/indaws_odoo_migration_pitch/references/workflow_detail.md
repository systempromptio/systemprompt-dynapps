# Workflow Detail — Odoo Migration Pitch

Detailed steps, queries, HTML snippets, placeholder lists, validation templates, file structure, and success criteria.

---

## Paso 0: Recopilacion de Informacion

PREGUNTAR al usuario los siguientes datos:

1. **Nombre del cliente** (obligatorio)
2. **Version actual de Odoo** (obligatorio): v14, v15, v16, v17, v18
3. **URL del repositorio GitHub** del cliente (obligatorio): Donde estan los modulos a medida
4. **Credenciales de Odoo del cliente** (obligatorio):
   - URL de la instancia (ej: `https://cliente.odoo.com`)
   - Base de datos
   - API Key
   - Usuario (login)
   - **Confirmar EXPLICITAMENTE que el acceso es SOLO LECTURA**
5. **Coste anual de licencias en EUR** (opcional): Para calcular impacto del sobrecoste del 25%
   - Si no se proporciona, mostrar formula generica sin cifra concreta
6. **Version destino** (opcional, default: v19)
7. **Datos del consultor** (opcional):
   - Nombre (default: `Equipo Comercial Indaws`)
   - Email (default: `comercial@indaws.es`)
   - Telefono (default: `+34 960 069 072`)

NO preguntar sobre tamano del proyecto, tipo de migracion, analisis profundo ni otros parametros tecnicos. Este es un skill COMERCIAL.

Mostrar mensaje de activacion con los datos recopilados.

> **DETENTE** aqui y espera confirmacion del usuario antes de continuar.

---

## Paso 1: Preparacion del Entorno de Analisis

Ejecutar en paralelo cuando sea posible:

### 1.1 Clonar repositorios Indaws Core

Ejecutar `scripts/clone_indaws_repos.sh [directorio] [rama]` que clona a un directorio temporal los siguientes repos via SSH. Por defecto clona la rama principal (version en produccion de Indaws). Si el usuario solicita explicitamente una version concreta de Indaws Core (ej: 17.0, 18.0), pasar la rama como segundo argumento:

```bash
REPOS=(
  "git@github.com:indawsodoo/indaws-odoo.git"                  # CE
  "git@github.com:indawsodoo/indaws-odoo-enterprise.git"        # EE
  "git@github.com:indawsodoo/vertical-odoo-inbuilt.git"         # Obras/Instaladores
  "git@github.com:indawsodoo/connector-sale-odoo.git"           # eCommerce/Marketplaces
  "git@github.com:indawsodoo/indaws-edicom.git"                 # EDI
  "git@github.com:indawsodoo/connector-stock-odoo.git"          # Stock connectors
)

# Rama por defecto (version en produccion):
bash scripts/clone_indaws_repos.sh /tmp/indaws-core-repos

# O con rama especifica si el usuario lo pide:
bash scripts/clone_indaws_repos.sh /tmp/indaws-core-repos 18.0
```

Si el script no existe, ejecutar los comandos directamente.

### 1.2 Clonar repositorio del cliente

```bash
git clone --depth 1 "$CLIENT_REPO_URL" "$TMPDIR/client_modules"
```

**Distribucion de agentes sugerida**:
- Agent 1: Clonar repos (Paso 1)
- Agent 2: Auditoria Odoo (Paso 2) — puede iniciar en paralelo con Agent 1

---

## Paso 2: Auditoria Odoo (SOLO LECTURA via odoo-pilot)

> **CRITICO**: Todas las operaciones son `search_read`. NUNCA usar `create`, `write`, `unlink` ni ningun metodo que modifique datos.

### 2.1 Modulos Instalados

```
Modelo: ir.module.module
Filtro: [["state", "=", "installed"]]
Campos: ["name", "shortdesc", "author", "website"]
```

Clasificar cada modulo por `author`:

| Categoria | Regla de Deteccion |
|-----------|-------------------|
| **Odoo Standard** | `author` contiene "Odoo" |
| **OCA** | `author` contiene "Odoo Community Association" o `website` contiene "github.com/OCA" |
| **Indaws Core** | `name` empieza por "indaws_" o `author` contiene "Indaws" o "inDAWS" |
| **Custom / Terceros** | Todo lo demas |

### 2.2 Personalizaciones Studio

```
# Campos personalizados
Modelo: ir.model.fields
Filtro: [["state", "=", "manual"]]
Campos: ["name", "model_id", "ttype", "field_description"]

# Vistas modificadas por Studio
Modelo: ir.ui.view
Filtro: ["|", ["key", "like", "studio"], ["arch_db", "like", "x_studio"]]
Campos: ["name", "model", "type", "key"]
```

Volumen Studio:
- **Bajo**: <30 campos, <15 vistas
- **Medio**: 30-80 campos, 15-40 vistas
- **Alto**: >80 campos, >40 vistas

### 2.3 Automatizaciones e Informes

```
# Acciones automatizadas
Modelo: base.automation | Filtro: [] | Campos: ["name", "model_id"]

# Acciones planificadas activas
Modelo: ir.cron | Filtro: [["active", "=", true]] | Campos: ["name", "model_id"]

# Informes custom (excluir estandar post-query)
Modelo: ir.actions.report
Filtro: [["report_type", "in", ["qweb-pdf", "qweb-html"]]]
Campos: ["name", "report_name"]
```

### 2.4 Usuarios Activos

```
Modelo: res.users
Filtro: [["active", "=", true], ["share", "=", false]]
Campos: ["name"]
```

Determinar tier de infraestructura segun `user_count`:
- 0-100: Pequeno (2 vCPU, 7 GB RAM, 60 GB SSD, 4 workers)
- 101-200: Mediano (4 vCPU, 12 GB RAM, 100 GB SSD, 8 workers)
- 201-300: Grande (6 vCPU, 20 GB RAM, 120 GB SSD, 12 workers)

### 2.5 Complexity Score

```
raw = (custom_modules * 15 + oca_sin_version * 20 + studio_fields * 0.3 +
       studio_views * 0.5 + automated_actions * 2 + custom_reports * 5 +
       versions_to_skip * 10)
score = min(round(raw / 200 * 100), 100)
```

| Score | Clasificacion |
|-------|--------------|
| 0-25 | Baja |
| 26-50 | Media |
| 51-75 | Alta |
| 76-100 | Muy Alta |

### VALIDACION 1

PRESENTAR al usuario:

```
=== AUDITORIA ODOO: {customer_name} ===
Version actual: {current_version} → Destino: {target_version}
Usuarios internos: {user_count} → Tier recomendado: {tier}

MODULOS INSTALADOS ({total}):
  Odoo Standard: {count} (migracion automatica)
  OCA:           {count} (requieren evaluacion)
  Indaws Core:   {count} (migrados por Indaws)
  Custom:        {count} (requieren plan de accion)

PERSONALIZACIONES STUDIO:
  Campos custom:      {studio_fields_count}
  Vistas modificadas: {studio_views_count}
  Volumen: {bajo/medio/alto}

AUTOMATIZACIONES:
  Acciones automatizadas: {count}
  Acciones planificadas:  {count}
  Informes custom:        {count}

COMPLEXITY SCORE: {score}/100 ({clasificacion})

¿Los datos son correctos? ¿Algun ajuste?
```

> **DETENTE** aqui y espera la aprobacion del usuario antes de continuar.

---

## Paso 3: Analisis de Codigo y Clasificacion de Modulos

Este es el paso CLAVE: en lugar de delegar a `/odoo-code-auditor`, analizar directamente el codigo clonado.

**Distribucion de agentes**: Agent 3 ejecuta este paso una vez que los repos del Paso 1 estan disponibles.

### 3.1 Escaneo de modulos del cliente

Localizar todos los directorios con `__manifest__.py` en el repo clonado del cliente. Para cada modulo, analizar:
- Modelos definidos (Python: `_name`, `_inherit`)
- Campos (Python: `fields.`)
- Vistas (XML)
- Logica de negocio (metodos Python, JS)

### 3.2 Clasificacion con prioridad estricta

Para cada modulo **Custom** y **OCA**, aplicar en este ORDEN EXACTO:

1. **Consultar repos Indaws Core clonados**: Buscar modulos con funcionalidad similar (por patrones de nombre, nombres de modelo, nombres de campo). Si hay coincidencia → **REEMPLAZAR POR INDAWS CORE** (indicar que modulo(s) Core lo cubren)

2. **Evaluar si es candidato a Studio**: Si el modulo tiene <50 lineas de logica Python (campos, computed fields, automatizaciones simples) → **RECONSTRUIR EN STUDIO**

3. **Consultar release notes de version destino**: Leer `references/odoo-versions/ODOO_{TARGET}_RELEASE_NOTES.md`. Si la funcionalidad es ahora nativa → **ELIMINAR (ahora estandar)**

4. **Evaluar obsolescencia**: Analizar si el modulo es obsoleto o no se usa → **ELIMINAR**

5. **Si ninguna de las anteriores aplica** → **MIGRAR**

### 3.3 Reglas de Oro (OBLIGATORIAS, anulan clasificacion normal)

- **Import/Export** → SIEMPRE Indaws Core (`indaws_data_import` / `indaws_data_export`)
- **Dashboards/KPIs** → SIEMPRE Standard con Spreadsheets

### 3.4 Tabla de acciones por modulo

Agrupar modulos por accion con headers de color:

| Grupo | Header | Contenido |
|-------|--------|-----------|
| **ELIMINAR** | Verde | Modulos cubiertos por estandar u obsoletos |
| **REEMPLAZAR** | Azul | Modulos cubiertos por Indaws Core o Studio |
| **EVALUAR** | Amarillo | Modulos OCA que requieren evaluacion |
| **MIGRAR** | Lila | Modulos verdaderamente custom que deben migrarse |

Cada fila: **Modulo** | **Tipo** (OCA/Custom/Terceros) | **Razon** (lenguaje de NEGOCIO, no tecnico) | **Complejidad** (Baja/Media/Alta)

**IMPORTANTE**: La tabla es COMERCIAL. No mostrar horas, LOC ni APIs deprecadas.

### VALIDACION 2

PRESENTAR al usuario:

```
=== PLAN DE ACCION DE MODULOS ===

| Modulo | Tipo | Accion | Razon | Complejidad |
|--------|------|--------|-------|-------------|
| ... | ... | ... | ... | ... |

RESUMEN:
  Eliminar:    {count} modulos
  Reemplazar:  {count} modulos (Core + Studio)
  Evaluar:     {count} modulos (OCA)
  Migrar:      {count} modulos

¿Apruebas el plan? ¿Algun cambio en las acciones recomendadas?
```

> **DETENTE** aqui y espera la aprobacion del usuario antes de generar el PDF.

---

## Paso 4: Generacion del PDF

> **SOLO** ejecutar tras las 2 validaciones aprobadas.

### 4.1 Cargar referencias

LEER los siguientes archivos para componer el contenido del PDF:

| Archivo | Cuando leer | Para que |
|---------|-------------|---------|
| `references/sales_arguments.md` | Paso 4 | Secciones 3 (Alerta 25%), 4 (Deuda Tecnica), 6 (ROI) |
| `references/version_improvements.md` | Paso 4 | Tarjetas de mejoras por app (solo apps instaladas) |
| `references/indaws_ecosystem.md` | Paso 4 | Secciones 7 (Ecosistema), 8 (Cloud), 9 (Metodologia) |
| `references/reglas_timeline.md` | Paso 4.1b | Calculo dinamico de semanas por fase |
| `references/module_classification.md` | Paso 3 | Reglas de clasificacion de modulos |
| `references/odoo-versions/ODOO_{VERSION}_RELEASE_NOTES.md` | Paso 3 | Funcionalidades nativas de la version destino |
| `templates/migration_pitch.html` | Paso 4 | Template HTML base |

### 4.1b Calcular timeline dinamico

LEER `references/reglas_timeline.md` y calcular las semanas de cada fase usando los datos del Paso 3:

1. Estimar horas por modulo segun accion (MIGRAR/REEMPLAZAR/EVALUAR/ELIMINAR) y complejidad
2. Sumar horas de personalizaciones Studio, automatizaciones e informes
3. Anadir tareas genericas (setup 12h + soporte + reuniones)
4. Convertir a semanas (capacidad 25h/semana, o 35h si >500h)
5. Distribuir en 3 fases: 50% / 35% / 15% con minimos (4 + 3 + 2)

Si no se pudo clonar el repo del cliente o no hay datos suficientes, usar valores por defecto: 8 + 6 + 2 = 16 semanas.

**IMPORTANTE**: Las horas son un calculo INTERNO. El PDF solo muestra semanas por fase.

### 4.2 Preparar placeholders

Reemplazar los 60+ placeholders `{{...}}` del template. Los principales:

**Datos basicos**: `{{customer_name}}`, `{{current_version}}`, `{{target_version}}`, `{{versions_to_skip}}`

**Conteos de modulos**: `{{total_modules}}`, `{{count_standard}}`, `{{count_oca}}`, `{{count_core}}`, `{{count_custom}}`, `{{custom_modules_count}}`, `{{oca_modules_count}}`, `{{modules_to_migrate}}`, `{{pct_standard}}`, `{{pct_core}}`, `{{pct_oca}}`, `{{pct_custom}}`

**Studio**: `{{studio_fields_count}}`, `{{studio_views_count}}`, `{{studio_fields_pct}}`, `{{studio_views_pct}}`, `{{studio_models_count}}`, `{{total_studio_customizations}}`, `{{studio_volume_label}}`, `{{studio_volume_badge}}`

**Automatizaciones**: `{{automated_actions_count}}`, `{{scheduled_actions_count}}`, `{{custom_reports_count}}`

**Usuarios e infra**: `{{user_count}}`, `{{user_capacity_pct}}`, `{{infra_tier_recommendation}}`, `{{recommended_plan_name}}`, `{{recommended_plan_users}}`, `{{recommended_plan_cpu}}`, `{{recommended_plan_ram}}`, `{{recommended_plan_ssd}}`, `{{recommended_plan_workers}}`

**Complexity Score**: `{{complexity_score}}`, `{{score_label}}`, `{{score_class}}`, `{{score_badge_class}}`

**Sobrecoste licencias**: `{{annual_license_cost}}`, `{{license_surcharge}}`, `{{total_with_surcharge}}`, `{{surcharge_3_years}}`, `{{surcharge_status}}`

**Timeline (calculado en 4.1b)**: `{{phase_1_weeks}}`, `{{phase_2_weeks}}`, `{{phase_3_weeks}}`, `{{total_weeks}}`, `{{total_months}}`, `{{phase_1_end_plus_1}}`, `{{phase_2_end}}`, `{{phase_2_end_plus_1}}`
  - `{{phase_1_end_plus_1}}` = phase_1_weeks + 1
  - `{{phase_2_end}}` = phase_1_weeks + phase_2_weeks
  - `{{phase_2_end_plus_1}}` = phase_2_end + 1

**Contenido dinamico**: `{{modules_action_grouped}}`, `{{app_improvements_cards}}`, `{{indaws_core_modules_list}}`

**Consultor**: `{{consultant_name}}`, `{{consultant_email}}`, `{{consultant_phone}}`

### 4.3 Generar modulos agrupados (HTML)

Generar `{{modules_action_grouped}}` como bloques HTML agrupados por accion:

```html
<div class="module-group">
  <div class="module-group-header module-group-header-eliminate">
    <svg>...</svg> ELIMINAR <span class="module-group-count">{count}</span>
  </div>
  <table class="data-table">
    <tbody>
      <tr><td>{modulo}</td><td>{tipo}</td><td>{razon}</td><td>{complejidad}</td></tr>
    </tbody>
  </table>
</div>
```

Headers: `-eliminate` (rojo), `-replace` (verde), `-evaluate` (amarillo), `-migrate` (lila).

**PAGINACION**: La pagina de Mapa de Modulos usa la clase CSS `page-flow` que permite que el contenido fluya naturalmente a multiples paginas A4 sin recortarse. NO es necesario dividir manualmente la tabla — Playwright generara los saltos de pagina automaticamente. Si se desea, se puede repetir el header "Mapa de Modulos (continuacion)" creando un nuevo `<div class="page page-flow bg-white">`.

**IMPORTANTE sobre clases CSS de pagina**:
- `page-fixed`: Para paginas con altura exacta A4 (portada, proximos pasos). Contenido se ajusta a 297mm.
- `page-flow`: Para paginas con contenido dinamico (mapa de modulos). Altura automatica, el contenido fluye a paginas adicionales.
- Sin clase adicional: `min-height: 297mm` con overflow visible. El contenido se muestra completo.

Consolidar modulos relacionados en una misma fila cuando tengan la misma accion.

### 4.4 Generar tarjetas de mejoras por app

CONSULTAR `references/version_improvements.md`. Generar tarjetas HTML SOLO para apps que el cliente tiene instaladas:

```html
<div class="glass-card">
  <div class="app-card-header">
    <div class="app-icon"><svg><!-- icono SVG inline --></svg></div>
    <h4>{app_name}</h4>
  </div>
  <ul>
    <li>{mejora_1}</li>
    <li>{mejora_2}</li>
  </ul>
</div>
```

Iconos SVG inline estilo feather (stroke, 16x16). Apps de upsell (no instaladas) usan fondo verde en `.app-icon`.

### 4.5 Generar HTML y convertir a PDF

1. INVOCAR `/dynapps-brand` para directrices de marca actualizadas
2. LEER `templates/migration_pitch.html`
3. Reemplazar TODOS los placeholders con datos reales
4. GUARDAR como `{customer_name}_migration_pitch.html`
5. Convertir a PDF via Playwright:
   - Format: A4
   - printBackground: true
   - Margins: 0 en todos los lados
6. GUARDAR como `{customer_name}_migration_pitch.pdf`

**OBLIGATORIO**: Usar Playwright para la conversion. Nunca otro metodo.

### VALIDACION 3

CONFIRMAR al usuario:

```
PDF generado exitosamente:
  Archivo: {customer_name}_migration_pitch.pdf
  Paginas: {N}

Secciones incluidas:
  1. Portada
  2. Diagnostico Express (dashboard)
  3. Alerta: Incremento del 25%
  4. Deuda Tecnica
  5. Mapa de Modulos
  6. Que Ganas Migrando ({N} apps con mejoras)
  7. Ecosistema de Valor Indaws
  8. Indaws Cloud (infraestructura)
  9. Metodologia de Migracion
  10. Proximos Pasos

¿Deseas algun ajuste antes de compartirlo con el cliente?
```

---

## Estructura de Archivos

```
odoo-migration-pitch/
├── SKILL.md                                    # Skill definition (compact)
├── references/
│   ├── workflow_detail.md                     # THIS FILE — full step detail
│   ├── sales_arguments.md                     # Argumentos de venta (25%, deuda tecnica, seguridad)
│   ├── indaws_ecosystem.md                    # Indaws Cloud + Core + metodologia
│   ├── module_classification.md               # Clasificacion de modulos y reglas
│   ├── reglas_timeline.md                     # Calculo dinamico de semanas por fase
│   ├── version_improvements.md                # Mejoras por version y por app
│   └── odoo-versions/
│       └── ODOO_{VERSION}_RELEASE_NOTES.md    # Release notes por version destino
├── scripts/
│   └── clone_indaws_repos.sh                  # Clonado automatico de repos Core
└── templates/
    └── migration_pitch.html                   # Template HTML (bento + glassmorphism)
```

---

## Success Criteria

- [ ] Informacion del cliente recopilada
- [ ] Repos Indaws Core clonados
- [ ] Auditoria Odoo completada (read-only)
- [ ] Modulos correctamente clasificados con analisis real de codigo
- [ ] Validacion 1 aprobada
- [ ] Tabla de acciones por modulo definida con consulta a Core
- [ ] Complejidad calculada
- [ ] Validacion 2 aprobada
- [ ] HTML generado con todos los placeholders reemplazados
- [ ] Cards de mejoras generados (solo apps instaladas)
- [ ] PDF generado via Playwright (10+ paginas, marca Indaws)
- [ ] Seccion sobrecoste 25% incluida
- [ ] NO incluye pricing/estimaciones de horas
- [ ] Archivo guardado como `{customer_name}_migration_pitch.pdf`
