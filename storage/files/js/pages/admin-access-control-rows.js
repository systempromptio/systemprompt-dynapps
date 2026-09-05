import { cloneTemplate, isWritable, setText, slot } from './admin-access-control-state.js';

const LAYER_CLASSES = {
  user: 'is-user',
  group: 'is-group',
  project: 'is-project',
  role: 'is-role',
  default: 'is-default',
};

const setOverrideState = (row, layer, effective) => {
  const pressed = {
    allow: layer === 'user' && effective === 'allow',
    deny: layer === 'user' && effective === 'deny',
    inherit: layer !== 'user',
  };
  for (const btn of row.querySelectorAll('.ac-override__btn')) {
    btn.setAttribute('aria-pressed', String(pressed[btn.dataset.override]));
    btn.disabled = !isWritable();
  }
};

const buildRow = (entityType, data) => {
  const effective = data.effective || 'deny';
  const row = cloneTemplate('matrix-row');
  row.dataset.entityType = entityType;
  row.dataset.entityId = data.entity_id;
  setText(row, 'name', data.entity_name);
  setText(row, 'id', data.entity_id);
  setText(row, 'effective', effective);
  slot(row, 'effective').classList.add(`is-${effective}`);
  setText(row, 'layer', data.source.layer);
  slot(row, 'layer').classList.add(LAYER_CLASSES[data.source.layer] ?? 'is-default');
  setText(row, 'detail', data.source.detail);
  setOverrideState(row, data.source.layer, effective);
  return row;
};

export const buildMatrixSection = (section) => {
  const rows = section.rows ?? [];
  const el = cloneTemplate('matrix-section');
  setText(el, 'label', section.label);
  setText(el, 'count', rows.length === 0 ? '' : rows.length);
  if (rows.length === 0) {
    el.append(cloneTemplate('matrix-empty'));
    return el;
  }
  for (const row of rows) el.append(buildRow(section.entity_type, row));
  return el;
};
