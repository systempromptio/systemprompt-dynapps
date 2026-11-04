// The Class row in the details drawer: a picker that offers only stricter
// classes, unless the viewer is an admin, who may loosen with a reason.
import { showPromptDialog } from '../services/confirm.js';
import {
  agentsCanText, isLoosening, loadClass, loadClassOptions, rankOf, saveClass,
} from '../services/document-class.js';

const KIND = 'document';

const fillPicker = (select, options, view) => {
  const floor = rankOf(options, view.class_id);
  const choices = options.filter((o, rank) => view.can_loosen || rank >= floor);
  select.replaceChildren(...choices.map((o) => new Option(o.label, o.id, false, o.id === view.class_id)));
  select.disabled = !view.can_change;
};

const render = (pane, options, view) => {
  fillPicker(pane.querySelector('[data-slot="ws-class"]'), options, view);
  pane.querySelector('[data-field="d-agents"]').textContent = agentsCanText(view);
  pane.dataset.classId = view.class_id;
};

const commit = async (pane, docId, options, next) => {
  const status = pane.querySelector('[data-field="d-class-status"]');
  const apply = async (reason) => {
    try {
      render(pane, options, await saveClass(KIND, docId, next, reason));
      status.textContent = '';
    } catch (err) {
      status.textContent = err.message;
      render(pane, options, await loadClass(KIND, docId));
    }
  };
  if (!isLoosening(options, pane.dataset.classId, next)) return apply(null);
  return showPromptDialog('Loosen class', 'Reason (recorded with the change)', '', apply);
};

const state = new WeakMap();

const bind = (pane) => {
  pane.querySelector('[data-slot="ws-class"]').addEventListener('change', (evt) => {
    const { docId, options } = state.get(pane);
    commit(pane, docId, options, evt.target.value);
  });
};

export const showClass = async (root, docId) => {
  const pane = root.querySelector('[data-slot="ws-class-row"]');
  if (!pane) return;
  const [options, view] = await Promise.all([loadClassOptions(), loadClass(KIND, docId)]);
  if (!state.has(pane)) bind(pane);
  state.set(pane, { docId, options });
  render(pane, options, view);
};
