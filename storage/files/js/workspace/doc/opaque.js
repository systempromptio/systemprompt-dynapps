// The DOM for a block the editor shows but never edits: an image is drawn
// from the document's media route, which serves only what the head holds,
// so an image placed since the last checkpoint shows its alt until then,
// and anything else (and a table not yet upgraded to rows) keeps a label.
let mediaBase = null;

export const setMediaBase = (documentApi) => {
  mediaBase = documentApi ? `${documentApi}/media` : null;
};
export const opaqueLabel = (kind) => {
  switch (kind.type) {
    case 'table': return `Table (${kind.rows?.length ?? 0} rows)`;
    case 'image': return kind.alt ? `Image: ${kind.alt}` : 'Image';
    default: return 'Formatted content kept as-is';
  }
};

const el = (tag, className, text) => {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text != null) node.textContent = text;
  return node;
};

const imageDom = (kind) => {
  const figure = el('figure', 'sp-doc__figure');
  const img = el('img', 'sp-doc__image');
  img.alt = kind.alt ?? '';
  img.loading = 'lazy';
  img.src = /^https?:/i.test(kind.blob) ? kind.blob : `${mediaBase}?ref=${encodeURIComponent(kind.blob)}`;
  img.addEventListener('error', () => figure.replaceChildren(el('div', 'sp-doc__opaque', opaqueLabel(kind))), { once: true });
  figure.append(img);
  return figure;
};

export const opaqueDom = (kind, label) => {
  if (kind.type === 'image' && kind.blob && mediaBase) {
    const dom = imageDom(kind);
    dom.setAttribute('contenteditable', 'false');
    return dom;
  }
  const dom = el('div', 'sp-doc__opaque', label || opaqueLabel(kind));
  dom.setAttribute('contenteditable', 'false');
  return dom;
};

export const readKindAttr = (attrs) => {
  try {
    return JSON.parse(attrs.kind || '{}');
  } catch {
    return { type: 'raw' };
  }
};
