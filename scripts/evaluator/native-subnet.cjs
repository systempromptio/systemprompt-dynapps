'use strict';
function interval(cidr) {
  if (typeof cidr !== 'string') throw Error('Invalid IPv4 route');
  const [address, prefix = '32'] = cidr.split('/');
  const octets = address.split('.');
  if (octets.length !== 4 || octets.some(value => !/^\d{1,3}$/.test(value) || Number(value) > 255) || !/^\d{1,2}$/.test(prefix) || Number(prefix) > 32 || cidr.split('/').length > 2) throw Error('Invalid IPv4 route');
  const value = octets.reduce((sum, octet) => sum * 256 + Number(octet), 0);
  const size = 2 ** (32 - Number(prefix)); const start = Math.floor(value / size) * size;
  return [start, start + size - 1];
}
function chooseSubnet(occupied, seed = 0) {
  if (!Number.isSafeInteger(seed) || seed < 0 || occupied.length > 32768) throw Error('Invalid bounded subnet inputs');
  const ranges = occupied.filter(cidr => cidr !== 'default' && cidr !== '0.0.0.0/0').map(interval);
  const pools = [[167772160, 1048576], [2886729728, 65536], [3232235520, 4096]];
  for (const [base, count] of pools) {
    for (let index = 0; index < Math.min(count, 4096); index++) {
      const start = base + ((seed % count + index * 4099) % count) * 16;
      if (ranges.some(([low, high]) => start <= high && start + 15 >= low)) continue;
      return `${[24, 16, 8, 0].map(shift => Math.floor(start / 2 ** shift) % 256).join('.')}/28`;
    }
  }
  throw Error('No collision-free private /28 found within bounded candidate set');
}
module.exports = { interval, chooseSubnet };
