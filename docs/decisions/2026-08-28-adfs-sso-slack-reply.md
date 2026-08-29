# ADFS SSO — reply to Astound IT (Slack-ready)

Companion to `2026-08-28-adfs-sso-replaces-salesforce.md`. Everything below the
line is formatted to paste straight into the Slack thread with Sergey and Boris.
Nothing here contains the test-account password; keep it that way.

---

Hi Sergey, Boris — thanks, the relying party works. Tested end to end with the e.burton account this afternoon.

**What works**
- Sign-in via "Systemprompt DEV" on the AD FS portal returns a signed assertion (RSA-SHA256, your metadata signing cert), audience `https://astound.systemprompt.io/saml/metadata`, ACS `https://astound.systemprompt.io/admin/auth/adfs/callback`. Signature verification passes on our side; a tampered assertion is rejected. So Entity ID / ACS are settled — the values you used are the ones.

**1. Open decisions**
- Protocol: settled — **SAML**, as you built it. No OIDC application group needed.
- Group → permission mapping: proposed `Systemprompt-Admins` → platform admin, `Systemprompt-Users` → standard user, further `Systemprompt-*` groups → team entitlements. No mapped group ⇒ no session. Tell me if you want different names.
- Sync policy: group changes apply at next sign-in (session TTL 1 h). Say if you need a harder cap.
- Deprovisioning: disabling the AD account stops sign-in immediately; tell me whether you also want the platform account auto-closed.

**2. What I need changed on the relying-party trust (issuance rules)**
1. **Name ID** claim — email-address format. Today the assertion has no `<NameID>`, so we cannot identify the user.
2. **E-Mail-Address** claim (`http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress`) — today only `name` is sent.
3. **Groups**: today the claim is `http://temp/groups`, unfiltered (`Domain Users`, `ADFS-FS-Users` come through). Please emit it as `http://schemas.xmlsoap.org/claims/Group`, filtered to the `Systemprompt-*` prefix.
4. Typo: the group is emitted as **`Sytemprompt-Admins`** (missing "s"). Please rename to `Systemprompt-Admins`.

**3. What I'm blocked on**
- Items 1–2 above: until the assertion carries a Name ID / email, every sign-in is (correctly) refused as "no subject".
- The final `Systemprompt-*` group names and their meaning.
- Please rotate the e.burton test password once we're done; it went over Slack.

Everything on our side — login page, signature verification, group-to-role mapping, audit trail — is built and deployed to staging; I'll confirm in this thread the moment a sign-in completes.
