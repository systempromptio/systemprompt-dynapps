# Estructura de Knowledge del Proyecto

Referencia para la creacion de la jerarquia de articulos en `knowledge.article` asociada a un proyecto de implantacion.

---

## Jerarquia de Articulos

```
{Nombre del Proyecto}  ← Articulo raiz (workspace)
├── Kick-Off                      ← Generado por: kick-off-presentation
├── Analisis                      ← Generado por: project-analysis
│   ├── Configuraciones Generales   (creados por project-analysis)
│   ├── CRM
│   ├── Ventas
│   └── ...
├── Cuestionarios de Analisis     ← Generado por: questionnaire-generator
│   ├── Configuraciones Generales   (creados por questionnaire-generator)
│   ├── CRM
│   ├── Ventas
│   └── ...
├── Formaciones                   ← Generado por: training-planner
│   ├── CRM                        (creados por training-planner)
│   ├── Ventas
│   └── ...
├── Actas de Reunion              ← Generado por: project-meeting-minutes
├── Adaptaciones                  ← Generado por: skills de desarrollo
└── Interno                       ← Solo usuarios internos Indaws
    └── (documentos internos)
```

**IMPORTANTE**: Este skill solo crea el articulo raiz y los 7 hijos directos. Los subarticulos (por app, por formacion, etc.) los crean los skills correspondientes cuando se ejecutan.

---

## Definicion de cada Articulo

### Articulo Raiz: {Nombre del Proyecto}

| Campo | Valor |
|-------|-------|
| `name` | Mismo nombre que `project.project.name` |
| `icon` | "📁" |
| `internal_permission` | "write" |
| `category` | "workspace" |
| `body` | HTML con informacion basica del proyecto (ver template abajo) |

**Body del articulo raiz:**
```html
<div style="font-family: Montserrat, Verdana, sans-serif; max-width: 800px;">
  <h2 style="color: #1C265D; margin-bottom: 16px;">{nombre_proyecto}</h2>
  <table style="border-collapse: collapse; width: 100%; margin-bottom: 24px;">
    <tr>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6; font-weight: 600; background: #f8f9fa; width: 200px;">Cliente</td>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6;">{nombre_cliente}</td>
    </tr>
    <tr>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6; font-weight: 600; background: #f8f9fa;">Fecha inicio</td>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6;">{fecha_inicio}</td>
    </tr>
    <tr>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6; font-weight: 600; background: #f8f9fa;">Fecha Go-Live</td>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6;">{fecha_fin}</td>
    </tr>
    <tr>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6; font-weight: 600; background: #f8f9fa;">Aplicaciones</td>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6;">{lista_apps}</td>
    </tr>
  </table>
  <p style="color: #666; font-size: 14px;">Este espacio contiene toda la documentacion del proyecto de implantacion. Cada seccion se actualiza automaticamente conforme avanza el proyecto.</p>
</div>
```

---

### Subarticulos Compartibles (1-6)

Estos articulos heredan permisos del padre y son visibles para el cliente.

#### 1. Kick-Off

| Campo | Valor |
|-------|-------|
| `name` | "Kick-Off" |
| `parent_id` | ID del articulo raiz |
| `icon` | "🚀" |
| `internal_permission` | "write" |
| `body` | "" (se genera con skill kick-off-presentation) |

#### 2. Analisis

| Campo | Valor |
|-------|-------|
| `name` | "Analisis" |
| `parent_id` | ID del articulo raiz |
| `icon` | "🔍" |
| `internal_permission` | "write" |
| `body` | "" (se genera con skill project-analysis) |

#### 3. Cuestionarios de Analisis

| Campo | Valor |
|-------|-------|
| `name` | "Cuestionarios de Analisis" |
| `parent_id` | ID del articulo raiz |
| `icon` | "📋" |
| `internal_permission` | "write" |
| `body` | "" (se genera con skill questionnaire-generator) |

#### 4. Formaciones

| Campo | Valor |
|-------|-------|
| `name` | "Formaciones" |
| `parent_id` | ID del articulo raiz |
| `icon` | "🎓" |
| `internal_permission` | "write" |
| `body` | "" (se genera con skill training-planner) |

#### 5. Actas de Reunion

| Campo | Valor |
|-------|-------|
| `name` | "Actas de Reunion" |
| `parent_id` | ID del articulo raiz |
| `icon` | "📝" |
| `internal_permission` | "write" |
| `body` | "" (se genera con skill project-meeting-minutes) |

#### 6. Adaptaciones

| Campo | Valor |
|-------|-------|
| `name` | "Adaptaciones" |
| `parent_id` | ID del articulo raiz |
| `icon` | "⚙️" |
| `internal_permission` | "write" |
| `body` | "" (se genera con skills de desarrollo) |

---

### Subarticulo Interno (7)

Este articulo es SOLO para usuarios internos de Indaws. El cliente NO debe verlo.

#### 7. Interno

| Campo | Valor |
|-------|-------|
| `name` | "Interno" |
| `parent_id` | ID del articulo raiz |
| `icon` | "🔒" |
| `internal_permission` | "write" |
| `is_desynchronized` | true |
| `body` | "" |

**Clave de seguridad**: `is_desynchronized=true` rompe la herencia de permisos del padre. Aunque el cliente sea miembro del articulo raiz con permiso "read", NO podra ver "Interno" porque este articulo gestiona sus propios permisos de forma independiente.

---

## Compartir con el Cliente (Portal)

Para que el cliente pueda ver los articulos del proyecto desde su portal:

### Paso 1: Identificar el partner del cliente

Usar el `commercial_partner_id` del `partner_id` del proyecto. Esto asegura que se usa la empresa (no un contacto individual).

Si `commercial_partner_id` es `false`, usar directamente `partner_id`.

### Paso 2: Anadir como miembro del articulo raiz

Crear un registro en `knowledge.article.member`:

```json
{
  "article_id": root_article_id,
  "partner_id": commercial_partner_id,
  "permission": "read"
}
```

**IMPORTANTE**: Solo se anade el miembro al articulo RAIZ. Los hijos heredan automaticamente el permiso (excepto "Interno" que tiene `is_desynchronized=true`).

### Paso 3: Verificar

Buscar si el miembro ya existe antes de crearlo:
```
search([('article_id', '=', root_article_id), ('partner_id', '=', commercial_partner_id)])
```

---

## Patron Search-Before-Create

Para cada articulo, seguir SIEMPRE este patron:

```python
# 1. Buscar si ya existe
existing = search([
    ('name', '=', article_name),
    ('parent_id', '=', parent_id)
])

# 2. Si existe, usar el existente
if existing:
    article_id = existing[0]
    # Opcionalmente actualizar body si esta vacio

# 3. Si no existe, crear
else:
    article_id = create({
        'name': article_name,
        'parent_id': parent_id,
        'icon': icon,
        'internal_permission': 'write',
        'body': body_html
    })
```

Este patron garantiza idempotencia: ejecutar el skill multiples veces no duplica articulos.

---

## Vinculacion Proyecto ↔ Knowledge

El campo `x_studio_knowledge` en `project.project` es un Many2one hacia `knowledge.article`. Permite:

- Acceder rapidamente al Knowledge del proyecto desde la vista de proyecto
- Que otros skills lean el articulo raiz sin preguntarlo al usuario

Despues de crear el articulo raiz:
```python
# Actualizar el proyecto con el articulo raiz
write(project_id, {
    'x_studio_knowledge': root_article_id
})
```
