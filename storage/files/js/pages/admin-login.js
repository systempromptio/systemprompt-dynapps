const SSO_MESSAGES = {
  unavailable: 'Single sign-on is not configured on this instance yet. Contact the platform team.',
  no_subject: 'Astound SSO did not identify your account (no NameID in the assertion). IT needs to add a Name ID claim to the relying party.',
  invalid_assertion: 'Astound SSO refused or cancelled the sign-in, or returned a response this platform could not verify. Try again; if it persists, contact the platform team.',
  no_email: 'Astound SSO did not return an email address for your account. Contact the platform team.',
  forbidden: 'Your account is not on an email domain this platform accepts.',
  no_group: 'Your account is not in a Systemprompt-* AD group, so it has no access here. Ask IT to add you to the right group.',
  not_provisioned: 'No account exists for you yet. Ask IT to add you to a Systemprompt-* AD group.',
  correlation_lost: 'Astound SSO sent a sign-in this platform did not start. Start again from this page.',
  error: 'Sign-in failed or the sign-in page expired. Start again from this page.'
};

const params = new URLSearchParams(window.location.search);

const ssoStatus = params.get('sso');
if (ssoStatus) {
  const errEl = document.getElementById('error');
  if (errEl) {
    errEl.textContent = SSO_MESSAGES[ssoStatus] || SSO_MESSAGES.error;
    errEl.hidden = false;
  }
  const retryEl = document.getElementById('retry');
  if (retryEl) {
    retryEl.hidden = false;
  }
}
