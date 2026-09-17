import { renderConnections, watchConnections } from "/assets/js/components/profile-connections.js";
import { SpElement, reactive } from "/assets/js/components/sp-element.js";
import { escapeHtml } from "/assets/js/utils/escape.js";
import { bridge } from "/assets/js/bridge.js";
import { t } from "/assets/js/i18n.js";
import { notifyErr } from "/assets/js/utils/notify.js";
import { renderProfileIdentity, renderProfilePlan, renderProfileSkeleton } from "/assets/js/components/profile-identity.js";
import { renderProfileUsage, renderProfileModels } from "/assets/js/components/profile-usage.js";
import { renderProfileConversations } from "/assets/js/components/profile-conversations.js";

export class SpProfile extends SpElement {
  constructor() {
    super();
    this.snapshot = null;
    this.profile = null;
    this.loading = true;
    this.error = "";
    this.registerAction("manage-connection", () => {
      if (!this.profile) return;
      const url = new URL("/admin/connectors", this.profile.gateway);
      url.searchParams.set("expected_user", this.profile.identity.user_id);
      bridge.openExternalUrl(url.toString()).catch((e) => notifyErr(e, url.toString()));
    });
    this.registerAction("refresh", () => this._fetch());
  }

  onConnect() {
    this.useSnapshot((s) => {
      const wasSignedIn = this.snapshot && this.snapshot.verified_identity;
      const nowSignedIn = s && s.verified_identity;
      this.snapshot = s;
      if (!wasSignedIn && nowSignedIn) { this._fetch(); }
      if (wasSignedIn && !nowSignedIn) { this.profile = null; }
    });
    this._unsubs.push(watchConnections(() => this.invalidate()));
    this._fetch();
  }

  async _fetch() {
    this.loading = true;
    this.error = "";
    try {
      this.profile = await bridge.profileFetch();
    } catch (e) {
      this.error = (e && e.message) || (t("profile-fetch-failed") || "Could not load your profile.");
      this.profile = null;
    } finally {
      this.loading = false;
    }
  }

  render() {
    return `
      <header class="sp-tab__header">
        <h1 data-l10n-id="profile-heading">Profile</h1>
        <button class="sp-btn-ghost" type="button" data-action="refresh" data-l10n-id="profile-refresh">Refresh</button>
      </header>
      ${this._renderBody()}
    `;
  }

  _renderBody() {
    if (this.error) {
      return `
        <section class="sp-profile-error" role="alert">
          <p data-l10n-id="profile-error-fetch">Could not load profile.</p>
          <p class="sp-u-muted">${escapeHtml(this.error)}</p>
        </section>
      `;
    }
    if (this.loading && !this.profile) { return renderProfileSkeleton(); }
    if (!this.profile) {
      return `<section class="sp-profile-empty"><p data-l10n-id="profile-signed-out">Sign in to see your profile.</p></section>`;
    }
    return `
      <div class="sp-profile-grid">
        ${renderProfileIdentity({ ...this.profile, identity: { ...this.profile.identity,
          extra: Object.fromEntries(Object.entries(this.profile.identity.extra || {}).filter(([key]) => key !== "connections")) } }, this.snapshot)}
        ${renderConnections(this.profile)}
        ${renderProfileUsage(this.profile)}
        ${renderProfileModels(this.profile)}
        ${renderProfileConversations(this.profile)}
        ${renderProfilePlan(this.profile)}
      </div>
    `;
  }
}

reactive(SpProfile.prototype, ["snapshot", "profile", "loading", "error"]);
customElements.define("sp-profile", SpProfile);
