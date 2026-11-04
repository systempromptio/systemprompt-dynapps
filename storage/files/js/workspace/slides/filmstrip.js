// The filmstrip: one thumbnail per slide (its title and first lines), the
// slide commands, and drag or Alt+Arrow to reorder. Every command is one
// MovableList op, so a concurrent reorder elsewhere merges instead of duplicating slides.
import { duplicateSlide, frameOf, insertSlide, isTitle, itemsOf, shapesOf, slideList } from './deck.js';
import { placeFrame } from './frames.js';
import { lockedKind } from './stage.js';

const thumb = (slide, number) => {
  const item = document.createElement('li');
  item.className = 'sp-deck-film__item';
  item.dataset.slide = slide.get('id');
  item.draggable = true;
  item.tabIndex = 0;
  item.setAttribute('role', 'option');
  const badge = document.createElement('span');
  badge.className = 'sp-deck-film__number';
  badge.textContent = String(number);
  const card = document.createElement('div');
  card.className = 'sp-deck-film__card';
  for (const shape of shapesOf(slide)) {
    const box = document.createElement('div');
    box.className = `sp-deck-film__shape${isTitle(shape) ? ' sp-deck-film__shape--title' : ''}`;
    box.textContent = lockedKind(shape)
      ?? itemsOf(shape.get('paragraphs')).map((p) => p.get('text')?.toString() ?? '').join('\n');
    placeFrame(box, frameOf(shape));
    card.append(box);
  }
  item.append(badge, card);
  return item;
};

const button = (action, label) => {
  const el = document.createElement('button');
  el.type = 'button';
  el.className = 'sp-btn sp-btn--xs sp-btn--ghost';
  el.dataset.action = action;
  el.textContent = label;
  return el;
};

export const createFilmstrip = (host, { session, onPick, onPresent }) => {
  const bar = document.createElement('div');
  bar.className = 'sp-deck-film__bar';
  const editing = session.state.mode === 'edit';
  if (editing) bar.append(button('add', '+ Slide'), button('duplicate', 'Duplicate'), button('delete', 'Delete'), button('text', '+ Text'));
  bar.append(button('present', 'Present'));
  const list = document.createElement('ol');
  list.className = 'sp-deck-film__list';
  list.setAttribute('role', 'listbox');
  list.setAttribute('aria-label', 'Slides');
  host.append(bar, list);
  let current = 0;
  let dragFrom = -1;

  const doc = () => session.state.doc;
  const commit = (index) => {
    doc().commit();
    onPick(index);
  };
  const move = (from, to) => {
    const slides = slideList(doc());
    if (!editing || from === to || to < 0 || to >= slides.length) return;
    slides.move(from, to);
    commit(to);
  };
  const actions = {
    add: () => { insertSlide(doc(), current + 1); commit(current + 1); },
    duplicate: () => { duplicateSlide(doc(), current); commit(current + 1); },
    delete: () => {
      const slides = slideList(doc());
      if (slides.length <= 1) return;
      slides.delete(current, 1);
      commit(Math.min(current, slides.length - 1));
    },
    present: () => onPresent(current),
  };

  bar.addEventListener('click', (event) => actions[event.target.closest('[data-action]')?.dataset.action]?.());
  const indexOf = (event) => [...list.children].indexOf(event.target.closest('[data-slide]'));
  list.addEventListener('click', (event) => { if (indexOf(event) >= 0) onPick(indexOf(event)); });
  list.addEventListener('keydown', (event) => {
    const at = indexOf(event);
    if (at < 0) return;
    const step = { ArrowUp: -1, ArrowDown: 1 }[event.key];
    if (event.key === 'Enter' || event.key === ' ') onPick(at);
    if (!step) return;
    event.preventDefault();
    if (event.altKey) move(at, at + step);
    else onPick(Math.min(Math.max(at + step, 0), list.children.length - 1));
  });
  list.addEventListener('dragstart', (event) => { dragFrom = indexOf(event); });
  list.addEventListener('dragover', (event) => event.preventDefault());
  list.addEventListener('drop', (event) => {
    event.preventDefault();
    if (dragFrom >= 0 && indexOf(event) >= 0) move(dragFrom, indexOf(event));
    dragFrom = -1;
  });

  return {
    actions,
    render: (index) => {
      current = index;
      const items = itemsOf(slideList(doc())).map((slide, i) => thumb(slide, i + 1));
      items[index]?.classList.add('is-active');
      items[index]?.setAttribute('aria-selected', 'true');
      list.replaceChildren(...items);
      list.children[index]?.scrollIntoView({ block: 'nearest' });
    },
  };
};
