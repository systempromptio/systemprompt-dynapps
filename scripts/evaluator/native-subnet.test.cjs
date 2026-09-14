'use strict';
const { test } = require('node:test');
const assert = require('node:assert/strict');
const { interval, chooseSubnet } = require('./native-subnet.cjs');
test('subnet excludes Docker pools and host connected/point routes', () => {
  const occupied = ['10.0.0.0/8', '172.16.0.0/12', '192.168.0.0/28', '192.168.0.17', 'default'];
  const selected = chooseSubnet(occupied);
  assert.equal(selected, '192.168.0.48/28');
  assert.deepEqual(interval(selected), [3232235568, 3232235583]);
});
test('explicit default route does not forbid all private networks', () => {
  assert.equal(chooseSubnet(['0.0.0.0/0'], 0), '10.0.0.0/28');
});
test('full private reservations and malformed routing evidence fail closed', () => {
  assert.throws(() => chooseSubnet(['10.0.0.0/8', '172.16.0.0/12', '192.168.0.0/16']), /No collision-free/);
  for (const route of ['1.2.3.999/24', '1.2.3.4/33', 'bad', '1.2.3.4/24/1']) assert.throws(() => chooseSubnet([route]), /Invalid IPv4/);
});
