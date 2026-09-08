// Document parsers. Only the formats that actually occur in our sources:
// PDF (meeting transcripts, some specs), DOCX (some specs), and plain
// text/markdown (Confluence/Jira from the Atlassian sync, ADRs). Other formats
// are intentionally unsupported.

import { readFile } from 'node:fs/promises';
import { extname } from 'node:path';

const NATIVE_TEXT = new Set(['.txt', '.md']);
const BINARY = new Set(['.pdf', '.docx']);

export const SUPPORTED_SUFFIXES = new Set([...NATIVE_TEXT, ...BINARY]);

export function isBinarySuffix(suffix) {
  return BINARY.has(suffix.toLowerCase());
}

/** Extract plain text from a supported file. */
export async function parse(filePath) {
  const suffix = extname(filePath).toLowerCase();
  if (suffix === '.pdf') return parsePdf(filePath);
  if (suffix === '.docx') return parseDocx(filePath);
  return readFile(filePath, 'utf8');
}

async function parsePdf(filePath) {
  // Import the library entry directly to avoid its debug harness on import.
  const { default: pdfParse } = await import('pdf-parse/lib/pdf-parse.js');
  const buffer = await readFile(filePath);
  const { text } = await pdfParse(buffer);
  return text;
}

async function parseDocx(filePath) {
  const mammoth = (await import('mammoth')).default;
  const { value } = await mammoth.extractRawText({ path: filePath });
  return value;
}
