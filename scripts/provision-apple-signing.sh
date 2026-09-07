#!/usr/bin/env bash
# Provision GitHub Actions from an existing Developer ID and App Store key.
# Private values travel over stdin to gh (client-side sealed-box encryption),
# never command arguments, source control, or terminal output.
set +x
set -euo pipefail
umask 077

REPO="${APPLE_SIGNING_REPO:-Astound-Digital/systemprompt-astound}"
: "${APPLE_CERT_PEM:?path to Developer ID certificate PEM}"
: "${APPLE_PRIVATE_KEY_PEM:?path to its private key PEM}"
: "${APPLE_NOTARY_KEY:?path to App Store Connect .p8 key}"
: "${APPLE_API_KEY_ID:?App Store Connect key ID}"
: "${APPLE_API_ISSUER_ID:?App Store Connect issuer UUID}"
TEAM_ID="${APPLE_TEAM_ID:-7FSAPLA7RX}"

for path in "$APPLE_CERT_PEM" "$APPLE_PRIVATE_KEY_PEM" "$APPLE_NOTARY_KEY"; do
    [ -f "$path" ] || { echo "Missing credential file: $path" >&2; exit 1; }
done
[[ "$APPLE_API_ISSUER_ID" =~ ^[0-9A-Fa-f-]{36}$ ]]
[[ "$APPLE_API_KEY_ID" =~ ^[A-Z0-9]+$ ]]
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# Prove the certificate belongs to the expected team and key before uploading.
SUBJECT=$(openssl x509 -in "$APPLE_CERT_PEM" -noout -subject -nameopt RFC2253)
[[ "$SUBJECT" == *"OU=$TEAM_ID"* ]]
IDENTITY=$(openssl x509 -in "$APPLE_CERT_PEM" -noout -subject -nameopt multiline | sed -n 's/^ *commonName *= //p')
[[ "$IDENTITY" == "Developer ID Application: "* ]]
openssl x509 -in "$APPLE_CERT_PEM" -checkend 86400 -noout >/dev/null
cert_key=$(openssl x509 -in "$APPLE_CERT_PEM" -pubkey -noout | openssl dgst -sha256)
private_key=$(openssl pkey -in "$APPLE_PRIVATE_KEY_PEM" -pubout | openssl dgst -sha256)
[ "$cert_key" = "$private_key" ] || { echo 'Certificate and private key do not match' >&2; exit 1; }
xcrun notarytool history --key "$APPLE_NOTARY_KEY" --key-id "$APPLE_API_KEY_ID" \
    --issuer "$APPLE_API_ISSUER_ID" --output-format json >/dev/null

CERT_PASSWORD=$(openssl rand -hex 32)
KEYCHAIN_PASSWORD=$(openssl rand -hex 32)
openssl pkcs12 -export -inkey "$APPLE_PRIVATE_KEY_PEM" -in "$APPLE_CERT_PEM" \
    -name "$IDENTITY" -out "$WORK/developer-id.p12" -passout fd:3 3<<<"$CERT_PASSWORD"
set_secret() {
    gh secret set "$1" --repo "$REPO"
    echo "Configured $1"
}
# Certificate secret is written last: its presence is not enough to enable a
# half-configured release, and CI checks the complete credential set anyway.
printf %s "$CERT_PASSWORD" | set_secret APPLE_CERT_PASSWORD
printf %s "$IDENTITY" | set_secret APPLE_SIGNING_IDENTITY
printf %s "$TEAM_ID" | set_secret APPLE_TEAM_ID
base64 < "$APPLE_NOTARY_KEY" | tr -d '\n' | set_secret APPLE_API_KEY_P8_BASE64
printf %s "$APPLE_API_KEY_ID" | set_secret APPLE_API_KEY_ID
printf %s "$APPLE_API_ISSUER_ID" | set_secret APPLE_API_ISSUER_ID
printf %s "$KEYCHAIN_PASSWORD" | set_secret KEYCHAIN_PASSWORD
base64 < "$WORK/developer-id.p12" | tr -d '\n' | set_secret APPLE_CERT_P12_BASE64
echo "Apple signing configured for $REPO. Prove it with a CI macOS build."
