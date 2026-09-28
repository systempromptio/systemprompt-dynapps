// Tweening the text of a figure from what it shows to what it should show:
// "$1,204.50" to "$1,310.20" counts up through the same prefix, grouping and
// decimals. Anything the pattern cannot read — an em-dash, a percentage with
// two numbers — snaps straight to the new text, as does everything under
// reduced motion.

const NUMBER_PATTERN = /^([^\d−-]*)(-?[\d,]*\.?\d+)(.*)$/;
const DEFAULT_MS = 300;

const reduceMotion = () => window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false;

const parse = (text) => {
  const match = NUMBER_PATTERN.exec(text.trim());
  if (!match) return null;
  const [, prefix, raw, suffix] = match;
  const decimals = raw.includes('.') ? raw.length - raw.indexOf('.') - 1 : 0;
  const value = Number.parseFloat(raw.replaceAll(',', ''));
  if (!Number.isFinite(value)) return null;
  return { prefix, value, decimals, grouped: raw.includes(','), suffix };
};

const format = (value, { decimals, grouped }) =>
  value.toLocaleString('en-US', {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
    useGrouping: grouped,
  });

const easeOut = (t) => 1 - (1 - t) ** 3;

export const tweenText = (el, nextText, ms = DEFAULT_MS) => {
  const current = el.textContent;
  if (current === nextText) return;
  const from = parse(current);
  const to = parse(nextText);
  const snap = !from || !to || from.prefix !== to.prefix || from.suffix !== to.suffix || reduceMotion();
  if (snap) {
    el.textContent = nextText;
    return;
  }
  const started = performance.now();
  const step = (now) => {
    const t = Math.min((now - started) / ms, 1);
    const value = from.value + (to.value - from.value) * easeOut(t);
    el.textContent = t < 1 ? `${to.prefix}${format(value, to)}${to.suffix}` : nextText;
    if (t < 1) window.requestAnimationFrame(step);
  };
  window.requestAnimationFrame(step);
};
