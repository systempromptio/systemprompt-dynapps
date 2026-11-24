// The `user` capability as the viewer shell answers it: the viewer, and the
// people a page names, as far as the manifest's `user` scopes let the page
// see them (nothing but `isOwner` and `canEdit` undeclared, the viewer's id
// with `user: {}`, names with `profile`, emails with `email`). Names and
// emails come from the Drive's people route and the documents directory
// search, as the viewer; every read resolves and never rejects.
import { avatarFor, colorFor } from './labels.js';

const MAX_PEOPLE = 200;
const MAX_HITS = 8;

export const createViewer = ({ urls, send, snapshot }) => {
  const people = new Map();

  const scopes = () => {
    const user = snapshot().capabilities && snapshot().capabilities.user;
    return user ? (user.scopes || []) : null;
  };
  const allows = (scope) => (scopes() || []).includes(scope);
  const self = () => snapshot().viewer || {};
  const myId = () => (scopes() ? self().id || null : null);

  const profileOf = (id, person) => {
    const name = allows('profile') && person ? person.display_name || '' : '';
    const email = allows('email') && person ? person.email || null : null;
    const color = colorFor(id);
    return { id, name, avatarUrl: avatarFor(name, color), color, email, isMe: id === myId(), guest: false, roles: null };
  };

  const lookUp = async (ids) => {
    const missing = ids.filter((id) => !people.has(id)).slice(0, MAX_PEOPLE);
    if (missing.length === 0 || !allows('profile')) return;
    const reply = await send(`${urls.people}?ids=${missing.map(encodeURIComponent).join(',')}`, { method: 'GET' });
    if (reply.status !== 200) return;
    (reply.body.people || []).forEach((person) => people.set(person.user_id, person));
  };

  const me = () => {
    const id = myId();
    const name = allows('profile') ? self().name || '' : '';
    const color = colorFor(self().id || '');
    return {
      id,
      name,
      avatarUrl: avatarFor(id ? name : '', color),
      color,
      email: allows('email') ? self().email || null : null,
      isOwner: Boolean(self().owner),
      canEdit: Boolean(snapshot().writer),
    };
  };

  const profiles = async ({ ids }) => {
    await lookUp(ids);
    return Object.fromEntries(ids.map((id) => [id, profileOf(id, people.get(id))]));
  };

  const search = async ({ query }) => {
    if (!allows('profile')) return [];
    if (query === '') {
      const mine = self().id ? [[self().id, { display_name: self().name, email: self().email }]] : [];
      const others = [...people.entries()].filter(([id]) => id !== self().id);
      return [...mine, ...others].slice(0, MAX_HITS).map(([id, person]) => profileOf(id, person));
    }
    const found = `${urls.search}?query=${encodeURIComponent(query)}&limit=${MAX_HITS}&document_id=${encodeURIComponent(urls.document)}`;
    const reply = await send(found, { method: 'GET' });
    if (reply.status !== 200) return [];
    return (reply.body.people || []).filter((person) => person.name).map((person) => {
      people.set(person.user_id, { user_id: person.user_id, display_name: person.name, email: person.email });
      return profileOf(person.user_id, people.get(person.user_id));
    });
  };

  return { me, profiles, search };
};
