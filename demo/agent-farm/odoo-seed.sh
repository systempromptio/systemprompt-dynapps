#!/usr/bin/env bash
# Seed (or reset) the demo-* Odoo records the Odoo farm agents work on.
#
#   demo/agent-farm/odoo-seed.sh            # idempotent: creates what is missing
#   demo/agent-farm/odoo-seed.sh --reset    # archives/unlinks only demo-* records
#
# Talks JSON-RPC as the dynapps-farm-bot user. Credentials come from the
# profile secrets (PROFILE, default local): odoo_url, odoo_db,
# odoo_farm_login, odoo_farm_api_key. Every record it creates carries a
# demo-* tag or a "demo " name prefix; nothing else is ever touched.
# Portable to macOS and Linux: bash, curl, jq only.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PROFILE="${PROFILE:-local}"
SECRETS="$ROOT/.systemprompt/profiles/$PROFILE/secrets.json"
[ -f "$SECRETS" ] || { echo "ERROR: $SECRETS not found" >&2; exit 1; }
command -v jq >/dev/null || { echo "ERROR: jq is required" >&2; exit 1; }

sec() { jq -er --arg k "$1" '.[$k] // empty' "$SECRETS" || { echo "ERROR: $1 missing in $SECRETS" >&2; exit 1; }; }
URL="$(sec odoo_url)"; DB="$(sec odoo_db)"; LOGIN="$(sec odoo_farm_login)"; KEY="$(sec odoo_farm_api_key)"

rpc() { # rpc <service> <method> <args-json>
  local body out
  body="$(jq -n --arg s "$1" --arg m "$2" --argjson a "$3" \
    '{jsonrpc:"2.0",method:"call",id:1,params:{service:$s,method:$m,args:$a}}')"
  out="$(curl -sS --fail "$URL/jsonrpc" -H 'content-type: application/json' -d "$body")"
  if [ "$(printf '%s' "$out" | jq 'has("error")')" = "true" ]; then
    printf '%s' "$out" | jq -r '.error.data.message // .error.message' >&2; exit 1
  fi
  printf '%s' "$out" | jq -c '.result'
}

UID_="$(rpc common authenticate "$(jq -n --arg d "$DB" --arg l "$LOGIN" --arg k "$KEY" '[$d,$l,$k,{}]')")"
[ "$UID_" != "false" ] || { echo "ERROR: Odoo authentication failed for $LOGIN" >&2; exit 1; }

kw() { # kw <model> <method> <args-json> [kwargs-json]
  rpc object execute_kw "$(jq -n --arg d "$DB" --argjson u "$UID_" --arg k "$KEY" \
    --arg mo "$1" --arg me "$2" --argjson a "$3" --argjson kw "${4:-{\}}" '[$d,$u,$k,$mo,$me,$a,$kw]')"
}
ALL='{"context":{"active_test":false}}'

# find_or_create <model> <domain-json> <values-json> -> id
find_or_create() {
  local id
  id="$(kw "$1" search "[[$2]]" '{"limit":1,"context":{"active_test":false}}' | jq '.[0] // empty')"
  if [ -z "$id" ]; then id="$(kw "$1" create "[$3]")"; fi
  echo "$id"
}

TAGS="demo-triage demo-dedup demo-gdpr"

if [ "${1:-}" = "--reset" ]; then
  for t in $TAGS; do
    tag="$(kw crm.tag search "[[[\"name\",\"=\",\"$t\"]]]" | jq '.[0] // empty')"
    cat="$(kw res.partner.category search "[[[\"name\",\"=\",\"$t\"]]]" | jq '.[0] // empty')"
    if [ -n "$tag" ]; then
      ids="$(kw crm.lead search "[[[\"tag_ids\",\"in\",[$tag]]]]" "$ALL")"
      [ "$ids" = "[]" ] || kw crm.lead write "[$ids,{\"active\":false}]" >/dev/null
      echo "archived $t leads: $ids"
    fi
    if [ -n "$cat" ]; then
      ids="$(kw res.partner search "[[[\"category_id\",\"in\",[$cat]]]]" "$ALL")"
      [ "$ids" = "[]" ] || kw res.partner write "[$ids,{\"active\":false}]" >/dev/null
      echo "archived $t partners: $ids"
    fi
  done
  drafts="$(kw account.move search '[[["partner_id.name","=","demo GDPR requester"],["state","=","draft"]]]' "$ALL")"
  [ "$drafts" = "[]" ] || kw account.move unlink "[$drafts]" >/dev/null
  echo "unlinked demo draft invoices: $drafts (posted invoices are kept: legal retention)"
  exit 0
fi

ctag() { local v="CRM_TAG_${1//-/_}"; echo "${!v}"; }
pcat() { local v="PARTNER_CAT_${1//-/_}"; echo "${!v}"; }
for t in $TAGS; do
  printf -v "CRM_TAG_${t//-/_}" '%s' "$(find_or_create crm.tag "[\"name\",\"=\",\"$t\"]" "{\"name\":\"$t\"}")"
  printf -v "PARTNER_CAT_${t//-/_}" '%s' "$(find_or_create res.partner.category "[\"name\",\"=\",\"$t\"]" "{\"name\":\"$t\"}")"
done
echo "crm.tag: demo-triage=$(ctag demo-triage) demo-dedup=$(ctag demo-dedup) demo-gdpr=$(ctag demo-gdpr)"
echo "res.partner.category: demo-triage=$(pcat demo-triage) demo-dedup=$(pcat demo-dedup) demo-gdpr=$(pcat demo-gdpr)"

lead() { # lead <tag> <name> <company> <contact> <email> <phone> <city> <description>
  find_or_create crm.lead "[\"name\",\"=\",\"$2\"],[\"tag_ids\",\"in\",[$(ctag $1)]]" \
    "$(jq -n --arg n "$2" --arg c "$3" --arg p "$4" --arg e "$5" --arg ph "$6" --arg ci "$7" \
      --arg d "$8" --argjson t "$(ctag $1)" \
      '{name:$n,type:"lead",partner_name:$c,contact_name:$p,email_from:$e,phone:$ph,city:$ci,description:$d,tag_ids:[[6,0,[$t]]]}')"
}

partner() { # partner <tag> <name> <email> <phone> <city> [is_company]
  find_or_create res.partner "[\"name\",\"=\",\"$2\"],[\"email\",\"=\",\"$3\"],[\"category_id\",\"in\",[$(pcat $1)]]" \
    "$(jq -n --arg n "$2" --arg e "$3" --arg ph "$4" --arg ci "$5" --argjson co "${6:-true}" \
      --argjson c "$(pcat $1)" '{name:$n,email:$e,phone:$ph,city:$ci,is_company:$co,category_id:[[6,0,[$c]]]}')"
}

echo "-- demo-triage leads"
for row in \
  "Odoo rollout for 40 users|Brouwerij De Kroon BV|Pieter Janssens|pieter.janssens@example.com|+32 3 555 0101|Antwerp|Craft brewery, 3 sites. Wants inventory + MRP, budget approved for Q1, decision maker on the call." \
  "Migrate from Exact Online|Logistiek Vandamme NV|Sofie Vandamme|sofie@example.com|+32 9 555 0102|Ghent|Freight forwarder, 25 staff. Unhappy with current ERP, asked for pricing only, no timeline." \
  "E-commerce + POS integration|Bodega Ruiz SL|Javier Ruiz|javier.ruiz@example.com|+34 91 555 0103|Madrid|Wine retailer, 2 shops + webshop. Demo booked next week, wants go-live before summer." \
  "Newsletter signup|Talleres Ortega SA|Lucia Ortega|lucia.ortega@example.com|+34 96 555 0104|Valencia|Downloaded a whitepaper. No reply to two follow-ups." \
  "Field service module question|Installaties Peeters BVBA|Koen Peeters|koen@example.com|+32 16 555 0105|Leuven|HVAC installer, 60 technicians. Asked about field service and helpdesk, evaluating two vendors." ; do
  IFS='|' read -r n c p e ph ci d <<<"$row"
  echo "  $(lead demo-triage "$n" "$c" "$p" "$e" "$ph" "$ci" "$d")  $n"
done

echo "-- demo-dedup partner pairs"
for row in \
  "Brouwerij De Kroon BV|info@dekroon.example.com|+32 3 555 0201|Antwerp" \
  "Brouwerij de Kroon|info@dekroon.example.com|+32 3 555 0201|Antwerpen" \
  "Bodega Ruiz SL|hola@bodegaruiz.example.com|+34 91 555 0202|Madrid" \
  "Bodega Ruiz S.L.|hola@bodegaruiz.example.com|+34 915550202|Madrid" \
  "Marta Gil|marta.gil@example.com|+34 93 555 0203|Barcelona" \
  "Gil, Marta|marta.gil@example.com|+34 93 555 0203|Barcelona" ; do
  IFS='|' read -r n e ph ci <<<"$row"
  echo "  $(partner demo-dedup "$n" "$e" "$ph" "$ci")  $n"
done
echo "-- demo-dedup duplicate leads"
for n in "Inventory module for Kroon" "Inventory module for Kroon (web form)"; do
  echo "  $(lead demo-dedup "$n" "Brouwerij De Kroon BV" "Pieter Janssens" "info@dekroon.example.com" "+32 3 555 0201" "Antwerp" "Wants stock management across three sites.")  $n"
done

echo "-- demo-gdpr requester"
GP="$(partner demo-gdpr "demo GDPR requester" "gdpr.requester@example.com" "+32 2 555 0301" "Brussels" false)"
echo "  partner $GP"
GL="$(find_or_create crm.lead "[\"name\",\"=\",\"demo GDPR requester - consulting enquiry\"],[\"tag_ids\",\"in\",[$(ctag demo-gdpr)]]" \
  "$(jq -n --argjson p "$GP" --argjson t "$(ctag demo-gdpr)" \
    '{name:"demo GDPR requester - consulting enquiry",type:"opportunity",partner_id:$p,email_from:"gdpr.requester@example.com",phone:"+32 2 555 0301",tag_ids:[[6,0,[$t]]]}')")"
echo "  lead $GL"
msgs="$(kw mail.message search_count "[[[\"model\",\"=\",\"res.partner\"],[\"res_id\",\"=\",$GP],[\"message_type\",\"=\",\"comment\"]]]")"
if [ "$msgs" -lt 2 ]; then
  kw res.partner message_post "[[$GP]]" '{"body":"Customer called asking about their invoices and contract renewal.","message_type":"comment"}' >/dev/null
  kw res.partner message_post "[[$GP]]" '{"body":"Customer emailed: please erase all my personal data under GDPR Art. 17.","message_type":"comment"}' >/dev/null
fi
echo "  messages: $(kw mail.message search_count "[[[\"model\",\"=\",\"res.partner\"],[\"res_id\",\"=\",$GP],[\"message_type\",\"=\",\"comment\"]]]")"

invoice() { # invoice <ref> <post:true|false> -> id
  local id
  id="$(kw account.move search "[[[\"ref\",\"=\",\"$1\"],[\"partner_id\",\"=\",$GP]]]" | jq '.[0] // empty')"
  if [ -z "$id" ]; then
    id="$(kw account.move create "[$(jq -n --argjson p "$GP" --arg r "$1" \
      '{move_type:"out_invoice",partner_id:$p,ref:$r,invoice_line_ids:[[0,0,{name:"demo consulting day",quantity:1,price_unit:850}]]}')]")"
    [ "$2" = "true" ] && kw account.move action_post "[[$id]]" >/dev/null
  fi
  echo "$id"
}
echo "  draft invoice  $(invoice demo-gdpr-draft false)"
echo "  posted invoice $(invoice demo-gdpr-posted true)"

echo "-- counts"
for t in $TAGS; do
  echo "  $t: leads=$(kw crm.lead search_count "[[[\"tag_ids\",\"in\",[$(ctag $t)]]]]") partners=$(kw res.partner search_count "[[[\"category_id\",\"in\",[$(pcat $t)]]]]")"
done
echo "  demo-gdpr invoices: $(kw account.move search_read "[[[\"partner_id\",\"=\",$GP]]]" '{"fields":["id","state","ref"]}')"
