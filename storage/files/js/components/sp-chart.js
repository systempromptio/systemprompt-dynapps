// Hover narration for the server-rendered SVG line charts. Any `[data-chart]`
// figure keeps its plot exactly as the server drew it; this adds a crosshair
// that snaps to the nearest bucket, a tooltip that prints every series' value
// for that bucket (values come from `data-values`, never from geometry), and
// legend items that toggle a series. Without this module the chart stays
// complete and static, only without the hover layer.

const PLOT_WIDTH = 100;

const parseValues = (text) => text.split(',').map((v) => Number.parseInt(v, 10) || 0);

const parseLabels = (text) => {
  try { const labels = JSON.parse(text || '[]'); return Array.isArray(labels) ? labels : []; } catch { return []; }
};

const formatValue = (value, unit) => {
  if (unit === 'µ$') return `$${(value / 1_000_000).toFixed(value >= 1_000_000 ? 2 : 4)}`;
  if (unit === 'ms') return value >= 1000 ? `${(value / 1000).toFixed(1)}s` : `${value}ms`;
  const text = value.toLocaleString();
  return unit ? `${text} ${unit}` : text;
};

export const initChart = (figure) => {
  const plot = figure.querySelector('[data-chart-plot]');
  const cursor = figure.querySelector('[data-chart-cursor]');
  const tip = figure.querySelector('[data-chart-tip]');
  if (!plot || !cursor || !tip) return null;
  const labels = parseLabels(figure.dataset.labels);
  const unit = figure.dataset.unit ?? '';
  const series = [...plot.querySelectorAll('[data-series]')].map((group) => ({
    group,
    label: group.dataset.label ?? '',
    color: group.dataset.color ?? '--sp-chart-purple',
    values: parseValues(group.dataset.values ?? ''),
    hidden: false,
  }));
  const buckets = Math.max(...series.map((s) => s.values.length), labels.length, 0);
  if (buckets === 0) return null;

  // The x axis places bucket i at (i + 0.5) / n of the plot width, so the
  // pointer's fraction of the width maps straight back to a bucket index.
  const bucketAt = (event) => {
    const box = plot.getBoundingClientRect();
    const fraction = Math.min(Math.max((event.clientX - box.left) / box.width, 0), 0.9999);
    return Math.floor(fraction * buckets);
  };

  const show = (index) => {
    const x = ((index + 0.5) / buckets) * PLOT_WIDTH;
    cursor.setAttribute('x1', x);
    cursor.setAttribute('x2', x);
    cursor.hidden = false;
    const rows = series
      .filter((s) => !s.hidden)
      .map((s) => `<span class="sp-svgchart__tip-row"><i style="background: var(${s.color})"></i>${s.label}<b>${formatValue(s.values[index] ?? 0, unit)}</b></span>`)
      .join('');
    tip.innerHTML = `<span class="sp-svgchart__tip-label">${labels[index] ?? `#${index + 1}`}</span>${rows}`;
    tip.hidden = false;
    const box = plot.getBoundingClientRect();
    const left = ((index + 0.5) / buckets) * box.width;
    tip.style.left = `${Math.min(Math.max(left, tip.offsetWidth / 2), box.width - tip.offsetWidth / 2)}px`;
  };

  const hide = () => { cursor.hidden = true; tip.hidden = true; };

  plot.addEventListener('pointermove', (event) => show(bucketAt(event)));
  plot.addEventListener('pointerleave', hide);
  plot.tabIndex = 0;
  let focused = buckets - 1;
  plot.addEventListener('keydown', (event) => {
    const step = { ArrowLeft: -1, ArrowRight: 1, Home: -buckets, End: buckets }[event.key];
    if (step === undefined) return;
    event.preventDefault();
    focused = Math.min(Math.max(focused + step, 0), buckets - 1);
    show(focused);
  });
  plot.addEventListener('blur', hide);

  for (const item of figure.querySelectorAll('[data-chart-toggle]')) {
    const index = Number.parseInt(item.dataset.chartToggle, 10);
    const target = series[index];
    if (!target) continue;
    item.setAttribute('role', 'button');
    item.tabIndex = 0;
    const toggle = () => {
      target.hidden = !target.hidden;
      target.group.classList.toggle('is-hidden', target.hidden);
      item.classList.toggle('is-hidden', target.hidden);
      item.setAttribute('aria-pressed', target.hidden ? 'false' : 'true');
    };
    item.addEventListener('click', toggle);
    item.addEventListener('keydown', (event) => {
      if (event.key !== 'Enter' && event.key !== ' ') return;
      event.preventDefault();
      toggle();
    });
  }
  figure.classList.add('sp-svgchart--live');
  return { show, hide, series };
};

export const initAllCharts = (scope = document) =>
  [...scope.querySelectorAll('[data-chart]')].map(initChart).filter(Boolean);

if (typeof document !== 'undefined') initAllCharts();
