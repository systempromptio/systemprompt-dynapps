# Estructura de Knowledge — Proyecto de Migracion

Knowledge tree especifico de proyectos de migracion. Difiere del que crea `project-odoo-creator` (que es para implantaciones).

---

## Jerarquia de articulos

```
{Nombre del Proyecto}                ← Articulo raiz (workspace)
├── Analisis Tecnico                 ← Inventario, decisiones de estrategia, classification
├── Scripts BD                       ← Pre-migrate, post-migrate, log de iteraciones
├── Ensayo                           ← Rehearsal: tiempos medidos, plan de rollback
├── Go-Live                          ← Planning, comunicaciones cliente, retro post-Go-Live
├── Retrospectiva                    ← Lecciones aprendidas, mejoras para futuras migraciones
└── Interno                          ← Notas internas Indaws (NO visibles al cliente)
```

**Solo 6 hijos**, no 7 como en implantacion. No hay "Kick-Off", "Cuestionarios", "Formaciones" ni "Adaptaciones" porque no aplican a migracion.

---

## Definicion de cada articulo

### Articulo raiz: {Nombre del Proyecto}

| Campo | Valor |
|-------|-------|
| `name` | Mismo que `project.project.name` |
| `icon` | "🔄" |
| `internal_permission` | "write" |
| `category` | "workspace" |
| `body` | HTML con datos clave de la migracion (ver template abajo) |

**Body del articulo raiz**:

```html
<div style="font-family: Montserrat, Verdana, sans-serif; max-width: 800px;">
  <h2 style="color: #1C265D; margin-bottom: 16px;">{nombre_proyecto}</h2>
  <table style="border-collapse: collapse; width: 100%; margin-bottom: 24px;">
    <tr>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6; font-weight: 600; background: #f8f9fa; width: 200px;">Cliente</td>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6;">{nombre_cliente}</td>
    </tr>
    <tr>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6; font-weight: 600; background: #f8f9fa;">Migracion</td>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6;">v{from_version} → v{to_version}</td>
    </tr>
    <tr>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6; font-weight: 600; background: #f8f9fa;">Estrategia</td>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6;">{strategy}</td>
    </tr>
    <tr>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6; font-weight: 600; background: #f8f9fa;">Hosting destino</td>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6;">{hosting_type}</td>
    </tr>
    <tr>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6; font-weight: 600; background: #f8f9fa;">Fecha inicio tecnico</td>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6;">{date_start}</td>
    </tr>
    <tr>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6; font-weight: 600; background: #f8f9fa;">Fecha Go-Live</td>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6;">{date_golive}</td>
    </tr>
    <tr>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6; font-weight: 600; background: #f8f9fa;">Modulos custom</td>
      <td style="padding: 8px 16px; border: 1px solid #dee2e6;">{n_modulos} ({total_horas}h estimadas)</td>
    </tr>
  </table>
  <p style="color: #666; font-size: 14px;">Espacio de documentacion del proyecto de migracion segun Articulo 2602. Cada subcarpeta cubre una fase del proceso.</p>
</div>
```

---

### Subarticulos compartibles (5)

Heredan permisos del raiz → visibles para el cliente con permiso "read".

#### 1. Analisis Tecnico

| Campo | Valor |
|-------|-------|
| `name` | "Analisis Tecnico" |
| `parent_id` | raiz |
| `icon` | "🔍" |
| `internal_permission` | "write" |
| `body` | Placeholder con secciones: Inventario de modulos / Decisiones de estrategia / Integraciones externas / Flujos criticos del cliente. Lo rellena el Dev/TL durante Fase 1. |

#### 2. Scripts BD

| Campo | Valor |
|-------|-------|
| `name` | "Scripts BD" |
| `parent_id` | raiz |
| `icon` | "💾" |
| `internal_permission` | "write" |
| `body` | Placeholder con secciones: Pre-migrate scripts / Post-migrate scripts / Log de iteraciones (tabla por iteracion). Lo rellena el Dev en Fase 3. |

#### 3. Ensayo

| Campo | Valor |
|-------|-------|
| `name` | "Ensayo" |
| `parent_id` | raiz |
| `icon` | "🎬" |
| `internal_permission` | "write" |
| `body` | Placeholder con secciones: Tiempos medidos / Incidencias del ensayo / Plan de rollback. Lo rellena el Dev en Fase 5. |

#### 4. Go-Live

| Campo | Valor |
|-------|-------|
| `name` | "Go-Live" |
| `parent_id` | raiz |
| `icon` | "🚀" |
| `internal_permission` | "write" |
| `body` | Placeholder con secciones: Planning detallado / Comunicaciones al cliente / Resumen post-Go-Live (que paso, tiempo total, incidencias). Lo rellena el PM en Fase 6. |

#### 5. Retrospectiva

| Campo | Valor |
|-------|-------|
| `name` | "Retrospectiva" |
| `parent_id` | raiz |
| `icon` | "📚" |
| `internal_permission` | "write" |
| `body` | Placeholder con secciones: Que funciono bien / Que no funciono / Lecciones aprendidas / Mejoras para futuras migraciones. Lo rellena el equipo en Gate 7. |

---

### Subarticulo interno (1)

#### 6. Interno

| Campo | Valor |
|-------|-------|
| `name` | "Interno" |
| `parent_id` | raiz |
| `icon` | "🔒" |
| `internal_permission` | "write" |
| `is_desynchronized` | **true** (CLAVE) |
| `body` | "" |

**Clave de seguridad**: `is_desynchronized=true` rompe la herencia de permisos del padre. El cliente, aunque sea miembro del raiz con "read", NO puede ver Interno.

---

## Compartir con el cliente (portal)

### Paso 1: Identificar partner

```python
partner = odoo_read('res.partner', [project['partner_id'][0]], ['commercial_partner_id'])
target_partner_id = partner['commercial_partner_id'][0] if partner.get('commercial_partner_id') else partner['id']
```

### Paso 2: Anadir como miembro del raiz

```python
existing = odoo_search('knowledge.article.member',
    [['article_id', '=', root_id], ['partner_id', '=', target_partner_id]])
if not existing:
    odoo_create('knowledge.article.member', {
        'article_id': root_id,
        'partner_id': target_partner_id,
        'permission': 'read',
    })
```

Solo se anade al RAIZ. Los hijos heredan (excepto Interno).

---

## Patron search-before-create

Para todos los articulos:

```python
def ensure_article(name, parent_id, icon, body='', is_desynchronized=False):
    existing = odoo_search('knowledge.article',
        [['name', '=', name], ['parent_id', '=', parent_id]], limit=1)
    if existing:
        return existing[0]
    vals = {
        'name': name,
        'parent_id': parent_id,
        'icon': icon,
        'internal_permission': 'write',
        'body': body,
    }
    if is_desynchronized:
        vals['is_desynchronized'] = True
    return odoo_create('knowledge.article', vals)
```

Esto garantiza idempotencia: ejecutar el skill multiples veces NO duplica articulos.

---

## Vinculacion proyecto ↔ knowledge

Al final del Paso 4:

```python
odoo_write('project.project', [project_id], {'x_studio_knowledge': root_id})
```

Si el campo no existe, abortar y avisar al usuario.
