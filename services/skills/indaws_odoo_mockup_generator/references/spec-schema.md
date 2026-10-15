# Spec del motor de mockups Odoo — contrato de 2 capas

La spec es **el corazón del motor**. Dos capas: el **modelo** (ORM) se define UNA vez; las **vistas**
referencian sus campos por nombre → coherencia automática entre form/list/kanban/search/etc.

```jsonc
{
  "odoo": "18",                         // major → CSS/anatomía y deltas de versión
  "model": {
    "name": "x_taller_orden",           // técnico; x_ = Studio, sin x_ = módulo dev
    "label": "Orden de reparación",
    "inherit": null,                    // p.ej. "repair.order" si extiende un modelo real
    "fields": [
      { "name": "km",
        "type": "integer",              // TIPO ORM (char/integer/float/monetary/boolean/date/
                                        //   datetime/text/html/binary/selection/many2one/
                                        //   one2many/many2many)
        "widget": "integer",            // WIDGET UI — debe estar en supportedTypes[type]
        "label": "Kilómetros",
        "origin": "new",                // "std" (ya existe en el modelo) | "new" (a crear)
        "required": true, "readonly": false, "invisible": false,
        "help": "", "placeholder": "",
        "compute": false,               // true → lógica → DEV
        "comodel": null,                // obligatorio en many2one/one2many/many2many
        "selection": []                 // valores si type=selection
      }
    ]
  },
  "views": [
    { "type": "form", "chatter": true, "chatterPosition": "right",   // "right" | "bottom"
      "header": { "statusbar": "state", "buttons": ["Cierre técnico"], "smart": ["diagnostico_ids"] },
      "body": [
        { "title": "name" },
        { "group": ["partner_id", "user_id", "schedule_date"] },
        { "group": ["vehiculo", "km", "riesgo"] },
        { "notebook": [ { "page": "Diagnósticos",
            "content": [ { "o2m": "diagnostico_ids",
              "columns": [ {"name":"averia","type":"char","label":"Avería"},
                           {"name":"importe","type":"monetary","widget":"monetary","label":"Importe"} ] } ] } ] }
      ] },
    { "type": "list",   "columns": ["name","partner_id","state","priority"] },
    { "type": "kanban", "groupby": "state", "card": ["name","partner_id","priority"] },
    { "type": "search", "fields": ["name","partner_id"], "filters": ["Mías"], "groupby": ["state"] }
  ]
}
```

## Reglas del motor
1. **tipo↔widget**: `widget` debe pertenecer a `supportedTypes[type]` (matriz `widget-matrix.json`, extraída de fuente). Si no → el motor lo marca inválido. Si falta `widget`, usa el por-defecto del tipo.
2. **Clasificación (derivada, no manual)** por campo/componente:
   - `origin:"std"` → **STD** (existe, gratis).
   - `origin:"new"` + tipo estándar + widget nativo + `compute:false` → **STUDIO** (se crea sin código).
   - `compute:true` / widget OWL / dominio-lógica / máquina de estados / wizard / QWeb → **DEV**.
   - vista ∈ {gantt, map, cohort} → **EE** (licencia).
3. **Coherencia**: los campos se definen en `model.fields`; las vistas solo referencian `name`. Cambiar un campo se propaga a todas las vistas.
4. **Relaciones**: `one2many`/`many2one` llevan `comodel`; un o2m puede definir columnas inline (mini-modelo) o apuntar a otro modelo de la spec (grafo).

## Emisores (consumen la misma spec)
- **HTML mockup** (validación cliente) — este motor.
- **XML de vista + modelo Python** (esqueleto para `odoo-module-creator`).
- **Doc de requisitos** (anotaciones dev → ticket).

## Versionado
`schemaVersion` + `odoo` major. El CSS y la anatomía se resuelven por major (v17≠v18≠v19).
