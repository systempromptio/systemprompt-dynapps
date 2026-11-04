// Document classes: the reads and the one write every class control shares,
// and the sentence that explains an effective agent cap. The server decides
// whether a change is allowed; this module only asks.
import { rawFetch } from './api.js';

const CLASS_API = '/drive/classes';
const enc = encodeURIComponent;

const CAP_TEXT = {
  none: 'nothing',
  read: 'read',
  comment: 'comment',
  suggest: 'suggest changes',
  edit: 'edit',
};

export const loadClassOptions = () => rawFetch(CLASS_API);

const CAP_PATHS = { document: '/api/v1/documents', folder: '/api/v1/document-folders' };

// The class grants a cap; an owner override on the target or a folder above
// it can only lower it, so the effective cap is core's answer, not ours.
const loadEffectiveCaps = async (kind, id) => {
  const url = (attended) => `${CAP_PATHS[kind]}/${enc(id)}/agent-cap?attended=${attended}`;
  const [attended, unattended] = await Promise.all([rawFetch(url(true)), rawFetch(url(false))]);
  return { attended: attended.agent_cap, unattended: unattended.agent_cap };
};

const withEffectiveCaps = async (kind, id, view) => {
  try {
    return { ...view, ...(await loadEffectiveCaps(kind, id)) };
  } catch {
    return { ...view, reason: `${view.reason} (owner limits could not be read)` };
  }
};

export const loadClass = async (kind, id) => withEffectiveCaps(kind, id, await rawFetch(`${CLASS_API}/${kind}/${enc(id)}`));

export const saveClass = async (kind, id, classId, reason) => withEffectiveCaps(kind, id, await rawFetch(`${CLASS_API}/${kind}/${enc(id)}`, {
  method: 'PUT',
  body: JSON.stringify({ class_id: classId, reason: reason ?? null }),
}));

export const loadClassLabels = async (kind, ids) => {
  if (!ids.length) return new Map();
  const rows = await rawFetch(`${CLASS_API}/${kind}?ids=${enc(ids.join(','))}`);
  return new Map(rows.map((row) => [row.id, row]));
};

const sourceText = (view) => {
  if (view.inherited_from) return 'inherited from its folder';
  if (view.is_default) return 'the default';
  return 'set here';
};

export const agentsCanText = (view) => {
  const { attended, unattended } = view;
  const caps = `${CAP_TEXT[attended]} in your session, ${CAP_TEXT[unattended]} when running on their own`;
  return `Agents can: ${caps}. ${view.label} (${sourceText(view)}): ${view.reason}`;
};

export const rankOf = (options, classId) => options.findIndex((o) => o.id === classId);

export const isLoosening = (options, from, to) => rankOf(options, to) < rankOf(options, from);
