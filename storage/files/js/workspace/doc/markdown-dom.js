// Draws the tree markdown.js reads from a comment body. Every element comes
// from the fixed table below and every piece of text from the body goes in as
// a text node or `textContent`, so nothing a comment says is ever parsed as
// markup. `nameOf` gives the current name for a mention's user id.
const BLOCKS = {
  p: ['p', 'sp-doc-md__block'],
  ul: ['ul', 'sp-doc-md__list'],
  ol: ['ol', 'sp-doc-md__list'],
  li: ['li', ''],
  strong: ['strong', ''],
  em: ['em', ''],
  a: ['a', 'sp-doc-md__link'],
};

export const buildBody = (blocks, nameOf, doc = document) => {
  const make = (tag, className, text) => {
    const node = doc.createElement(tag);
    if (className) node.className = className;
    if (text != null) node.textContent = text;
    return node;
  };

  const lines = (value) => value.split('\n').flatMap((line, at) => (at ? [make('br', ''), doc.createTextNode(line)] : [doc.createTextNode(line)]));

  const draw = (node) => {
    if (node.t === 'text') return lines(node.v);
    if (node.t === 'code') return [make('code', 'sp-doc-md__code', node.v)];
    if (node.t === 'mention') return [chip(node)];
    if (node.t === 'pre') return [preformatted(node)];
    const [tag, className] = BLOCKS[node.t];
    const element = make(tag, className);
    if (node.t === 'a') {
      element.href = node.href;
      element.target = '_blank';
      element.rel = 'noopener noreferrer nofollow';
    }
    element.append(...node.c.flatMap(draw));
    return [element];
  };

  const chip = (node) => {
    const name = node.id ? nameOf(node.id) : node.label;
    const span = make('span', 'sp-doc-mention', `@${name === node.id ? node.label : name}`);
    if (node.id) span.dataset.userId = node.id;
    return span;
  };

  const preformatted = (node) => {
    const pre = make('pre', 'sp-doc-md__pre');
    pre.append(make('code', '', node.v));
    return pre;
  };

  const fragment = doc.createDocumentFragment();
  fragment.append(...blocks.flatMap(draw));
  return fragment;
};
