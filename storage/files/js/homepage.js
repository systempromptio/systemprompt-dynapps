import { initCopyButtons } from './site/copy-buttons.js';
import { initStatusCard } from './site/status-card.js';

function initHeaderScroll() {
  const header = document.querySelector('.site-header');
  if (!header) return;
  const update = () => header.classList.toggle('is-scrolled', window.scrollY > 24);
  window.addEventListener('scroll', update, { passive: true });
  update();
}

function initReveal() {
  const targets = document.querySelectorAll('[data-reveal]');
  if (!targets.length) return;

  const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  if (reduced || !('IntersectionObserver' in window)) {
    targets.forEach((el) => el.classList.add('is-revealed'));
    return;
  }

  const observer = new IntersectionObserver((entries) => {
    entries.forEach((entry) => {
      if (!entry.isIntersecting) return;
      entry.target.classList.add('is-revealed');
      observer.unobserve(entry.target);
    });
  }, { rootMargin: '0px 0px -12% 0px', threshold: 0.1 });

  targets.forEach((el) => observer.observe(el));
}

export function initHomepage() {
  initCopyButtons();
  initStatusCard();
  initHeaderScroll();
  initReveal();
}

initHomepage();
