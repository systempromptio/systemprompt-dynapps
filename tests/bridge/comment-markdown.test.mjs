// A comment body in the web Drive's threads: the Markdown reader draws a
// small safe subset, a mention stays one mention, and nothing a body says is
// ever markup. The DOM is a minimal recorder, enough to see which elements
// and text nodes the builder makes.
import test from 'node:test';
import assert from 'node:assert/strict';
import { buildBody } from '../../storage/files/js/workspace/doc/markdown-dom.js';
import { isSystemLine, parseBody } from '../../storage/files/js/workspace/doc/markdown.js';
import { mentionIds, mentionToken } from '../../storage/files/js/workspace/doc/mention-token.js';

class Text {
  constructor(data) { this.data = data; }
  get html() { return this.data.replace(/&/g, '&amp;').replace(/</g, '&lt;'); }
}

class Element {
  constructor(tag) {
    this.tag = tag;
    this.children = [];
    this.dataset = {};
    this.className = '';
  }

  append(...nodes) { this.children.push(...nodes); }

  set textContent(value) { this.children = [new Text(value)]; }

  get html() {
    const attrs = [this.className && `class="${this.className}"`, this.href && `href="${this.href}"`, this.rel && `rel="${this.rel}"`, this.dataset.userId && `user="${this.dataset.userId}"`]
      .filter(Boolean).join(' ');
    return `<${this.tag}${attrs ? ` ${attrs}` : ''}>${this.children.map((c) => c.html).join('')}</${this.tag}>`;
  }
}

const recorder = () => {
  const made = [];
  const doc = {
    made,
    createElement: (tag) => { made.push(tag); return new Element(tag); },
    createTextNode: (data) => new Text(data),
    createDocumentFragment: () => new Element('#fragment'),
  };
  return doc;
};

const names = { u1: 'Ana Ruiz' };
const nameOf = (id) => names[id] ?? id;
const draw = (body) => {
  const doc = recorder();
  const fragment = buildBody(parseBody(body), nameOf, doc);
  return { html: fragment.children.map((c) => c.html).join(''), made: doc.made };
};
const plain = (body) => draw(body).html.replace(/ (class|rel|user)="[^"]*"/g, '');

test('bold, italic and inline code are drawn', () => {
  assert.equal(plain('a **bold** and __also__ and *it* and _too_ and `x < y`'),
    '<p>a <strong>bold</strong> and <strong>also</strong> and <em>it</em> and <em>too</em> and <code>x &lt; y</code></p>');
});

test('bullet and numbered lists, and a fenced block, are drawn', () => {
  assert.equal(plain('- one\n- **two**\n\n1. first\n2) second'), '<ul><li>one</li><li><strong>two</strong></li></ul><ol><li>first</li><li>second</li></ol>');
  assert.equal(plain('before\n```js\nlet a = 1 < 2;\n**kept**\n```\nafter'),
    '<p>before</p><pre><code>let a = 1 &lt; 2;\n**kept**</code></pre><p>after</p>');
});

test('an unclosed fence runs to the end and a line break stays a break', () => {
  assert.equal(plain('```\nopen\nstill'), '<pre><code>open\nstill</code></pre>');
  assert.equal(plain('one\ntwo'), '<p>one<br></br>two</p>');
});

test('an https link is drawn with a safe rel, other schemes stay text', () => {
  const link = draw('see [the doc](https://example.com/a?b=1) and https://example.org/x.');
  assert.match(link.html, /<a class="sp-doc-md__link" href="https:\/\/example\.com\/a\?b=1" rel="noopener noreferrer nofollow">the doc<\/a>/);
  assert.match(link.html, /<a [^>]*href="https:\/\/example\.org\/x"[^>]*>https:\/\/example\.org\/x<\/a>\.<\/p>/);
  for (const body of ['[x](javascript:alert(1))', '[x](http://example.com)', '[x](data:text/html,hi)', '[x](//evil.test)']) {
    assert.ok(!draw(body).html.includes('<a '), body);
  }
});

test('HTML in a body stays text and makes no element', () => {
  const body = '<script>alert(1)</script> <img src=x onerror=alert(1)> [<b>x</b>](https://a.example/)';
  const drawn = draw(body);
  assert.deepEqual([...new Set(drawn.made)].sort(), ['a', 'p']);
  assert.ok(drawn.html.includes('&lt;script>alert(1)&lt;/script>'));
  assert.ok(drawn.html.includes('&lt;b>x&lt;/b></a>'));
  assert.deepEqual(parseBody('<script>x</script>'), [{ t: 'p', c: [{ t: 'text', v: '<script>x</script>' }] }]);
});

test('every element made comes from the fixed set', () => {
  const body = '**a** *b* `c` [d](https://e.example) @[Ana Ruiz](user:u1)\n\n- x\n\n1. y\n\n```\nz\n```';
  const allowed = new Set(['p', 'strong', 'em', 'code', 'a', 'span', 'ul', 'ol', 'li', 'pre', 'br']);
  for (const tag of draw(body).made) assert.ok(allowed.has(tag), tag);
});

test('a mention token is one chip with the current name, inside other marks too', () => {
  assert.equal(plain('hi @[Ana Ruiz](user:u1), read this'), '<p>hi <span>@Ana Ruiz</span>, read this</p>');
  assert.match(draw('**@[Ana Ruiz](user:u1)**').html, /<strong><span class="sp-doc-mention" user="u1">@Ana Ruiz<\/span><\/strong>/);
  assert.match(draw('- ask @[Bea](user:u2)').html, /<li>ask <span class="sp-doc-mention" user="u2">@Bea<\/span><\/li>/);
});

test('a mention written as an email is one mention, never split at the dots', () => {
  const label = 'ana.ruiz_dev@dynapps.es';
  assert.equal(plain(`@[${label}](user:u9) and more`), `<p><span>@${label}</span> and more</p>`);
  assert.equal(plain('thanks @ana.ruiz_dev@dynapps.es.'), '<p>thanks <span>@ana.ruiz_dev@dynapps.es</span>.</p>');
  assert.deepEqual(parseBody('@ana.ruiz@dynapps.es')[0].c, [{ t: 'mention', label: 'ana.ruiz@dynapps.es', id: null }]);
  assert.equal(plain('write to ana_ruiz@dynapps.es now'), '<p>write to ana_ruiz@dynapps.es now</p>');
});

test('words with marks inside them and lone markers stay as typed', () => {
  assert.equal(plain('snake_case_name and 2 * 3 * 4 and a*b'), '<p>snake_case_name and 2 * 3 * 4 and a*b</p>');
  assert.equal(plain('**open and `open'), '<p>**open and `open</p>');
});

test('a hostile body is bounded and does not hang', () => {
  const started = Date.now();
  for (const body of ['*a '.repeat(2000), '['.repeat(4000), '`'.repeat(4000), '**'.repeat(2000), `${'*_'.repeat(30)}x${'_*'.repeat(30)}`]) parseBody(body);
  assert.ok(Date.now() - started < 2000);
  const depth = JSON.stringify(parseBody(`${'**a *b '.repeat(8)}x${' b* a**'.repeat(8)}`));
  assert.ok(depth.split('"t":"strong"').length < 8);
});

test('the claim reply is a system line, other bodies are not', () => {
  assert.ok(isSystemLine('Dynwork picked this up.'));
  assert.ok(isSystemLine('  Dynwork picked this up.\n'));
  assert.ok(!isSystemLine('Dynwork picked this up. Thanks'));
});

test('the mention token grammar the composer writes round-trips', () => {
  const token = mentionToken({ user_id: 'u1', display_name: 'Ana (QA) [x]' });
  assert.equal(token, '@[Ana  QA   x](user:u1)');
  assert.deepEqual(mentionIds(`${token} and ${token} @[B](user:u2)`), ['u1', 'u2']);
});
