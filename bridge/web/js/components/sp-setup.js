/* Astound onboarding: sign in, then confirm who you are.
 *
 * Two steps, no stepper. Agents used to be a third step, but the Agents tab
 * and the automatic first-run install already own that surface, so the wizard
 * asked the user to do something the app had already done.
 *
 * Nothing here names an operating system. The platform is decided at compile
 * time by core (`PLATFORM_DISPLAY` in `src/web_assets.rs`) and handed to the
 * page as `data-platform-display`; a hardcoded "Mac" read wrong on every
 * Windows build we shipped.
 */
import { SpElement, reactive, escapeHtml } from "/assets/js/components/sp-element.js";
import { onBridgeEvent } from "/assets/js/events/bridge-events.js";
import { bridge } from "/assets/js/bridge.js";
import { fmtDuration } from "/assets/js/utils/format.js";
import { tr } from "/assets/js/utils/l10n.js";
import "/assets/js/components/sp-setup-gateway.js";

function isConfigured(snap) {
  const reachable = snap.gateway_status && snap.gateway_status.state === "reachable";
  const id = snap.verified_identity;
  return !!(reachable && id && id.user_id);
}

function gatewayHost(url) {
  try { return new URL(url).host; } catch (e) { return url || "your gateway"; }
}

export class SpSetup extends SpElement {
  constructor() {
    super();
    this.snapshot = null;
    this.step = "connect";
    this._finished = false;
    /** Latched once the app proper is on screen; see `_applySnapshot`. */
    this._leftSetup = false;
    this._logoFragment = null;
    this._onSetupOpen = () => { document.body.classList.add("is-setup-mode"); };
    this.registerAction("finish", () => this._finish());
    this.registerAction("sign-out", () => {
      bridge.logout().catch((e) => console.warn("logout", e));
    });
  }

  onConnect() {
    const tpl = this.querySelector('template[data-slot="logo"]');
    if (tpl) {
      this._logoFragment = tpl.content;
      tpl.remove();
    }
    bridge.stateSnapshot().then((s) => this._applySnapshot(s)).catch((e) => console.warn("snapshot failed", e));
    this.bridgeSubscribe("state.changed", (s) => this._applySnapshot(s));
    this._unsubOpen = onBridgeEvent("setup-open", this._onSetupOpen);
  }

  onDisconnect() {
    if (this._unsubOpen) { this._unsubOpen(); this._unsubOpen = null; }
  }

  _applySnapshot(snap) {
    this.snapshot = snap;
    if (!snap) { return; }
    const configured = isConfigured(snap);
    this.step = configured ? "done" : "connect";

    // Signing out is the one thing that legitimately sends us back to the
    // splash. Clear the latch so it can.
    if (!snap.verified_identity || !snap.verified_identity.user_id) { this._leftSetup = false; }

    // Only decide overlay visibility on a settled gateway probe — partial
    // snapshots flip `configured` mid-startup and made the window flick
    // splash → app → splash.
    const gatewayProbing = !snap.gateway_status || snap.gateway_status.state === "probing"
      || snap.gateway_status.state === "unknown";
    if (gatewayProbing) { return; }

    // One-way latch: once the app proper has been shown, a later probe result
    // must not yank the user back into onboarding mid-session.
    if (this._leftSetup) { return; }

    // `agents_onboarded` is the durable sentinel core writes on setup.complete;
    // without consulting it a returning user re-entered onboarding on every
    // launch.
    const onboarded = this._finished || !!snap.agents_onboarded;
    const inSetup = !configured || !onboarded;
    if (!inSetup) { this._leftSetup = true; }
    document.body.classList.toggle("is-setup-mode", inSetup);
  }

  _finish() {
    this._finished = true;
    this._leftSetup = true;
    bridge.setupComplete().catch((err) => console.warn("setup complete", err));
    document.body.classList.remove("is-setup-mode");
  }

  afterRender() {
    document.body.dataset.setupStep = this.step;
    const slot = this.querySelector("[data-logo-slot]");
    if (slot && this._logoFragment && !slot.firstElementChild) {
      slot.append(this._logoFragment.cloneNode(true));
    }
  }

  _identityEmail() {
    const id = this.snapshot && this.snapshot.verified_identity;
    return (id && (id.email || id.user_id)) || "";
  }

  /** One row of the closing summary. */
  _summaryRow(ok, label, value) {
    return `<div class="sp-setup__done-row" data-ok="${ok ? "yes" : "no"}">
      <i aria-hidden="true">${ok ? "✓" : "!"}</i>
      <span>${escapeHtml(label)}</span>
      <b>${escapeHtml(value)}</b>
    </div>`;
  }

  /* What the desktop can prove right now: who it linked as, where it sends
   * inference, whether the proxy agents talk to is up, and how long the
   * session has left. The full SSO identity — issuer, AD groups, department,
   * organization — is on the Account tab. */
  _renderDoneRows() {
    const snap = this.snapshot || {};
    const proxy = snap.local_proxy || {};
    const proxyOk = proxy.state === "Listening";
    const ttl = snap.cached_token ? snap.cached_token.ttl_seconds : null;

    let rows = "";
    rows += this._summaryRow(true, tr("astound-done-gateway", "Governed by"), gatewayHost(snap.gateway_url));
    rows += this._summaryRow(proxyOk, tr("astound-done-proxy", "Local proxy"),
      proxyOk ? (proxy.url || "listening") : "not listening yet");
    if (ttl != null) {
      rows += this._summaryRow(ttl >= 600, tr("astound-done-session", "Session"), `expires in ${fmtDuration(ttl)}`);
    }
    return rows;
  }

  render() {
    const step = this.step;
    const version = this.dataset.version || "";
    const platform = this.dataset.platform || "linux";
    const platformDisplay = this.dataset.platformDisplay || "this computer";

    return `
      <div class="sp-setup__split">
        <aside class="sp-setup__brand">
          <div class="sp-setup__mark" data-logo-slot data-preserve></div>
          <div class="sp-setup__pitch">
            <p class="sp-setup__pitch-head">Govern every coding agent.</p>
            <p class="sp-setup__pitch-body">One gateway. Every agent. Every tool call audited.</p>
          </div>
          <footer class="sp-setup__brand-foot">
            <p class="sp-setup__demo">
              <strong>${escapeHtml(tr("astound-demo-strong", "Demo software."))}</strong>
              <span>${escapeHtml(tr("astound-demo-body", "This build is provided for demonstration purposes only and is not licensed for production use."))}</span>
            </p>
            <p class="sp-setup__meta">
              <span class="sp-setup__version">v${escapeHtml(version)}</span>
              <span class="sp-setup__meta-sep">·</span>
              <a class="sp-setup__docs" href="https://systemprompt.io/docs/bridge/${escapeHtml(platform)}" target="_blank" rel="noopener noreferrer">
                Documentation for ${escapeHtml(platformDisplay)} →
              </a>
              <span class="sp-setup__meta-sep">·</span>
              <a href="mailto:ed@systemprompt.io?subject=Astound%20Bridge%20licensing">Licensing</a>
            </p>
          </footer>
        </aside>

        <section class="sp-setup__panel">
          <div class="sp-setup__panel-inner">
            <div class="sp-setup__step" data-step="connect" ${step !== "connect" ? "hidden" : ""}>
              <h1 id="setup-heading">${escapeHtml(tr("astound-setup-connect-heading", "Sign in"))}</h1>
              <p class="sp-setup__lede">
                ${escapeHtml(tr("astound-setup-connect-lede",
                  "Your browser opens to Astound SSO. Whoever you sign in as there is the account this device links to — your bridge account is created automatically the first time."))}
              </p>
              <sp-setup-gateway></sp-setup-gateway>
            </div>

            <div class="sp-setup__step" data-step="done" ${step !== "done" ? "hidden" : ""}>
              <h1>${escapeHtml(tr("astound-setup-done-heading", "You're connected"))}</h1>
              <p class="sp-setup__lede">${escapeHtml(tr("astound-setup-done-lede", "This device is linked to"))}</p>
              <p class="sp-setup__identity">${escapeHtml(this._identityEmail())}</p>
              <div class="sp-setup__done">${step === "done" ? this._renderDoneRows() : ""}</div>
              <p class="sp-setup__hint">
                ${escapeHtml(tr("astound-setup-done-hint-lead", "Not you?"))}
                <a href="#" data-action="sign-out">${escapeHtml(tr("astound-setup-sign-out", "Sign out"))}</a>
                ${escapeHtml(tr("astound-setup-done-hint-rest",
                  "and sign in again — you choose the account in the browser. Coding agents are managed in the Agents tab, and your full sign-in details are on the Account tab."))}
              </p>
              <div class="sp-setup__actions">
                <button class="sp-btn-primary" type="button" data-action="finish">
                  <span class="sp-btn__label">${escapeHtml(tr("astound-setup-open", "Open Astound Bridge"))}</span>
                </button>
              </div>
            </div>
          </div>
        </section>
      </div>
    `;
  }
}

reactive(SpSetup.prototype, ["snapshot", "step"]);
customElements.define("sp-setup", SpSetup);
