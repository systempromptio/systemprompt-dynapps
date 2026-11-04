// Human messages for the upload API's `error_key`s. An unknown key falls back
// to the server's own message so a new refusal is still shown, not swallowed.
const MESSAGES = {
  upload_closed: 'This upload is no longer open. Try uploading the file again.',
  missing_parts: 'Some parts never arrived. Try uploading the file again.',
  too_large: 'This file is larger than uploads allow.',
  checksum_mismatch: 'The file changed or was corrupted in transit. Try again.',
  mime_mismatch: 'The file contents do not match its type.',
  unsupported_type: 'This file type cannot be uploaded.',
  archive_limits: 'This document holds more than an upload may unpack.',
  image_limits: 'This image is too large in pixels to accept.',
  unsafe_svg: 'This SVG contains scripts or external references and was refused.',
  part_budget_exhausted: 'This upload used up its retries. Upload the file again to start fresh.',
  unchecked_format: 'This file type cannot be checked safely, so it was refused.',
  quota_exceeded: 'Your storage quota is used up.',
  too_many_open_uploads: 'Too many uploads are in progress. Finish or cancel some first.',
};

export const describeError = async (resp) => {
  const text = await resp.text();
  try {
    const body = JSON.parse(text);
    return MESSAGES[body.error_key] ?? body.message ?? body.error ?? resp.statusText;
  } catch {
    return text || resp.statusText;
  }
};
