// The MIME type sent for a file, and the import that turns every completed
// blob into a Drive document. Browsers leave `file.type` empty for several
// types (Markdown on most platforms), so the extension decides then. Legacy
// Office formats have no codec in core: they import as plain binary files.
import { importDocument } from './transport.js';

const OOXML = 'application/vnd.openxmlformats-officedocument';
const ODF = 'application/vnd.oasis.opendocument';

const BY_EXTENSION = {
  docx: `${OOXML}.wordprocessingml.document`,
  xlsx: `${OOXML}.spreadsheetml.sheet`,
  pptx: `${OOXML}.presentationml.presentation`,
  odt: `${ODF}.text`,
  ods: `${ODF}.spreadsheet`,
  odp: `${ODF}.presentation`,
  pdf: 'application/pdf',
  png: 'image/png',
  jpg: 'image/jpeg',
  jpeg: 'image/jpeg',
  gif: 'image/gif',
  webp: 'image/webp',
  svg: 'image/svg+xml',
  txt: 'text/plain',
  md: 'text/markdown',
  markdown: 'text/markdown',
  html: 'text/html',
  htm: 'text/html',
  csv: 'text/csv',
};

const LEGACY_OFFICE = { doc: 'docx', xls: 'xlsx', ppt: 'pptx', rtf: 'docx' };

const extensionOf = (file) => file.name.split('.').pop()?.toLowerCase() ?? '';

// Why: browsers report .docx as application/zip or octet-stream on some
// platforms and .csv as application/vnd.ms-excel on Windows, which makes core
// skip the archive guard and sniff check. file.type is never sent: a known
// extension names the type, anything else is octet-stream for core to sniff.
export const mimeOf = (file) => BY_EXTENSION[extensionOf(file)] ?? 'application/octet-stream';

export const legacyHint = (file) => {
  const modern = LEGACY_OFFICE[extensionOf(file)];
  return modern ? `Stored as a file. Save it as .${modern} to edit it in Drive.` : null;
};

export const importBlob = ({ blobRef, name, folderId }, signal) => importDocument({
  blob_ref: blobRef,
  name,
  ...(folderId ? { folder_id: folderId } : {}),
}, signal);
