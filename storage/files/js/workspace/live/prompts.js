// The viewer's own prompts in the shell of a live page: allowing a
// connector for the page, approving one action, and the permissions panel.
// They are the shell's, drawn outside the page's frame, so the page can
// neither draw nor click them. One dialog is open at a time; a second ask
// waits for the first to close. All text is set with textContent.
const STATES = [
  ['granted', 'Allowed'],
  ['prompt', 'Ask first'],
  ['denied', 'Blocked'],
];

const el = (tag, className, attrs = {}) => Object.assign(document.createElement(tag), { className, ...attrs });

export const createPrompts = (root, onOpenChange) => {
  let queue = Promise.resolve();
  let open = false;

  const dialog = (build) => {
    const run = () => new Promise((resolve) => {
      const shell = el('dialog', 'sp-dialog sp-live__dialog');
      const form = el('form', 'sp-dialog-form', { method: 'dialog' });
      const finish = build(form, resolve);
      shell.append(form);
      shell.addEventListener('close', () => {
        shell.remove();
        open = false;
        onOpenChange(false);
        finish(shell.returnValue);
      }, { once: true });
      root.append(shell);
      open = true;
      onOpenChange(true);
      shell.showModal();
    });
    const next = queue.then(run, run);
    queue = next.then(() => null, () => null);
    return next;
  };

  const actions = (form, cancel, accept) => {
    const menu = el('menu', 'sp-live__actions');
    menu.append(
      el('button', 'sp-btn sp-btn--sm sp-btn--ghost', { type: 'submit', value: 'cancel', textContent: cancel }),
      el('button', 'sp-btn sp-btn--sm sp-btn--primary', { type: 'submit', value: 'accept', textContent: accept }),
    );
    form.append(menu);
  };

  const consent = (servers) => dialog((form, resolve) => {
    form.append(
      el('h3', '', { textContent: 'Allow this page to use your connectors?' }),
      el('p', 'sp-live__text', { textContent: 'It reads and acts as you, with your own access, through:' }),
    );
    const list = el('ul', 'sp-live__servers');
    list.append(...servers.map((server) => el('li', '', { textContent: server })));
    form.append(list, el('p', 'sp-live__muted', { textContent: 'You can change this later in the page\'s permissions.' }));
    actions(form, 'Don\'t allow', 'Allow');
    return (value) => resolve(value === 'accept');
  });

  const confirm = ({ server, tool, summary }) => dialog((form, resolve) => {
    form.append(
      el('h3', '', { textContent: 'Allow this action?' }),
      el('p', 'sp-live__text', { textContent: `The page wants to run "${tool}" on "${server}" as you. It may change data there.` }),
    );
    if (summary.length > 0) {
      const facts = el('dl', 'sp-live__summary');
      summary.forEach(([key, text]) => facts.append(el('dt', '', { textContent: key }), el('dd', '', { textContent: text })));
      form.append(facts);
    }
    actions(form, 'Cancel', 'Allow');
    return (value) => resolve(value === 'accept');
  });

  const panel = ({ servers, states, decide }) => dialog((form, resolve) => {
    form.append(el('h3', '', { textContent: 'Page permissions' }));
    if (servers.length === 0) form.append(el('p', 'sp-live__muted', { textContent: 'This page uses no connectors.' }));
    const rows = el('ul', 'sp-live__permissions');
    servers.forEach((server) => {
      const scope = `mcp:${server}`;
      const choice = el('select', 'sp-live__choice', { name: scope });
      choice.setAttribute('aria-label', `Access to ${server}`);
      STATES.forEach(([value, label]) => choice.append(el('option', '', { value, textContent: label, selected: states[scope] === value })));
      choice.addEventListener('change', async () => {
        choice.disabled = true;
        try {
          await decide(scope, choice.value);
        } finally {
          choice.disabled = false;
        }
      });
      const row = el('li', 'sp-live__permission');
      row.append(el('span', '', { textContent: server }), choice);
      rows.append(row);
    });
    form.append(rows);
    const menu = el('menu', 'sp-live__actions');
    menu.append(el('button', 'sp-btn sp-btn--sm sp-btn--primary', { type: 'submit', value: 'close', textContent: 'Done' }));
    form.append(menu);
    return () => resolve();
  });

  return { consent, confirm, panel, busy: () => open };
};
