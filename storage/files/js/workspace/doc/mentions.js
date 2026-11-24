// Mentions in a comment body: the thread draws the body as safe Markdown and
// each mention as a chip carrying the person's current name. The token
// grammar lives in mention-token.js.
import { person } from '../people.js';
import { buildBody } from './markdown-dom.js';
import { parseBody } from './markdown.js';

export { mentionIds, mentionToken } from './mention-token.js';

export const renderBody = (body) => buildBody(parseBody(body), person);
