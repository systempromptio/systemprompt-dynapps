// Display names for user ids, read in batches from the drive people route
// and cached for the page, so presence, owners and grants show people.
import { readJson } from '../services/drive-rows.js';

const names = new Map();

export const resolvePeople = async (peopleApi, ids) => {
  const missing = [...new Set(ids.filter((id) => id && !names.has(id)))];
  if (!missing.length) return;
  try {
    const body = await readJson(`${peopleApi}?ids=${missing.map(encodeURIComponent).join(',')}`);
    for (const p of body?.people ?? []) names.set(p.user_id, p.display_name || p.email || p.user_id);
  } catch {
    // Names are a convenience; every caller falls back to the raw id.
  }
};

export const person = (id) => (id ? names.get(id) ?? id : '—');

export const initialsOf = (name) => {
  const parts = String(name).split(/[\s@._-]+/).filter(Boolean);
  return parts.slice(0, 2).map((p) => p[0].toUpperCase()).join('') || '?';
};

export const toneOf = (id) => {
  let sum = 0;
  for (const ch of String(id)) sum = (sum + ch.charCodeAt(0)) % 12;
  return sum;
};

export const documentIdOf = (root) => decodeURIComponent(root.dataset.documentApi.split('/').pop());

export const subjectsApiOf = (root) => `${root.dataset.documentsApi}/subjects`;

// Core's subject search is the one authority on who exists and, with a
// document id, on the access each match holds there (`none` for no access).
export const searchPeople = async (subjectsApi, term, documentId) => {
  const params = new URLSearchParams({ kind: 'user', q: term, limit: '8' });
  if (documentId) params.set('document_id', documentId);
  const body = await readJson(`${subjectsApi}?${params}`);
  const people = (body?.subjects ?? []).map((s) => ({
    user_id: s.value,
    display_name: s.display_name,
    email: s.detail ?? null,
    access_level: s.access_level === 'none' ? null : s.access_level ?? null,
  }));
  for (const p of people) names.set(p.user_id, p.display_name || p.email || p.user_id);
  return people;
};

// Any other subject kind (role, group, …) through the same search. A kind
// whose values cannot be listed answers `searchable: false`, and the caller
// takes the typed text as the value.
export const searchSubjectsOfKind = async (subjectsApi, kind, term) => {
  const params = new URLSearchParams({ kind, q: term, limit: '8' });
  const body = await readJson(`${subjectsApi}?${params}`);
  const matches = (body?.subjects ?? []).map((s) => ({
    user_id: s.value,
    display_name: s.display_name || s.value,
    email: s.detail ?? null,
  }));
  return { searchable: body?.searchable !== false, matches };
};

const ORDER = ['view', 'comment', 'edit', 'share', 'own'];

export const permits = (level, needed) => ORDER.indexOf(level) >= ORDER.indexOf(needed);
