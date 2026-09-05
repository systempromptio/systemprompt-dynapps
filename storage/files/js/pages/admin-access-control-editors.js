import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import {
  DB_SOURCE_LABEL,
  cloneTemplate,
  entitySections,
  isWritable,
  setText,
  showPane,
  slot,
} from './admin-access-control-state.js';

const RULES_PATH = '/access-control';

const setRowSource = (select, label) => {
  const source = select.closest('.ac-band-rule')?.querySelector('.ac-source');
  if (source) source.textContent = label;
};

const buildRule = (type, item) => {
  const row = cloneTemplate('band-rule');
  setText(row, 'label', item.label || item.id);
  setText(row, 'id', item.id);
  const select = row.querySelector('select');
  select.dataset.entityType = type;
  select.dataset.entityId = item.id;
  select.disabled = !isWritable();
  return row;
};

const buildSection = (section) => {
  const el = cloneTemplate('band-section');
  setText(el, 'label', section.label);
  const list = slot(el, 'rules');
  list.dataset.entityType = section.type;
  if (section.items.length === 0) {
    list.append(cloneTemplate('band-rule-empty'));
    return el;
  }
  for (const item of section.items) list.append(buildRule(section.type, item));
  return el;
};

const saveRule = (band, value, select) => {
  const { entityType, entityId } = select.dataset;
  if (select.value === 'inherit') {
    return apiFetch(`${RULES_PATH}/bulk-template`, {
      method: 'POST',
      body: JSON.stringify({
        entity_type: entityType,
        subject_type: band,
        subject_value: value,
        action: 'clear',
      }),
    });
  }
  const path = `${RULES_PATH}/entity/${encodeURIComponent(entityType)}/${encodeURIComponent(entityId)}/rules`;
  return apiFetch(path, {
    method: 'POST',
    body: JSON.stringify({ rule_type: band, rule_value: value, access: select.value }),
  });
};

const bindRules = (editor, band, value) => {
  editor.addEventListener('change', async (ev) => {
    const select = ev.target.closest('select[data-action="band-rule-access"]');
    if (!select) return;
    try {
      await saveRule(band, value, select);
      setRowSource(select, select.value === 'inherit' ? '—' : DB_SOURCE_LABEL);
    } catch {
      showToast(`Could not save the rule for ${value}`, 'error');
    }
  });
};

const applyCurrentRules = (editor, band, value, rules) => {
  for (const rule of rules) {
    if (rule.rule_type !== band || rule.rule_value !== value) continue;
    const select = editor.querySelector(
      `select[data-entity-type="${CSS.escape(rule.entity_type)}"][data-entity-id="${CSS.escape(rule.entity_id)}"]`,
    );
    if (!select) continue;
    select.value = rule.access;
    setRowSource(select, DB_SOURCE_LABEL);
  }
};

const loadCurrentRules = async (editor, band, value) => {
  try {
    const resp = await apiFetch(RULES_PATH);
    applyCurrentRules(editor, band, value, resp?.rules ?? []);
  } catch {
    showToast('Could not load the current access rules', 'error');
  }
};

export const renderBandEditor = (band, value, label) => {
  const editor = document.querySelector('[data-pane="editor"]');
  const content = cloneTemplate('band-editor');
  setText(content, 'name', label);
  const sections = slot(content, 'sections');
  for (const section of entitySections()) sections.append(buildSection(section));
  editor.replaceChildren(content);
  showPane('editor');
  if (isWritable()) bindRules(editor, band, value);
  loadCurrentRules(editor, band, value);
};
