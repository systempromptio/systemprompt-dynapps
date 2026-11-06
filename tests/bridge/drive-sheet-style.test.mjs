import test from 'node:test';
import assert from 'node:assert/strict';
import { cellLook } from '../../storage/files/js/workspace/sheet/style.js';

test('a resolved fill, ink and font marks reach the grid', () => {
  const look = cellLook(JSON.stringify({ font: { b: true, i: true, color: '#C0504D' }, fill: { color: '#95B3D7' } }));
  assert.deepEqual(look, { fill: '#95B3D7', ink: '#C0504D', bold: true, italic: true, underline: false, strike: false });
});

test('default black ink is left to the theme unless the cell has a fill', () => {
  assert.equal(cellLook(JSON.stringify({ font: { color: '#000000' } })), null);
  assert.equal(cellLook(JSON.stringify({ font: { color: '#000000' }, fill: { color: '#FFFF00' } })).ink, '#000000');
});

test('an unreadable style or a colour that is not hex draws nothing', () => {
  assert.equal(cellLook('not json'), null);
  assert.equal(cellLook(null), null);
  assert.equal(cellLook(JSON.stringify({ fill: { color: 'url(x)' } })), null);
});
