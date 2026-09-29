// A form post that ends in a redirect leaves its outcome in the address as
// `flash` and `flash_tone` (see admin-core ssr::flash). This draws it once as
// a notice above the content and strips the parameters, so a reload or a
// shared link never repeats a stale outcome.

const TONES = ['ok', 'warn', 'err', 'info'];

const render = (region, message, tone) => {
  const notice = document.createElement('div');
  notice.className = `sp-notice sp-notice--${tone}`;
  notice.setAttribute('role', tone === 'err' ? 'alert' : 'status');
  const text = document.createElement('p');
  text.textContent = message;
  notice.append(text);
  region.replaceChildren(notice);
  region.hidden = false;
};

export const initFlash = () => {
  const region = document.getElementById('sp-flash');
  if (!region) return;
  const url = new URL(window.location.href);
  const message = url.searchParams.get('flash');
  if (!message) return;
  const wanted = url.searchParams.get('flash_tone') || 'info';
  const tone = TONES.includes(wanted) ? wanted : 'info';
  render(region, message, tone);
  url.searchParams.delete('flash');
  url.searchParams.delete('flash_tone');
  window.history.replaceState(window.history.state, '', url.toString());
};
