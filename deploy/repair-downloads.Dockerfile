# Preserve the released server layers; replace only the verified Mac download.
ARG BASE_IMAGE
FROM ${BASE_IMAGE}
COPY --chown=app:app dynapps-bridge-macos.dmg dynapps-bridge-macos.dmg.sha256 /app/storage/files/downloads/
