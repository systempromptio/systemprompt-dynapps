// One row per file in the upload list, cloned from the page's
// `drive-upload-row` template. The row owns its <progress>, its status text
// and its Cancel button; the queue drives it through the returned handle.
const PHASE_LABEL = { hashing: 'Checking', uploading: 'Uploading' };

const percent = (done, total) => (total ? Math.round((done / total) * 100) : 100);

export const createRow = (list, file, onCancel) => {
  const row = document.querySelector('template[data-template="drive-upload-row"]').content
    .firstElementChild.cloneNode(true);
  const bar = row.querySelector('[data-field="bar"]');
  const status = row.querySelector('[data-field="status"]');
  const cancel = row.querySelector('[data-action="upload-cancel"]');
  row.querySelector('[data-field="name"]').textContent = file.name;
  cancel.addEventListener('click', onCancel, { once: true });
  list.append(row);
  list.hidden = false;
  const finish = (state, text) => {
    row.classList.add(`sp-upload-row--${state}`);
    status.textContent = text;
    cancel.hidden = true;
  };
  return {
    progress: ({ phase, done, total }) => {
      bar.value = percent(done, total);
      status.textContent = `${PHASE_LABEL[phase]} ${bar.value}%`;
    },
    importing: () => { status.textContent = 'Importing'; },
    done: (text) => finish('done', text),
    failed: (message) => finish('failed', message),
  };
};
