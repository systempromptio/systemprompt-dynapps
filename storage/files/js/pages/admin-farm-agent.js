// The agent page's launch dialog and copyable remote execution commands. The
// dialog is a plain form that works without script; this layer turns the
// open link into a modal, closes it in place, and counts prompt characters.
import '../components/sp-copy.js';
import { on } from '../services/events.js';

const launchDialog = () => document.querySelector('dialog[data-dialog="farm-launch"]');
const launchField = (name) => document.querySelector(`[data-field="${name}"]`);

const updateCount = () => {
  const prompt = launchField('farm-launch-prompt');
  const count = launchField('farm-launch-count');
  if (!prompt || !count) return;
  const length = [...prompt.value].length;
  const max = Number(prompt.getAttribute('maxlength')) || 0;
  count.textContent = String(length);
  count.parentElement?.classList.toggle('is-over', max > 0 && length > max);
};

const dropLaunchParam = () => {
  const url = new URL(window.location.href);
  if (!url.searchParams.has('launch')) return;
  url.searchParams.delete('launch');
  window.history.replaceState(null, '', url.toString());
};

const openLaunch = (event) => {
  const dialog = launchDialog();
  if (!dialog) return;
  event?.preventDefault();
  if (dialog.open) dialog.close();
  dialog.showModal();
  updateCount();
  launchField('farm-launch-prompt')?.focus();
};

const closeLaunch = (event) => {
  const dialog = launchDialog();
  if (!dialog) return;
  event?.preventDefault();
  dialog.close();
  dropLaunchParam();
};

export const initFarmAgent = () => {
  on('click', '[data-action="farm-launch-open"]', (event) => openLaunch(event));
  on('click', '[data-action="farm-launch-close"]', (event) => closeLaunch(event));
  on('input', '[data-field="farm-launch-prompt"]', updateCount);
  const dialog = launchDialog();
  if (dialog?.open) openLaunch(null);
  dialog?.addEventListener('close', dropLaunchParam);
};

initFarmAgent();
