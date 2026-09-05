import { showToast } from '../services/toast.js';

showToast('Email verified! Your account is now active.', 'is-success');
window.history.replaceState({}, '', '/admin/setup');
