// Mentions in a comment body are stored as `@[Display Name](user:<id>)`: the
// id is what core notifies and checks for access, and the name keeps the
// body readable anywhere the token is shown raw (exports, notifications).
// The thread renders each token as a chip carrying the current name. The
// grammar is core's: a label of 1 to 120 chars with none of []() or a
// newline; core refuses a body with a token that breaks it.
import { person } from '../people.js';

// The `u` flag makes {1,120} count code points, as core counts chars.
const TOKEN = /@\[([^\[\]()\n]{1,120})\]\(user:([^)\s]+)\)/gu;

const MAX_LABEL = 120;

const labelOf = (name) => [...String(name).replace(/[[\]()\n]/g, ' ').trim()].slice(0, MAX_LABEL).join('').trim() || 'someone';

export const mentionToken = ({ user_id: id, display_name: name, email }) => `@[${labelOf(name || email || id)}](user:${id})`;

export const mentionIds = (body) => [...new Set(Array.from(body.matchAll(TOKEN), (m) => m[2]))];

export const renderBody = (body) => {
  const fragment = document.createDocumentFragment();
  let last = 0;
  for (const match of body.matchAll(TOKEN)) {
    const [token, label, id] = match;
    fragment.append(body.slice(last, match.index));
    const chip = document.createElement('span');
    chip.className = 'sp-doc-mention';
    chip.dataset.userId = id;
    const name = person(id);
    chip.textContent = `@${name === id ? label : name}`;
    fragment.append(chip);
    last = match.index + token.length;
  }
  fragment.append(body.slice(last));
  return fragment;
};
