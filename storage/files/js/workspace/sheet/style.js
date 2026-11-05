// A cell's stored look, read from the engine's style JSON into values the grid
// can apply. Black is the engine's default ink, so it is left to the theme and
// stays readable on a dark page; a cell with a fill keeps its own ink.
const HEX = /^#[0-9a-f]{6}$/i;

const colour = (value) => (typeof value === 'string' && HEX.test(value) ? value : null);

export const cellLook = (json) => {
  if (!json) return null;
  let style;
  try {
    style = JSON.parse(json);
  } catch {
    return null;
  }
  const font = style?.font ?? {};
  const fill = colour(style?.fill?.color);
  const given = colour(font.color);
  const ink = given && (fill || given.toUpperCase() !== '#000000') ? given : null;
  const look = { fill, ink, bold: font.b === true, italic: font.i === true, underline: font.u === true, strike: font.strike === true };
  return look.fill || look.ink || look.bold || look.italic || look.underline || look.strike ? look : null;
};
