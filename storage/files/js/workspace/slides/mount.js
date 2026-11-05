// The deck editor for PowerPoint and OpenDocument presentations: a filmstrip
// and a stage over the live session's Loro replica, synced through the same
// collab updates and live channel as the doc editor. Below edit access the
// session joins in comment or view mode and nothing is editable.
import { loadEngine } from '../doc/loro.js';
import { createSession } from '../doc/session.js';
import {
  SLIDE_H, SLIDE_W, frameOf, indexOfId, insertImage, insertTextBox, itemsOf, setFrame, shapesOf, slideList,
} from './deck.js';
import { readParagraphs, writeParagraphs } from './paragraphs.js';
import { createStage, rememberLocalImage, setDeckMediaBase } from './stage.js';
import { uploadFile } from '../upload/session.js';
import { showToast } from '../../services/toast.js';
import { createFilmstrip } from './filmstrip.js';
import { createCursors } from './cursors.js';
import { present } from './present.js';

const FILM_IDLE_MS = 400;

// A re-join hands back the edits the last session never sent, merged over the
// new snapshot. Core accepts only this join's peer, so each carried shape's
// text and frame is written again into the new replica rather than imported:
// without this, keystrokes typed while a post was refused vanish.
const carryOver = (doc, carried) => {
  const slides = slideList(doc);
  for (const slide of itemsOf(slideList(carried))) {
    // Shape ids are unique only within their slide, so a shape is matched
    // under the slide with the same id, never across the deck.
    const at = indexOfId(slides, slide.get('id'));
    const here = at >= 0 ? shapesOf(slides.get(at)) : [];
    for (const was of shapesOf(slide)) {
      const shape = here.find((s) => s.get('id') === was.get('id'));
      if (!shape) continue;
      setFrame(shape, frameOf(was));
      const list = shape.get('paragraphs');
      if (list && was.get('paragraphs')) writeParagraphs(list, readParagraphs(was.get('paragraphs')), new Map());
    }
  }
  doc.commit();
};

const IMAGE_TYPES = ['image/png', 'image/jpeg', 'image/gif'];
const PICKABLE = [...IMAGE_TYPES, 'image/webp'];

// Office releases do not all open WebP inside a pptx, so it is saved as PNG.
const placeable = async (file) => {
  if (IMAGE_TYPES.includes(file.type)) return file;
  const bitmap = await createImageBitmap(file);
  const canvas = Object.assign(document.createElement('canvas'), { width: bitmap.width, height: bitmap.height });
  canvas.getContext('2d').drawImage(bitmap, 0, 0);
  bitmap.close();
  const blob = await new Promise((resolve) => { canvas.toBlob(resolve, 'image/png'); });
  if (!blob) throw new Error('The picture could not be converted.');
  return new File([blob], `${file.name.replace(/\.[^.]+$/, '')}.png`, { type: 'image/png' });
};

const fittedFrame = async (file) => {
  const bitmap = await createImageBitmap(file);
  const scale = Math.min((SLIDE_W / 2) / bitmap.width, (SLIDE_H / 2) / bitmap.height);
  const w = bitmap.width * scale;
  const h = bitmap.height * scale;
  bitmap.close();
  return { x: (SLIDE_W - w) / 2, y: (SLIDE_H - h) / 2, w, h };
};

const pickImage = () => new Promise((resolve) => {
  const input = Object.assign(document.createElement('input'), { type: 'file', accept: PICKABLE.join(','), hidden: true });
  input.dataset.deckImage = '';
  input.addEventListener('change', () => { input.remove(); resolve(input.files[0] ?? null); }, { once: true });
  input.addEventListener('cancel', () => { input.remove(); resolve(null); }, { once: true });
  document.body.append(input);
  input.click();
});

const el = (tag, className) => Object.assign(document.createElement(tag), { className });

export const mount = async (slot, { root, readOnly }) => {
  await loadEngine();
  const api = root.dataset.documentApi;
  setDeckMediaBase(api);
  const saveLabel = root.querySelector('[data-field="save"]');
  const ui = { stage: null, film: null, cursors: null, index: 0 };

  const anchor = (slide, index) => slot.dispatchEvent(new CustomEvent('ws:anchor', {
    bubbles: true, detail: { node: slide.get('id'), label: `Slide ${index + 1}` },
  }));

  const pick = (index) => {
    const slides = itemsOf(slideList(session.state.doc));
    ui.index = Math.min(Math.max(index, 0), Math.max(slides.length - 1, 0));
    const slide = slides[ui.index] ?? null;
    ui.stage.show(slide);
    ui.film.render(ui.index);
    if (slide) anchor(slide, ui.index);
    ui.cursors.draw();
  };

  const redraw = () => {
    if (!ui.stage) return;
    const id = ui.stage.current()?.get('id');
    const at = id ? indexOfId(slideList(session.state.doc), id) : -1;
    pick(at >= 0 ? at : ui.index);
  };

  const session = createSession({
    mode: readOnly ? 'comment' : 'edit',
    collabApi: `${api}/collab`,
    liveApi: `${api}/live`,
    onState: (text) => { saveLabel.textContent = text; },
    onReset: (s, carried) => {
      if (carried) carryOver(s.doc, carried);
      redraw();
    },
    onRemote: redraw,
    onEvent: (name, data) => { if (name === 'document.awareness') ui.cursors?.receive(data); },
  });
  await session.start();

  const shell = el('div', 'sp-deck');
  const film = el('nav', 'sp-deck-film');
  const desk = el('div', 'sp-deck-desk');
  shell.append(film, desk);
  slot.replaceChildren(shell);

  ui.cursors = createCursors({
    api: `${api}/collab/awareness`, session, selfId: root.dataset.userId, shapeEl: (id) => ui.stage?.shapeEl(id),
  });
  // Typing commits per keystroke, so the filmstrip follows local edits on a
  // pause rather than redrawing every thumbnail per character.
  let filmTimer = 0;
  const refreshFilm = () => {
    clearTimeout(filmTimer);
    filmTimer = setTimeout(() => ui.film?.render(ui.index), FILM_IDLE_MS);
  };
  ui.stage = createStage(desk, {
    session,
    onEdit: refreshFilm,
    onSelect: (shapeId, view) => {
      const shape = itemsOf(ui.stage.current()?.get('shapes')).find((s) => s.get('id') === shapeId);
      if (shape) ui.cursors.moved(shape, view);
    },
  });
  ui.film = createFilmstrip(film, { session, onPick: pick, onPresent: (at) => present(session, at) });
  ui.film.actions.text = () => {
    const slide = ui.stage.current();
    if (!slide) return;
    insertTextBox(slide);
    session.state.doc.commit();
    redraw();
  };
  ui.film.actions.image = async () => {
    const picked = await pickImage();
    const slide = ui.stage.current();
    if (!picked || !slide) return;
    if (!PICKABLE.includes(picked.type)) {
      showToast('A slide holds PNG, JPEG, GIF or WebP pictures.', 'error');
      return;
    }
    try {
      const file = await placeable(picked);
      const frame = await fittedFrame(file);
      const { sha256 } = await uploadFile(file, { mime: file.type });
      rememberLocalImage(sha256, URL.createObjectURL(file));
      insertImage(slide, sha256, frame);
      session.state.doc.commit();
      redraw();
    } catch (error) {
      showToast(error.message ?? 'The picture could not be added.', 'error');
    }
  };
  root.addEventListener('ws:reveal', (event) => {
    const at = indexOfId(slideList(session.state.doc), event.detail?.node ?? '');
    if (at >= 0) pick(at);
  });
  pick(0);
  return {
    readOnly: session.state.mode !== 'edit',
    hint: 'Click a shape to edit its text; drag its grip to move, its corner to resize. Alt+Arrow reorders slides.',
  };
};
