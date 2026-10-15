# Anatomía UI de Odoo — esqueleto para mockups

Copia-pega y adapta. HTML **autocontenido** (CSS/JS inline, sin CDNs). Estos tokens y estructuras
reproducen la UI real del backend de Odoo 17-19. No inventes componentes que Odoo no tiene.

## Tokens de marca (paleta real Odoo)

```css
:root{
  --o-purple:#714B67;--o-purple-d:#5c3d54;   /* primary Odoo */
  --o-bg:#f4f4f6;--o-white:#fff;--o-border:#e0e0e6;
  --o-text:#1f2937;--o-muted:#6b7280;--o-link:#017e84;
  --o-green:#28a745;--o-blue:#0d6efd;--o-orange:#f0ad4e;--o-red:#dc3545;
}
*{box-sizing:border-box}
body{margin:0;font-family:"Segoe UI",Roboto,Helvetica,Arial,sans-serif;background:var(--o-bg);color:var(--o-text);font-size:13px}
```

## Chrome común (todas las vistas de backend)

```html
<!-- Top navbar (app + menús) -->
<div class="navbar"><span class="brand">App</span>
  <span class="menu"><span>Menú1 ▾</span><span>Menú2 ▾</span></span>
  <span class="spacer"></span><span class="sysname">CLIENTE · Odoo 18</span></div>
<!-- Control panel: breadcrumb + botones de acción + pager -->
<div class="cp"><div class="breadcrumb">Modelo / <b>REG/0001</b></div>
  <div class="btns"><button class="btn primary">Guardar</button><button class="btn">Descartar</button>
    <button class="btn warn">Acción workflow</button></div>
  <div class="pager"><span>2 / 137</span><span class="arrow">‹</span><span class="arrow">›</span></div></div>
```
```css
.navbar{background:var(--o-purple);color:#fff;display:flex;align-items:center;gap:18px;padding:0 14px;height:44px}
.navbar .spacer{flex:1}.navbar .sysname{background:rgba(255,255,255,.15);padding:3px 9px;border-radius:4px;font-size:12px}
.cp{background:#fff;border-bottom:1px solid var(--o-border);padding:8px 16px;display:flex;align-items:center;gap:14px;flex-wrap:wrap}
.btn{border:1px solid var(--o-border);background:#fff;border-radius:4px;padding:5px 11px;font-size:12.5px;cursor:pointer}
.btn.primary{background:var(--o-purple);color:#fff;border-color:var(--o-purple)}
.btn.warn{background:#fff;color:var(--o-purple);border-color:var(--o-purple)}
.pager{margin-left:auto;color:var(--o-muted);font-size:12px;display:flex;gap:8px}
```

## FORM view

Componentes reales (usa solo los que apliquen):
- **statusbar** (arriba-dcha): las etapas de un campo `state`/selection o `stage_id`. La actual resaltada.
- **smart buttons**: caja de botones con `count` + label; cada uno abre un related (`One2many`/related count).
- **título** (`<h1>` del campo principal) + **grupos de campos** en 2 columnas (`<group>`).
- **notebook** con pestañas (`<page>`), típicamente con listas `One2many`.
- **chatter** a la derecha (mensajes / notas de log / actividades / seguidores) — solo si el modelo hereda `mail.thread`.

```html
<div class="sheet">
  <div class="statusbar"><div class="stages">
    <div class="stage done">Borrador</div><div class="stage current">En curso</div><div class="stage">Hecho</div></div></div>
  <div class="smartbar">
    <div class="smart"><span class="n">3</span><span class="l">🧾 Facturas</span></div>
    <div class="smart"><span class="n">4h</span><span class="l">⏱ Partes</span></div></div>
  <div class="body">
    <div class="title">REG/0001</div>
    <div class="groups">
      <div><div class="fld"><label>Cliente</label><span class="val link">ACME S.L.</span></div></div>
      <div><div class="fld req"><label>Fecha</label><span class="val">01/07/2026</span></div></div></div>
    <div class="tabs"><div class="tab active" data-t="l1">Líneas</div><div class="tab" data-t="l2">Notas</div></div>
    <div class="tabpane active" id="l1"><table>…o2m…</table><div class="addline">➕ Añadir línea</div></div>
  </div></div>
```
```css
.sheet{background:#fff;border:1px solid var(--o-border);border-radius:8px;max-width:1000px;margin:0 auto}
.statusbar{display:flex;justify-content:flex-end;border-bottom:1px solid var(--o-border);background:#fbfbfc;border-radius:8px 8px 0 0;overflow:hidden}
.stage{padding:9px 20px 9px 26px;font-size:12px;color:var(--o-muted);background:#f1f1f4;clip-path:polygon(0 0,calc(100% - 12px) 0,100% 50%,calc(100% - 12px) 100%,0 100%,12px 50%);margin-left:-10px}
.stage.done{background:#e7e2e6;color:var(--o-purple)}.stage.current{background:var(--o-purple);color:#fff;font-weight:600}
.smartbar{display:flex;padding:10px 18px 0}.smart{border:1px solid var(--o-border);border-right:none;background:#fff;padding:7px 16px;min-width:110px;text-align:left}
.smart:last-child{border-right:1px solid var(--o-border)}.smart .n{font-weight:700;color:var(--o-purple)}.smart .l{font-size:11.5px;color:var(--o-muted);display:block}
.groups{display:grid;grid-template-columns:1fr 1fr;gap:2px 40px;padding:0 22px}@media(max-width:820px){.groups{grid-template-columns:1fr}}
.fld{display:flex;padding:5px 0;border-bottom:1px dotted #eee}.fld label{width:150px;color:var(--o-muted);flex-shrink:0}.fld .val{font-weight:600}.fld .val.link{color:var(--o-link)}
.req label:after{content:" *";color:var(--o-red)}
.tabs{display:flex;gap:2px;border-bottom:2px solid var(--o-border);margin:18px 22px 0}.tab{padding:9px 16px;cursor:pointer;color:var(--o-muted);border-bottom:2px solid transparent;margin-bottom:-2px;font-weight:600}
.tab.active{color:var(--o-purple);border-bottom-color:var(--o-purple)}.tabpane{display:none;padding:14px 22px}.tabpane.active{display:block}
.addline{color:var(--o-link);padding:8px;cursor:pointer}
```

## LIST (tree) view
Cabeceras en MAYÚSCULAS, filas clicables, opcional agrupar (`group_by`) y totales de columnas numéricas.
```html
<table><thead><tr><th>Ref</th><th>Cliente</th><th>Estado</th><th class="num">Importe</th></tr></thead>
<tbody><tr class="o2m"><td>REG/0001</td><td>ACME</td><td><span class="st st-blue">En curso</span></td><td class="num">1.150,00 €</td></tr></tbody></table>
```
```css
table{width:100%;border-collapse:collapse;font-size:12.5px}
th{text-align:left;color:var(--o-muted);font-weight:600;font-size:11.5px;padding:7px 8px;border-bottom:1px solid var(--o-border);text-transform:uppercase;letter-spacing:.3px}
td{padding:8px;border-bottom:1px solid #f0f0f2}.num{text-align:right;font-variant-numeric:tabular-nums}
tr.o2m:hover{background:#faf7f9;cursor:pointer}
.st{display:inline-block;padding:2px 9px;border-radius:11px;font-size:11px;font-weight:600}
.st-blue{background:#e7f1ff;color:#0d6efd}.st-green{background:#e2f7e9;color:#1e7e34}.st-grey{background:#f1f1f4;color:#6b7280}.st-amber{background:#fff3cd;color:#8a6d00}
```

## KANBAN view
Columnas por `stage_id`/`state`; tarjetas arrastrables (simuladas). En v19 la tarjeta es `<t t-name="card">`.
```html
<div class="kb"><div class="kcol"><div class="khead">Por hacer <span>3</span></div>
  <div class="kcard"><div class="kt">REG/0001</div><div class="km">ACME S.L. · 1.150 €</div><div class="ktags"><span class="st st-amber">Urgente</span></div></div>
</div><div class="kcol"><div class="khead">En curso <span>2</span></div></div></div>
```
```css
.kb{display:flex;gap:12px;padding:16px;overflow-x:auto}.kcol{background:#f1f1f4;border-radius:8px;padding:8px;min-width:260px}
.khead{font-weight:700;color:var(--o-muted);padding:4px 6px 10px;display:flex;justify-content:space-between}
.kcard{background:#fff;border:1px solid var(--o-border);border-radius:6px;padding:10px;margin-bottom:8px;box-shadow:0 1px 2px rgba(0,0,0,.05)}
.kt{font-weight:700}.km{color:var(--o-muted);font-size:12px;margin:3px 0}.ktags{margin-top:6px}
```

## SEARCH / filtros
Barra de búsqueda con facets, panel Filtros/Agrupar/Favoritos.
```html
<div class="search"><div class="sbox">🔎 <span class="facet">Cliente: ACME ✕</span><span class="facet">Estado: En curso ✕</span></div>
  <div class="sactions"><span>Filtros ▾</span><span>Agrupar por ▾</span><span>Favoritos ▾</span></div></div>
```
```css
.search{background:#fff;border:1px solid var(--o-border);border-radius:6px;padding:6px 10px;display:flex;gap:12px;align-items:center;margin:12px}
.sbox{flex:1;display:flex;gap:6px;align-items:center}.facet{background:#eef4ff;color:#0d6efd;border-radius:11px;padding:2px 9px;font-size:11.5px}
.sactions{display:flex;gap:14px;color:var(--o-muted);font-size:12.5px}
```

## DASHBOARD (tarjetas KPI + gráficos placeholder)
Solo KPIs y contenedores de gráfico como placeholders (los gráficos reales son `graph`/`pivot` o Spreadsheet).
```html
<div class="dash"><div class="kpi"><div class="kn">137</div><div class="kl">Órdenes abiertas</div></div>
  <div class="kpi"><div class="kn">24.310 €</div><div class="kl">Facturado mes</div></div>
  <div class="chart">📊 [gráfico: graph view / Spreadsheet]</div></div>
```
```css
.dash{display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:14px;padding:16px}
.kpi{background:#fff;border:1px solid var(--o-border);border-radius:8px;padding:16px}.kn{font-size:28px;font-weight:700;color:var(--o-purple)}.kl{color:var(--o-muted);font-size:12.5px}
.chart{grid-column:1/-1;background:#fff;border:1px solid var(--o-border);border-radius:8px;padding:40px;text-align:center;color:var(--o-muted)}
```

## JS mínimo (tabs + expandir filas)
```html
<script>
document.querySelectorAll('.tab').forEach(t=>t.addEventListener('click',()=>{
  document.querySelectorAll('.tab,.tabpane').forEach(x=>x.classList.remove('active'));
  t.classList.add('active');document.getElementById(t.dataset.t).classList.add('active');}));
</script>
```
