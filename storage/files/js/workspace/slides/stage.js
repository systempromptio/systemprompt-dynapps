// The stage: the current slide at full width, each shape absolutely placed
// from its EMU frame and edited through its own ProseMirror view. A redraw
// keeps the views of shapes that still exist, so a remote change never
// steals the local caret. Image and table shapes carry no text in the replica:
// they move, resize and delete like any shape, and their content stays as is.
import { frameOf, indexOfId, isTitle, shapesOf } from './deck.js';
import { placeFrame, wireFrames } from './frames.js';
import { mountShapeText } from './shape-text.js';

const grip = (mode, label) => {
  const el = document.createElement('span');
  el.className = mode === 'move' ? 'sp-deck-shape__grip' : 'sp-deck-shape__corner';
  el.dataset.frame = mode;
  el.title = label;
  return el;
};

const LOCKED_LABEL = { image: 'Image', table: 'Table', other: 'Embedded object' };

export const lockedKind = (shape) => LOCKED_LABEL[shape.get('kind')] ?? null;

const lockedEl = (shape) => {
  const el = document.createElement('div');
  el.className = 'sp-deck-shape sp-deck-shape--locked';
  el.dataset.shape = shape.get('id');
  el.dataset.kind = shape.get('kind');
  el.textContent = lockedKind(shape);
  el.title = `${lockedKind(shape)}: drag to move or resize. Its content is edited in the original application.`;
  const src = shape.get('src');
  if (shape.get('kind') === 'image' && typeof src === 'string' && src && mediaBase) {
    const img = Object.assign(document.createElement('img'), {
      className: 'sp-deck-shape__image', alt: '', src: `${mediaBase}?ref=${encodeURIComponent(src)}`,
    });
    // A part the media route cannot serve leaves the labelled placeholder.
    img.addEventListener('load', () => el.replaceChildren(img, ...el.querySelectorAll('[data-frame], [data-remove]')), { once: true });
    img.addEventListener('error', () => img.remove(), { once: true });
  }
  return el;
};

let mediaBase = null;

export const setDeckMediaBase = (documentApi) => {
  mediaBase = documentApi ? `${documentApi}/media` : null;
};

const removeButton = () => {
  const el = document.createElement('button');
  el.type = 'button';
  el.className = 'sp-deck-shape__remove';
  el.dataset.remove = 'shape';
  el.title = 'Delete this shape';
  el.setAttribute('aria-label', 'Delete this shape');
  el.textContent = '×';
  return el;
};

export const createStage = (host, { session, onSelect, onEdit }) => {
  const stage = document.createElement('div');
  stage.className = 'sp-deck-stage';
  host.append(stage);
  const views = new Map();
  let slide = null;

  const shapeEl = (shape) => {
    if (lockedKind(shape)) {
      const el = lockedEl(shape);
      if (session.state.mode === 'edit') el.append(grip('move', 'Drag to move'), grip('resize', 'Drag to resize'), removeButton());
      views.set(shape.get('id'), { el, view: null });
      return el;
    }
    const el = document.createElement('div');
    el.className = `sp-deck-shape${isTitle(shape) ? ' sp-deck-shape--title' : ''}`;
    el.dataset.shape = shape.get('id');
    const text = document.createElement('div');
    el.append(grip('move', 'Drag to move'), text, grip('resize', 'Drag to resize'));
    if (session.state.mode === 'edit') el.append(removeButton());
    const view = mountShapeText(text, {
      shape, session, onEdit, onSelect: (v) => onSelect(shape.get('id'), v),
    });
    views.set(shape.get('id'), { el, view });
    return el;
  };

  const show = (next) => {
    slide = next;
    const shapes = next ? shapesOf(next) : [];
    const ids = new Set(shapes.map((s) => s.get('id')));
    for (const [id, { el, view }] of views) {
      if (ids.has(id)) continue;
      view?.destroy();
      el.remove();
      views.delete(id);
    }
    shapes.forEach((shape) => {
      const known = views.get(shape.get('id'));
      const el = known ? known.el : shapeEl(shape);
      known?.view?.refresh(shape);
      placeFrame(el, frameOf(shape));
      if (el.parentNode !== stage) stage.append(el);
    });
    stage.dataset.slide = next?.get('id') ?? '';
  };

  wireFrames(stage, {
    session,
    onEdit,
    shapeFor: (id) => shapesOf(slide).find((s) => s.get('id') === id) ?? null,
  });

  const removeShape = (id) => {
    const shapes = slide?.get('shapes');
    const at = shapes ? indexOfId(shapes, id) : -1;
    if (at < 0 || session.state.mode !== 'edit') return;
    shapes.delete(at, 1);
    session.state.doc.commit();
    show(slide);
    onEdit?.();
  };

  stage.addEventListener('click', (event) => {
    const button = event.target.closest('[data-remove="shape"]');
    if (button) removeShape(button.closest('[data-shape]').dataset.shape);
  });

  return {
    stage,
    show,
    current: () => slide,
    shapeEl: (id) => views.get(id)?.el ?? null,
    focusFirst: () => [...views.values()].find((v) => v.view)?.view.view.focus(),
  };
};
