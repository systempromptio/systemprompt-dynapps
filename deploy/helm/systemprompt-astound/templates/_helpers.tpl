{{/* Names and labels */}}
{{- define "astound.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "astound.fullname" -}}
{{- if .Values.fullnameOverride -}}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- $name := default .Chart.Name .Values.nameOverride -}}
{{- if contains $name .Release.Name -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}
{{- end -}}

{{- define "astound.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" }}
app.kubernetes.io/name: {{ include "astound.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ include "astound.tag" . | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{/* selectorLabels: pass (dict "ctx" $ "component" "gateway") */}}
{{- define "astound.selectorLabels" -}}
app.kubernetes.io/name: {{ include "astound.name" .ctx }}
app.kubernetes.io/instance: {{ .ctx.Release.Name }}
app.kubernetes.io/component: {{ .component }}
{{- end -}}

{{- define "astound.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
{{- default (include "astound.fullname" .) .Values.serviceAccount.name -}}
{{- else -}}
{{- required "serviceAccount.name is required when serviceAccount.create is false" .Values.serviceAccount.name -}}
{{- end -}}
{{- end -}}

{{- define "astound.tag" -}}
{{- default .Chart.AppVersion .Values.image.tag -}}
{{- end -}}

{{- define "astound.image" -}}
{{- if .Values.image.digest -}}
{{- printf "%s@%s" .Values.image.repository .Values.image.digest -}}
{{- else -}}
{{- printf "%s:%s" .Values.image.repository (include "astound.tag" .) -}}
{{- end -}}
{{- end -}}

{{- define "astound.profileDir" -}}/app/.systemprompt/profiles/k8s{{- end -}}

{{/*
profile.yaml for one role: pass (dict "ctx" $ "role" "gateway").
Mirrors deploy/self-host/profile.yaml.template; values.profile.overrides is
deep-merged last. server.instance_id is deliberately absent: the replica
identity is the pod name (HOSTNAME).
*/}}
{{- define "astound.profile" -}}
{{- $v := .ctx.Values -}}
{{- $p := $v.profile -}}
{{- $url := required "profile.externalUrl is required" $p.externalUrl -}}
{{- $base := dict
  "name" "k8s"
  "display_name" "Kubernetes"
  "target" "local"
  "site" (dict "name" "Astound Digital" "github_link" nil)
  "database" (dict "type" "postgres" "external_db_access" false "migrate_on_boot" false)
  "paths" (dict "system" "/app" "services" "/app/services" "bin" "/app/bin" "web_path" "/app/web" "storage" "/app/storage" "geoip_database" nil)
  "services" (dict "cache_dir" "/app/services-cache")
  "system_admin" (dict "username" "admin" "email" (required "profile.adminEmail is required" $p.adminEmail))
  "runtime" (dict "environment" "production" "log_level" $p.logLevel "output_format" "json" "no_color" true "non_interactive" true)
  "secrets" (dict "secrets_path" "./secrets.json" "validation" "strict" "source" "file")
  "extensions" (dict "disabled" list)
  "retention" $p.retention
  "rate_limits" $p.rateLimits
-}}
{{- $server := dict
  "host" "0.0.0.0"
  "port" (int $v.service.port)
  "api_server_url" $url
  "api_internal_url" (printf "http://localhost:%d" (int $v.service.port))
  "api_external_url" $url
  "use_https" true
  "cors_allowed_origins" (list $url "https://claude.ai")
  "content_negotiation" (dict "enabled" false "markdown_suffix" ".md")
  "security_headers" (dict "enabled" true "hsts" "max-age=63072000; includeSubDomains; preload" "frame_options" "DENY" "content_type_options" "nosniff" "referrer_policy" "strict-origin-when-cross-origin" "permissions_policy" "camera=(), microphone=(), geolocation=()" "content_security_policy" nil)
  "role" .role
  "max_in_flight" (int $p.maxInFlight)
  "max_concurrent_streams" (int $p.maxConcurrentStreams)
  "trusted_proxies" $p.trustedProxies
-}}
{{- if $v.metrics.enabled }}{{- $_ := set $server "metrics_port" (int $v.metrics.port) -}}{{- end -}}
{{- $_ := set $base "server" $server -}}
{{- $st := $p.storage -}}
{{- if eq $st.backend "gcs" -}}
{{- $gcs := dict "backend" "gcs" "bucket" (required "profile.storage.bucket is required with backend gcs" $st.bucket) "credentials" $st.credentials -}}
{{- if $st.prefix }}{{- $_ := set $gcs "prefix" $st.prefix -}}{{- end -}}
{{- $_ := set $base "storage" $gcs -}}
{{- else -}}
{{- $_ := set $base "storage" (dict "backend" $st.backend) -}}
{{- end -}}
{{- $_ := set $base "security" (dict "jwt_issuer" $url "jwt_access_token_expiration" 2592000 "jwt_refresh_token_expiration" 15552000 "jwt_audiences" (list "web" "api" "a2a" "mcp") "allowed_resource_audiences" (list "hook") "allow_registration" false "signing_key_path" "signing_key.pem" "id_jag_ttl_secs" 300) -}}
{{- $_ := set $base "governance" (dict "authz" (dict "hook" (dict "mode" "webhook" "url" (printf "http://localhost:%d/api/public/govern/authz" (int $v.service.port)) "timeout_ms" 500 "acknowledgement" nil))) -}}
{{- $merged := mergeOverwrite $base (deepCopy (default dict $p.overrides)) -}}
{{- toYaml $merged -}}
{{- end -}}

{{/*
Pod spec shared by the gateway, admin and migrate pods.
pass (dict "ctx" $ "role" "gateway" "component" "gateway" "profileConfigMap" NAME
          "serviceAccount" NAME "spc" NAME "secretName" NAME "job" bool "args" list "resources" dict)
*/}}
{{- define "astound.podSpec" -}}
{{- $ctx := .ctx -}}
{{- $v := $ctx.Values -}}
{{- $profileDir := include "astound.profileDir" $ctx -}}
serviceAccountName: {{ .serviceAccount }}
automountServiceAccountToken: false
{{- with $v.imagePullSecrets }}
imagePullSecrets:
  {{- toYaml . | nindent 2 }}
{{- end }}
securityContext:
  {{- toYaml $v.podSecurityContext | nindent 2 }}
{{- if .job }}
restartPolicy: Never
{{- else }}
terminationGracePeriodSeconds: {{ $v.pod.terminationGracePeriodSeconds }}
{{- end }}
initContainers:
{{- if $v.cloudSqlProxy.enabled }}
  # Native sidecar: starts first, stays up, stops after the app (and lets a Job complete).
  - name: cloud-sql-proxy
    image: "{{ $v.cloudSqlProxy.image.repository }}:{{ $v.cloudSqlProxy.image.tag }}"
    restartPolicy: Always
    args:
      - --port={{ $v.cloudSqlProxy.port }}
      - --address=127.0.0.1
      {{- if $v.cloudSqlProxy.autoIamAuthn }}
      - --auto-iam-authn
      {{- end }}
      {{- if $v.cloudSqlProxy.privateIp }}
      - --private-ip
      {{- end }}
      - --structured-logs
      - --exit-zero-on-sigterm
      - --health-check
      - --http-address=0.0.0.0
      - --http-port=9801
      {{- range $v.cloudSqlProxy.extraArgs }}
      - {{ . }}
      {{- end }}
      - {{ required "cloudSqlProxy.instanceConnectionName is required" $v.cloudSqlProxy.instanceConnectionName | quote }}
    startupProbe:
      httpGet: {path: /startup, port: 9801}
      periodSeconds: 2
      failureThreshold: 60
    livenessProbe:
      httpGet: {path: /liveness, port: 9801}
      periodSeconds: 10
    securityContext:
      {{- toYaml $v.containerSecurityContext | nindent 6 }}
      runAsUser: 65532
      runAsGroup: 65532
    resources:
      {{- toYaml $v.cloudSqlProxy.resources | nindent 6 }}
{{- end }}
  # The server reads profile.yaml and secrets.json from one directory; assemble
  # it in memory from the ConfigMap and the secret source.
  - name: assemble-profile
    image: {{ include "astound.image" $ctx }}
    imagePullPolicy: {{ $v.image.pullPolicy }}
    command: ["/bin/sh", "-ec"]
    args:
      - |
        cp /src/profile/profile.yaml {{ $profileDir }}/profile.yaml
        cp /src/secrets/{{ $v.secrets.fileName }} {{ $profileDir }}/secrets.json
        chmod 0600 {{ $profileDir }}/secrets.json
    securityContext:
      {{- toYaml $v.containerSecurityContext | nindent 6 }}
    resources:
      requests: {cpu: 10m, memory: 16Mi}
      limits: {cpu: 100m, memory: 64Mi}
    volumeMounts:
      - {name: profile-src, mountPath: /src/profile, readOnly: true}
      - {name: secrets-src, mountPath: /src/secrets, readOnly: true}
      - {name: profile, mountPath: {{ $profileDir }}}
containers:
  - name: {{ if .job }}migrate{{ else }}server{{ end }}
    image: {{ include "astound.image" $ctx }}
    imagePullPolicy: {{ $v.image.pullPolicy }}
    {{- with .args }}
    args:
      {{- toYaml . | nindent 6 }}
    {{- end }}
    env:
      - {name: SYSTEMPROMPT_PROFILE_DIR, value: {{ $profileDir }}}
      - {name: HOME, value: /home/app}
      {{- with $v.pod.extraEnv }}
      {{- toYaml . | nindent 6 }}
      {{- end }}
    {{- if not .job }}
    ports:
      - {name: http, containerPort: {{ $v.service.port }}, protocol: TCP}
      {{- if $v.metrics.enabled }}
      - {name: metrics, containerPort: {{ $v.metrics.port }}, protocol: TCP}
      {{- end }}
    startupProbe:
      httpGet: {path: {{ $v.probes.startup.path }}, port: http}
      periodSeconds: {{ $v.probes.startup.periodSeconds }}
      failureThreshold: {{ $v.probes.startup.failureThreshold }}
    livenessProbe:
      httpGet: {path: {{ $v.probes.liveness.path }}, port: http}
      periodSeconds: {{ $v.probes.liveness.periodSeconds }}
      timeoutSeconds: {{ $v.probes.liveness.timeoutSeconds }}
      failureThreshold: {{ $v.probes.liveness.failureThreshold }}
    readinessProbe:
      httpGet: {path: {{ $v.probes.readiness.path }}, port: http}
      periodSeconds: {{ $v.probes.readiness.periodSeconds }}
      timeoutSeconds: {{ $v.probes.readiness.timeoutSeconds }}
      failureThreshold: {{ $v.probes.readiness.failureThreshold }}
    lifecycle:
      preStop:
        exec:
          command: ["/bin/sh", "-c", "sleep {{ $v.pod.preStopSleepSeconds }}"]
    {{- end }}
    securityContext:
      {{- toYaml $v.containerSecurityContext | nindent 6 }}
    resources:
      {{- toYaml .resources | nindent 6 }}
    volumeMounts:
      - {name: profile, mountPath: {{ $profileDir }}, readOnly: true}
      - {name: storage, mountPath: /app/storage}
      - {name: web, mountPath: /app/web}
      - {name: services-cache, mountPath: /app/services-cache}
      - {name: logs, mountPath: /app/logs}
      - {name: sessions, mountPath: /app/.systemprompt/sessions}
      - {name: tmp, mountPath: /tmp}
      - {name: home, mountPath: /home/app}
      {{- with $v.pod.extraVolumeMounts }}
      {{- toYaml . | nindent 6 }}
      {{- end }}
volumes:
  - name: profile-src
    configMap:
      name: {{ .profileConfigMap }}
  - name: secrets-src
  {{- if eq $v.secrets.mode "csi" }}
    csi:
      driver: {{ $v.secrets.csi.driver }}
      readOnly: true
      volumeAttributes:
        secretProviderClass: {{ .spc }}
  {{- else }}
    secret:
      secretName: {{ .secretName }}
      defaultMode: 0400
      items:
        - key: {{ if eq $v.secrets.mode "existingSecret" }}{{ $v.secrets.existingSecret.key }}{{ else }}secrets.json{{ end }}
          path: {{ $v.secrets.fileName }}
  {{- end }}
  - name: profile
    emptyDir: {medium: Memory, sizeLimit: 16Mi}
  - name: storage
    emptyDir: {sizeLimit: {{ $v.pod.volumes.storage }}}
  - name: web
    emptyDir: {sizeLimit: {{ $v.pod.volumes.web }}}
  - name: services-cache
    emptyDir: {sizeLimit: {{ $v.pod.volumes.servicesCache }}}
  - name: logs
    emptyDir: {sizeLimit: {{ $v.pod.volumes.logs }}}
  - name: sessions
    emptyDir: {sizeLimit: 16Mi}
  - name: tmp
    emptyDir: {medium: Memory, sizeLimit: {{ $v.pod.volumes.tmp }}}
  - name: home
    emptyDir: {sizeLimit: {{ $v.pod.volumes.home }}}
{{- with $v.pod.extraVolumes }}
  {{- toYaml . | nindent 2 }}
{{- end }}
{{- end -}}

{{/* Name of the Secret holding secrets.json in the non-CSI modes. */}}
{{- define "astound.secretName" -}}
{{- if eq .Values.secrets.mode "existingSecret" -}}
{{- required "secrets.existingSecret.name is required" .Values.secrets.existingSecret.name -}}
{{- else -}}
{{- printf "%s-secrets" (include "astound.fullname" .) -}}
{{- end -}}
{{- end -}}

{{/* SecretProviderClass spec body (shared by the app and migrate copies). */}}
{{- define "astound.spcSpec" -}}
provider: {{ .Values.secrets.csi.provider }}
parameters:
  secrets: |
    - resourceName: {{ required "secrets.csi.secretVersion is required with secrets.mode csi" .Values.secrets.csi.secretVersion | quote }}
      path: {{ .Values.secrets.fileName | quote }}
{{- end -}}
