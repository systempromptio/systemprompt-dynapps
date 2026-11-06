// Open requests for access to a document, each granted or declined by someone
// who can share it.
import { person } from './people.js';
import { button, el, heading } from './pane-dom.js';

const requestRow = (request, onDecide) => {
  const item = el('li', 'sp-ws__entry');
  item.append(el('span', 'sp-ws__entry-title', `${person(request.requester_user_id)} asks for ${request.level}`));
  if (request.message) item.append(el('span', 'sp-ws__muted', request.message));
  const actions = el('div', 'sp-ws-pane__actions');
  actions.append(
    button('Grant', 'sp-btn--primary', () => onDecide(request.request_id, 'grant')),
    button('Decline', 'sp-btn--outline', () => onDecide(request.request_id, 'decline')),
  );
  item.append(actions);
  return item;
};

export const accessRequests = (requests, onDecide) => {
  if (!requests?.length) return null;
  const box = el('section', 'sp-ws__pane');
  const list = el('ul', 'sp-ws__list');
  list.append(...requests.map((r) => requestRow(r, onDecide)));
  box.append(heading('Access requests'), list);
  return box;
};
