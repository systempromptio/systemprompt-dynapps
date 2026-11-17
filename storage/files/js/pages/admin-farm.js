// Farm pages: a control marked data-autosubmit reloads the page with its
// form's values as soon as it changes, so the tenant (and date) pickers need
// no "Show" button. Without JS the form's <noscript> button does the same.
// A date is typed in pieces, so it waits for a pause before submitting.
const DATE_PAUSE_MS = 600;
let timer = null;

for (const form of document.querySelectorAll('form[method="get"]')) {
  form.addEventListener('change', (e) => {
    const target = e.target;
    if (!(target instanceof HTMLElement) || !target.matches('[data-autosubmit]')) return;
    clearTimeout(timer);
    if (target.matches('input[type="date"]')) {
      timer = setTimeout(() => form.submit(), DATE_PAUSE_MS);
    } else {
      form.submit();
    }
  });
}
