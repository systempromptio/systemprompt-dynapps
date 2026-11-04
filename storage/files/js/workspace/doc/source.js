// The Markdown source view: the live document written out as Markdown,
// read-only. Edits happen in the rich view, so both views always show the
// same session state; the checkpoint writes the file itself on the server.
const WRAP = { bold: '**', italic: '_', strike: '~~', code: '`' };

const inline = (node) => {
  let out = '';
  node.forEach((child) => {
    if (!child.isText) {
      out += '  \n';
      return;
    }
    let text = child.text;
    for (const mark of child.marks) {
      const wrap = WRAP[mark.type.name];
      if (wrap) text = `${wrap}${text}${wrap}`;
      if (mark.type.name === 'link') text = `[${text}](${mark.attrs.href})`;
    }
    out += text;
  });
  return out;
};

const cellMarkdown = (cell) => cell.textContent.replace(/\|/g, '\\|').replace(/\n/g, ' ');

const tableMarkdown = (table) => {
  const rows = [];
  table.forEach((row) => {
    const cells = [];
    row.forEach((cell) => cells.push(cellMarkdown(cell)));
    rows.push(`| ${cells.join(' | ')} |`);
  });
  const width = table.firstChild?.childCount ?? 1;
  rows.splice(1, 0, `|${' --- |'.repeat(width)}`);
  return rows.join('\n');
};

const line = (node, counters) => {
  const { attrs } = node;
  switch (node.type.name) {
    case 'heading': return `${'#'.repeat(attrs.level)} ${inline(node)}`;
    case 'list_item': {
      const pad = '  '.repeat(Math.max(0, attrs.level - 1));
      if (!attrs.ordered) return `${pad}- ${inline(node)}`;
      counters[attrs.level] = (counters[attrs.level] ?? 0) + 1;
      return `${pad}${counters[attrs.level]}. ${inline(node)}`;
    }
    case 'quote': return `> ${inline(node)}`;
    case 'code_block': return `\`\`\`${attrs.language ?? ''}\n${node.textContent}\n\`\`\``;
    case 'divider': return '---';
    case 'page_break': return '<div style="page-break-after: always"></div>';
    case 'table': return tableMarkdown(node);
    case 'opaque': {
      const kind = JSON.parse(attrs.kind);
      return kind.type === 'raw' ? kind.source : `<!-- ${attrs.label} -->`;
    }
    default: return inline(node);
  }
};

export const toMarkdown = (doc) => {
  const out = [];
  let counters = {};
  let previous = null;
  doc.forEach((node) => {
    if (node.type.name !== 'list_item') counters = {};
    const tight = previous === 'list_item' && node.type.name === 'list_item';
    if (out.length) out.push(tight ? '\n' : '\n\n');
    out.push(line(node, counters));
    previous = node.type.name;
  });
  return `${out.join('')}\n`;
};
