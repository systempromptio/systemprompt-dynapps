# JS snippets + dropdown map (Claude in Chrome → `javascript_tool`)

All snippets run in the page context on a `references` tab. Wrap any code using
`await`/`fetch` in `(async () => { ... })()` — top-level `await` is not supported.

## Contents
- Dropdown value map (verify live before writing)
- Read the real option map
- Scan all pages for empties
- Apply one page + verify
- Final sweep
- Save mechanism (debug)
- Page count

## Dropdown value map (verify live before writing)

> These `value`s are Odoo industry IDs and **can change**. ALWAYS re-read the live
> `<option>`s (next snippet) and rebuild this map if it differs before saving.

| value | texto |
|---|---|
| 15 | Administración Pública |
| 1  | Agricultura |
| 30 | Alimentación / Hospitalidad / Turismo |
| 12 | Bienes Raíces |
| 13 | Ciencia y tecnología |
| 6  | Construcción y Renovación |
| 16 | Educación |
| 18 | Entretenimiento |
| 21 | Extraterritorial |
| 3  | Fabricación / Mantenimiento |
| 11 | Finanzas / Seguros |
| 20 | Hogares |
| 2  | Minería |
| 59 | Reclutamiento y dotación de personal |
| 17 | Salud / Bienestar Social / Farmacéutica |
| 14 | Servicios públicos / Energía / Abastecimiento de agua |
| 19 | Servicios y consultoría |
| 10 | TI / Comunicación / Marketing |
| 8  | Transporte / Logística |
| 7  | Venta al por mayor / Venta al por menor |

## Read the real option map (run once, on the live page)

```js
const sel = document.querySelector('select[name="industry"]');
JSON.stringify(Array.from(sel.options).map(o => `${o.value}=${o.text.trim()}`));
```

## Scan all pages for references with EMPTY industry

```js
(async () => {
  const base = 'https://www.odoo.com/es_ES/my/partner/references';
  const out = [];
  for (let p = 1; p <= 10; p++) {
    const url = p === 1 ? base : `${base}/page/${p}`;
    const html = await fetch(url, { credentials: 'include' }).then(r => r.text());
    const doc = new DOMParser().parseFromString(html, 'text/html');
    Array.from(doc.querySelectorAll('select[name="industry"]')).forEach(sel => {
      const opt = sel.options[sel.selectedIndex];
      const isEmpty = sel.value === '' || (opt && opt.text.toLowerCase().includes('seleccionar'));
      if (isEmpty) {
        const tr = sel.closest('tr');
        const tds = Array.from(tr.querySelectorAll('td'));
        const webInput = tr.querySelector('input[name="website"]');
        out.push({
          page: p,
          dataId: tr.getAttribute('data-id'),
          name: tds[1] ? tds[1].innerText.trim() : '',
          web: webInput ? webInput.value.trim() : ''
        });
      }
    });
  }
  window.__empties = out;
  return JSON.stringify({ emptyTotal: out.length });
})();
```

Then read `window.__empties` in chunks if the output truncates:
```js
JSON.stringify(window.__empties.slice(0, 15));
```

## Apply one page (set + save each target), then verify

`targets` maps `dataId` → `industry_value` for the rows on the CURRENT page.

```js
(async () => {
  const targets = { "7383735": "19", "7340788": "18" }; // edit per page
  const wait = ms => new Promise(r => setTimeout(r, ms));
  const setter = Object.getOwnPropertyDescriptor(window.HTMLSelectElement.prototype, 'value').set;
  const log = [];
  for (const [id, val] of Object.entries(targets)) {
    const tr = document.querySelector(`tr.o_update_implementation[data-id="${id}"]`);
    if (!tr) { log.push(`ERR notfound ${id}`); continue; }
    const sel = tr.querySelector('select[name="industry"]');
    setter.call(sel, val);
    sel.dispatchEvent(new Event('input',  { bubbles: true }));
    sel.dispatchEvent(new Event('change', { bubbles: true }));
    (tr.querySelector('button.save') || document.querySelector('#save' + id)).click();
    log.push(`set ${sel.value} ${tr.querySelectorAll('td')[1].innerText.trim().slice(0, 35)}`);
    await wait(1500);
  }
  return JSON.stringify(log);
})();
```

Verify (re-fetch the same page URL and compare):

```js
(async () => {
  const targets = { "7383735": "19", "7340788": "18" };
  const url = 'https://www.odoo.com/es_ES/my/partner/references/page/3'; // current page
  const html = await fetch(url, { credentials: 'include' }).then(r => r.text());
  const doc = new DOMParser().parseFromString(html, 'text/html');
  return JSON.stringify(Object.entries(targets).map(([id, exp]) => {
    const tr = doc.querySelector(`tr.o_update_implementation[data-id="${id}"]`);
    const sel = tr.querySelector('select[name="industry"]');
    return `${sel.value === exp ? 'OK' : 'FAIL'} ${sel.value} ${tr.querySelectorAll('td')[1].innerText.trim().slice(0, 35)}`;
  }));
})();
```

## Final sweep (0 empties expected)

```js
(async () => {
  const base = 'https://www.odoo.com/es_ES/my/partner/references';
  let total = 0; const stillEmpty = [];
  for (let p = 1; p <= 10; p++) {
    const url = p === 1 ? base : `${base}/page/${p}`;
    const html = await fetch(url, { credentials: 'include' }).then(r => r.text());
    const doc = new DOMParser().parseFromString(html, 'text/html');
    Array.from(doc.querySelectorAll('select[name="industry"]')).forEach(sel => {
      total++;
      const opt = sel.options[sel.selectedIndex];
      if (sel.value === '' || (opt && opt.text.toLowerCase().includes('seleccionar'))) {
        stillEmpty.push(sel.closest('tr').querySelectorAll('td')[1].innerText.trim());
      }
    });
  }
  return JSON.stringify({ totalReferences: total, stillEmptyCount: stillEmpty.length, stillEmpty });
})();
```

## Confirming the save mechanism (debugging only)

`change` alone does **not** persist. Clicking `button.save` fires:
`POST https://www.odoo.com/my/partner/references/edit` → 200. Confirm with
`read_network_requests` (call it BEFORE the click; tracking starts on first call).

## Page count

The pager may grow over time. Read the last numeric pager link instead of
hard-coding 10:
```js
Math.max(1, ...Array.from(document.querySelectorAll('.pagination a'))
  .map(a => parseInt(a.textContent.trim(), 10)).filter(Number.isFinite));
```
