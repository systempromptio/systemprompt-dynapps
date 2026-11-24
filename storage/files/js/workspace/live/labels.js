// What the viewer shell shows of people and of a call: a person's colour and
// avatar (a data: URL, so the frame needs no network to draw it), and the
// readable summary of a tool's input that the write confirmation names.
// Pure functions, shared by the bridge and the prompts.
const PALETTE = ['#3b6fd8', '#2f8f6b', '#b05a2a', '#8a4fc2', '#c0446d', '#1f8a9e', '#7a7f1f', '#a83f3f'];
const MAX_ENTRIES = 8;
const MAX_TEXT = 80;

export const colorFor = (id) => {
  const text = String(id || '');
  const hash = [...text].reduce((sum, char) => (sum * 31 + char.charCodeAt(0)) % 104729, 7);
  return PALETTE[hash % PALETTE.length];
};

const initialsOf = (name) => String(name || '')
  .split(/\s+/)
  .filter(Boolean)
  .slice(0, 2)
  .map((word) => word[0].toUpperCase())
  .join('')
  .replace(/[^\p{L}\p{N}]/gu, '');

export const avatarFor = (name, color) => {
  const initials = initialsOf(name);
  const mark = initials
    ? `<text x="16" y="21" font-family="sans-serif" font-size="13" text-anchor="middle" fill="#fff">${initials}</text>`
    : '<circle cx="16" cy="13" r="5" fill="#fff"/><path d="M7 27c1-5 5-7 9-7s8 2 9 7" fill="#fff"/>';
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><rect width="32" height="32" rx="16" fill="${color}"/>${mark}</svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
};

const clip = (text) => (text.length > MAX_TEXT ? `${text.slice(0, MAX_TEXT - 1)}…` : text);

const valueText = (value) => {
  if (value === null || value === undefined) return '—';
  if (typeof value === 'string') return clip(value);
  if (typeof value === 'number' || typeof value === 'boolean') return String(value);
  if (Array.isArray(value)) return clip(`${value.length} item${value.length === 1 ? '' : 's'}: ${JSON.stringify(value)}`);
  return clip(JSON.stringify(value));
};

export const summaryOf = (input) => {
  if (input === null || typeof input !== 'object' || Array.isArray(input)) return [];
  const keys = Object.keys(input);
  const shown = keys.slice(0, MAX_ENTRIES).map((key) => [key, valueText(input[key])]);
  if (keys.length > MAX_ENTRIES) shown.push(['…', `${keys.length - MAX_ENTRIES} more`]);
  return shown;
};
