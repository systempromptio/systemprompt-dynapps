// Uploads a batch of files one after another, each through the resumable
// session, then imports it into Drive, where it is listed. Files run in
// sequence so parallelism is spent on parts, never on several files at once.
import { uploadFile } from './session.js';
import { importBlob, legacyHint, mimeOf } from './import.js';
import { createRow } from './progress.js';

const uploadOne = async (file, { list, folderId }) => {
  const controller = new AbortController();
  const row = createRow(list, file, () => controller.abort(new DOMException('Cancelled', 'AbortError')));
  const { signal } = controller;
  try {
    const { blobRef } = await uploadFile(file, { mime: mimeOf(file), signal, onProgress: row.progress });
    row.importing();
    await importBlob({ blobRef, name: file.name, folderId }, signal);
    row.done(legacyHint(file) ?? 'Imported');
  } catch (err) {
    row.failed(signal.aborted ? 'Cancelled' : err.message);
  }
};

export const uploadAll = async (files, options) => {
  for (const file of files) await uploadOne(file, options);
};
