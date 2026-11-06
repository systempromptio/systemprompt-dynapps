// Dragging a shape's grip moves it; dragging its corner resizes it. The DOM
// follows the pointer and the frame is written once on release, per
// property, so a concurrent edit to another property survives.
import { SLIDE_W, SLIDE_H, frameOf, setFrame } from './deck.js';

const MIN = 228600;

export const placeFrame = (el, { x, y, w, h }) => {
  el.style.setProperty('--sp-deck-x', `${(x / SLIDE_W) * 100}%`);
  el.style.setProperty('--sp-deck-y', `${(y / SLIDE_H) * 100}%`);
  el.style.setProperty('--sp-deck-w', `${(w / SLIDE_W) * 100}%`);
  el.style.setProperty('--sp-deck-h', `${(h / SLIDE_H) * 100}%`);
};

const clamp = (value, lo, hi) => Math.min(Math.max(value, lo), hi);

const nextFrame = (start, mode, dx, dy) => {
  if (mode === 'move') {
    return { ...start, x: clamp(start.x + dx, 0, SLIDE_W - start.w), y: clamp(start.y + dy, 0, SLIDE_H - start.h) };
  }
  return { ...start, w: clamp(start.w + dx, MIN, SLIDE_W - start.x), h: clamp(start.h + dy, MIN, SLIDE_H - start.y) };
};

export const wireFrames = (stage, { session, shapeFor, onEdit }) => {
  stage.addEventListener('pointerdown', (event) => {
    const handle = event.target.closest('[data-frame]');
    const el = handle?.closest('[data-shape]');
    if (!el || session.state.mode !== 'edit') return;
    event.preventDefault();
    const shape = shapeFor(el.dataset.shape);
    if (!shape) return;
    const start = frameOf(shape);
    const box = stage.getBoundingClientRect();
    const scale = { x: SLIDE_W / box.width, y: SLIDE_H / box.height };
    const origin = { x: event.clientX, y: event.clientY };
    let frame = start;
    handle.setPointerCapture(event.pointerId);
    el.classList.add('is-dragging');
    const move = (e) => {
      frame = nextFrame(start, handle.dataset.frame, (e.clientX - origin.x) * scale.x, (e.clientY - origin.y) * scale.y);
      placeFrame(el, frame);
    };
    const up = () => {
      handle.removeEventListener('pointermove', move);
      el.classList.remove('is-dragging');
      if (frame === start) return;
      setFrame(shape, frame);
      session.state.doc.commit();
      onEdit?.();
    };
    handle.addEventListener('pointermove', move);
    handle.addEventListener('pointerup', up, { once: true });
    handle.addEventListener('pointercancel', up, { once: true });
  });
};
