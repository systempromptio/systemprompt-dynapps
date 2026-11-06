// Present mode: the deck full screen, one slide at a time, read from the
// live replica so an edit made while presenting shows on the next slide.
// Arrows, Space and clicks advance; Escape (or leaving full screen) ends it.
// Text is drawn through the doc schema, so bold, links and colour show as
// they do on the stage.
import { DOMSerializer } from 'prosemirror-model';
import { schema } from '../doc/schema.js';
import { frameOf, isTitle, shapesOf, slideList } from './deck.js';
import { placeFrame } from './frames.js';
import { readParagraphs } from './paragraphs.js';
import { lockedKind } from './stage.js';

const serializer = DOMSerializer.fromSchema(schema);

const slideView = (slide) => {
  const card = document.createElement('div');
  card.className = 'sp-deck-present__slide';
  for (const shape of shapesOf(slide)) {
    const box = document.createElement('div');
    box.className = `sp-deck-present__shape${isTitle(shape) ? ' sp-deck-present__shape--title' : ''}`;
    if (lockedKind(shape)) box.textContent = lockedKind(shape);
    else box.append(serializer.serializeFragment(readParagraphs(shape.get('paragraphs')).content));
    placeFrame(box, frameOf(shape));
    card.append(box);
  }
  return card;
};

export const present = async (session, start) => {
  const overlay = document.createElement('div');
  overlay.className = 'sp-deck-present';
  overlay.tabIndex = -1;
  overlay.setAttribute('role', 'dialog');
  overlay.setAttribute('aria-label', 'Presentation');
  const opener = document.activeElement;
  document.body.append(overlay);
  let at = start;
  const total = () => slideList(session.state.doc).length;
  const draw = () => {
    const slide = slideList(session.state.doc).get(at);
    overlay.replaceChildren(slide ? slideView(slide) : '');
  };
  const close = () => {
    overlay.remove();
    if (document.fullscreenElement) document.exitFullscreen().catch(() => undefined);
    opener?.focus?.();
  };
  const go = (step) => {
    at = Math.min(Math.max(at + step, 0), total() - 1);
    draw();
  };
  overlay.addEventListener('keydown', (event) => {
    const step = { ArrowRight: 1, ArrowDown: 1, ' ': 1, PageDown: 1, ArrowLeft: -1, ArrowUp: -1, PageUp: -1 }[event.key];
    if (event.key === 'Escape') close();
    else if (step) go(step);
    else return;
    event.preventDefault();
  });
  overlay.addEventListener('click', () => go(1));
  overlay.addEventListener('fullscreenchange', () => { if (!document.fullscreenElement) close(); });
  draw();
  overlay.focus();
  try {
    await overlay.requestFullscreen();
  } catch {
    overlay.classList.add('is-windowed');
  }
};
