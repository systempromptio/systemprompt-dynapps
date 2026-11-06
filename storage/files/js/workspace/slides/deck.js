// The deck replica's shape (core's contract): root `slides` MovableList of
// slide Maps holding id, layout and shapes. Each shape holds id, kind, a frame
// Map of x/y/w/h in EMU, and a paragraphs MovableList of rich block Maps. Frames are written per
// property, so two people moving and resizing one shape both land.
import { LoroMap, LoroMovableList, LoroText } from 'loro-crdt';
import { newBlock } from '../doc/loro.js';
import { freshId } from '../doc/ids.js';

export const SLIDE_W = 12192000;
export const SLIDE_H = 6858000;
const FRAME_KEYS = ['x', 'y', 'w', 'h'];

export const slideList = (doc) => doc.getMovableList('slides');

export const itemsOf = (list) => {
  const out = [];
  for (let i = 0; i < (list?.length ?? 0); i += 1) out.push(list.get(i));
  return out;
};

export const shapesOf = (slide) => itemsOf(slide.get('shapes'));

export const frameOf = (shape) => {
  const frame = shape.get('frame');
  const read = (key, fallback) => Number(frame?.get(key) ?? fallback);
  return { x: read('x', 0), y: read('y', 0), w: read('w', SLIDE_W), h: read('h', SLIDE_H / 4) };
};

export const setFrame = (shape, next) => {
  let frame = shape.get('frame');
  if (!(frame instanceof LoroMap)) frame = shape.ensureMergeableMap('frame');
  for (const key of FRAME_KEYS) {
    const value = Math.round(next[key]);
    if (Number(frame.get(key)) !== value) frame.set(key, value);
  }
};

export const isTitle = (shape) => {
  const kind = shape.get('kind');
  const text = typeof kind === 'string' ? kind : JSON.stringify(kind ?? '');
  return /title|ctrTitle/i.test(text);
};

export const indexOfId = (list, id) => itemsOf(list).findIndex((m) => m.get('id') === id);

export const findShape = (doc, shapeId) => {
  for (const slide of itemsOf(slideList(doc))) {
    const shape = shapesOf(slide).find((s) => s.get('id') === shapeId);
    if (shape) return shape;
  }
  return null;
};

const addShape = (shapes, kind, frame, words) => {
  const shape = shapes.insertContainer(shapes.length, new LoroMap());
  shape.set('id', freshId());
  shape.set('kind', kind);
  setFrame(shape, frame);
  const paras = shape.setContainer('paragraphs', new LoroMovableList());
  const block = newBlock(paras, 0, freshId(), { type: 'paragraph' });
  block.get('text').insert(0, words);
};

export const insertSlide = (doc, at) => {
  const slide = slideList(doc).insertContainer(at, new LoroMap());
  slide.set('id', freshId());
  slide.set('layout', 'titleAndContent');
  const shapes = slide.setContainer('shapes', new LoroMovableList());
  addShape(shapes, 'title', { x: 838200, y: 365125, w: 10515600, h: 1325563 }, 'New slide');
  addShape(shapes, 'body', { x: 838200, y: 1825625, w: 10515600, h: 4351338 }, 'First point');
  return slide;
};

export const insertTextBox = (slide) => addShape(slide.get('shapes'), 'text', {
  x: SLIDE_W / 4, y: SLIDE_H / 3, w: SLIDE_W / 2, h: SLIDE_H / 6,
}, 'Text');

// A picture's `src` is its upload's hash until a checkpoint writes the part.
export const insertImage = (slide, sha256, frame) => {
  const shapes = slide.get('shapes');
  const shape = shapes.insertContainer(shapes.length, new LoroMap());
  shape.set('id', freshId());
  shape.set('kind', 'image');
  shape.set('src', sha256);
  setFrame(shape, frame);
  shape.setContainer('paragraphs', new LoroMovableList());
};

const copyBlock = (list, at, block) => {
  const kind = block.get('kind');
  const copy = newBlock(list, at, freshId(), JSON.parse(kind ?? '{"type":"paragraph"}'));
  const text = block.get('text');
  if (text instanceof LoroText && text.length) copy.get('text').applyDelta(text.toDelta());
};

export const duplicateSlide = (doc, index) => {
  const list = slideList(doc);
  const source = list.get(index);
  const slide = list.insertContainer(index + 1, new LoroMap());
  slide.set('id', freshId());
  slide.set('layout', source.get('layout') ?? 'blank');
  const shapes = slide.setContainer('shapes', new LoroMovableList());
  for (const shape of shapesOf(source)) {
    const copy = shapes.insertContainer(shapes.length, new LoroMap());
    copy.set('id', freshId());
    copy.set('kind', shape.get('kind') ?? 'text');
    setFrame(copy, frameOf(shape));
    const paras = copy.setContainer('paragraphs', new LoroMovableList());
    itemsOf(shape.get('paragraphs')).forEach((block, i) => copyBlock(paras, i, block));
  }
  return slide;
};
