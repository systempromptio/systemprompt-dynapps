// The mention token a comment body stores: `@[Display Name](user:<id>)`. The
// id is what core notifies and checks for access, and the name keeps the
// body readable anywhere the token is shown raw (exports, notifications).
// The grammar is core's: a label of 1 to 120 chars with none of []() or a
// newline; core refuses a body with a token that breaks it. This file holds
// the grammar alone, with no page imports, so tests and the Markdown reader
// share it with the composer.

// The `u` flag makes {1,120} count code points, as core counts chars.
export const TOKEN = /@\[([^\[\]()\n]{1,120})\]\(user:([^)\s]+)\)/gu;

const MAX_LABEL = 120;

const labelOf = (name) => [...String(name).replace(/[[\]()\n]/g, ' ').trim()].slice(0, MAX_LABEL).join('').trim() || 'someone';

export const mentionToken = ({ user_id: id, display_name: name, email }) => `@[${labelOf(name || email || id)}](user:${id})`;

export const mentionIds = (body) => [...new Set(Array.from(body.matchAll(TOKEN), (m) => m[2]))];
