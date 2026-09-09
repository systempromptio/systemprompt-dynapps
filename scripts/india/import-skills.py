#!/usr/bin/env python3
"""Reproduce the India skill migration from sibling, pinned source checkouts."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import yaml

ROOT = Path(__file__).resolve().parents[2]
SOURCES = [
    ('ba', 'mcp-ba-platform', 'mcp-ba-server/src/db/seedContent/skills', 'astound-india-ba'),
    ('sf', 'sf-dev-ai-platform', 'mcp-dev-platform/content/sf-core-skills/skills', 'astound-india-sfcore'),
]
# These are workspace operations, not fictitious MCP tool names.
ADAPTERS = {
    'read_csv_tracker': 'read the named CSV in .astound/ba/trackers using workspace file tools',
    'write_csv_tracker': 'append or update the named CSV in .astound/ba/trackers using workspace file tools',
    'read_project_file': 'read the named project document using workspace file tools',
    'write_project_file': 'write the named document in .astound/ba/documents using workspace file tools',
    'draft_ticket': 'save the draft as Markdown in .astound/ba/drafts using workspace file tools',
    'manage_drafts': 'list, read, or edit .astound/ba/drafts using workspace file tools',
    'push_to_jira': 'Jira issue creation (`discover`, then `executeWrite`)',
    'read_transcript': 'read the transcript from .astound/ba/transcripts using workspace file tools',
    'write_transcript': 'create a new Markdown transcript in .astound/ba/transcripts using workspace file tools',
    'manage_session': 'manage named Markdown session files in .astound/ba/sessions using workspace file tools',
    'store_correction': 'append a correction to the session corrections.jsonl using workspace file tools',
    'analyze_impact': 'analyze the retrieved scope and Jira evidence using the output structure below',
    'trace_requirement': 'compare retrieved requirements with Jira evidence and the local RTM',
    'get_backlog_stats': 'Jira issue search by JQL (`discover`, then `executeRead`)',
    'get_sprint_issues': 'Jira issue search by JQL (`discover`, then `executeRead`)',
    'bulk_update_jira': 'Jira issue update (`discover`, then `executeWrite`)',
    'validate_release': 'evaluate retrieved Jira issues against the project release checklist',
    'check_release_readiness': 'summarize evidenced release checks and unresolved blockers',
    'trigger_scheduled_job': 'prepare an on-demand report and a schedule proposal in .astound/ba/reports; scheduling is unsupported',
    'send_slack_message': 'prepare an unsent Slack draft in .astound/ba/notifications; sending is unsupported',
    'send_email': 'prepare an unsent email draft in .astound/ba/notifications; sending is unsupported',
}
DISCOVER = ['get_board_info', 'get_sprint_info', 'get_jira_versions', 'create_jira_version',
            'create_sprint', 'assign_to_sprint', 'add_jira_attachment', 'get_jira_changelog',
            'get_recent_jira_events', 'get_jira_event_summary']
# Atlassian v2 exposes primary tools plus discover/executeRead/executeWrite; the
# upstream bodies name Rovo-era tools, which are rewritten to capabilities.
LEGACY_TOOLS = {
    'getVisibleJiraProjects': 'Jira project listing (`discover`, then `executeRead`)',
    'getJiraIssue': 'Jira issue read (`discover`, then `executeRead`)',
    'searchJiraIssuesUsingJql': 'Jira issue search by JQL (`discover`, then `executeRead`)',
    'getTransitionsForJiraIssue': 'Jira transition listing (`discover`, then `executeRead`)',
    'getIssueLinkTypes': 'Jira link-type listing (`discover`, then `executeRead`)',
    'lookupJiraAccountId': 'Jira account lookup (`discover`, then `executeRead`)',
    'atlassianUserInfo': 'Atlassian current-user read (`discover`, then `executeRead`)',
    'createJiraIssue': 'Jira issue creation (`discover`, then `executeWrite`)',
    'editJiraIssue': 'Jira issue update (`discover`, then `executeWrite`)',
    'transitionJiraIssue': 'Jira issue transition (`discover`, then `executeWrite`)',
    'addCommentToJiraIssue': 'Jira issue comment (`discover`, then `executeWrite`)',
    'createIssueLink': 'Jira issue link creation (`discover`, then `executeWrite`)',
    'searchConfluenceUsingCql': 'Confluence search by CQL (`discover`, then `executeRead`)',
    'getConfluencePage': 'Confluence page read (`discover`, then `executeRead`)',
    'getConfluenceSpaces': 'Confluence space listing (`discover`, then `executeRead`)',
    'createConfluencePage': 'Confluence page creation (`discover`, then `executeWrite`)',
    'updateConfluencePage': 'Confluence page update (`discover`, then `executeWrite`)',
}
COMMON = '''## Systemprompt execution contract

Workspace access installs these skills. The `atlassian`, `salesforce`, and
`github` connectors are delivered separately after the signed-in user connects
and verifies each provider account. An installed skill does not prove a connector
is ready. Local file tasks can run without provider connections. For remote
operations, use only the currently advertised tools and input schemas. Never request, print, or copy credentials, and never bypass Systemprompt
with direct provider HTTP requests. Missing tools or rejected authentication are
failures: name the connector and stop the affected operation. Do not substitute
mock data or report an unexecuted action as successful.

Use the current repository as project context. Confirm unknown project identifiers
from authenticated reads; never invent an org, Jira key, schema, or repository.
Respect the user's existing authorization and the review requirements below.
For provider writes, use the format accepted by the advertised input schema.

Atlassian v2 exposes primary tools plus `discover` and `executeRead`/`executeWrite`.
Inspect live schemas; do not assume legacy Jira or Confluence tool names.

'''
BA = '''## Workspace adaptation

Custom BA platform operations have been replaced with workspace file operations
or the `atlassian` connector (`discover`, `executeRead`, `executeWrite`). Paths are relative to the current repository.
Keep drafts, trackers, sessions, transcripts, and reports beneath `.astound/ba/`;
report the actual saved path as the local identifier. Read existing files first,
use proper CSV quoting, preserve existing rows, and avoid concurrent overwrites.
Missing required trackers or project standards are missing inputs, not empty
sources: stop and request the missing input. Never invent their contents.

Local session corrections persist only in this workspace and must be explicitly
read on later sessions. They do not modify global memory. Transcripts are local
Markdown files, not rows in the retired BA backend. For bulk Jira edits, show all
changes first, apply approved edits individually, and report partial failures.
For story creation, apply the writing checks below before the Jira issue write.
Use its advertised content schema, not an assumed legacy wiki or ADF format.
Notification and scheduling operations produce drafts/proposals only; explicitly
report that sending or scheduling was not executed. Never invent a job history.

'''

def sha(data):
    return hashlib.sha256(data).hexdigest()

DESCRIPTIONS = {
    'ba_notification_sending': 'Prepare unsent Slack and email drafts for BA events and updates.',
    'ba_scheduled_reporting': 'Prepare on-demand BA reports and schedule proposals; jobs are not installed.',
    'ba_transcript_management': 'Store and retrieve discovery/interview transcripts as local workspace Markdown.',
}


def adapt_reviewed_content(skill_id, body):
    body = body.replace('The marketplace provisions authenticated `atlassian`, `salesforce`, and `github`\nMCP connectors for the signed-in user. Use their advertised tools and input\nschemas.', 'Workspace access installs these skills. The `atlassian`, `salesforce`, and\n`github` connectors are delivered separately after the signed-in user connects\nand verifies each provider account. An installed skill does not prove a connector\nis ready. Local file tasks can run without provider connections. For remote\noperations, use only the currently advertised tools and input schemas.')
    """Keep imported instructions consistent with the hosts and capabilities shipped."""
    body = re.sub(
        r'\.cursor/skills/([A-Za-z_]+)\.md',
        lambda m: 'the installed sf-' + m[1].lower().replace('_', '-') + ' skill',
        body,
    )
    body = body.replace(' and the `Apex_Architect` agent', '')
    body = body.replace('Fetch a transcript by numeric ID or exact title',
                        'Fetch a transcript by saved workspace path or exact title')
    body = body.replace('Create a BA-standard story with Gherkin guard + ADF conversion (approval required)',
                        'Create a BA-standard story after local writing checks using the advertised content schema (approval required)')
    body = body.replace('Update multiple issues in one call (approval required)',
                        'Update approved issues individually and report partial failures (approval required)')
    if skill_id == 'ba_notification_sending':
        body = body.replace('Send Slack messages and emails for BA events and updates.', DESCRIPTIONS[skill_id])
        body = body.replace('Send a message to a Slack channel or reply in a thread', 'Save an unsent Slack message or thread-reply draft')
        body = body.replace('Send an email via the configured Resend integration', 'Save an unsent email draft with recipient, subject and body')
        body = body.replace('## Pre-send checklist', '## Draft review checklist')
        body = body.replace('for review before sending', 'for review; sending is unsupported')
    if skill_id == 'ba_transcript_management':
        body = body.replace('Store and retrieve discovery/interview transcripts from the BA platform.', DESCRIPTIONS[skill_id])
    if skill_id == 'ba_scheduled_reporting':
        body = body.replace('Trigger or inspect standup, backlog grooming, and release-notes scheduled jobs.', DESCRIPTIONS[skill_id])
        body = body.replace('Trigger a job or inspect its status/history', 'Save an on-demand report or schedule proposal; no job execution or history')
        start = body.find('## prepare an on-demand report')
        if start != -1:
            body = body[:start] + '''## Report and proposal actions

- Generate a report from authenticated Jira reads, then save it locally.
- Save a proposed schedule with its time zone; do not activate a job.
- For status/history requests, explain that this marketplace installs no jobs
  and has no execution history. Do not invent a running job or previous result.

## Guardrails
- Verify the target project through authenticated reads before preparing a report.
- Inspect advertised read capabilities; report unavailable activity feeds explicitly.
- Save report drafts locally. Sending and unattended scheduling are unsupported.
'''
    if skill_id == "sf_app_utilitybar_pattern" and '## Required utility-bar delivery checks' not in body:
        body += '\n\n## Required utility-bar delivery checks\n\nBefore finishing, include these checks in the saved repair plan, even for a local\nfixture exercise:\n- Retrieve and inspect the full target profile before editing visibility; preserve\n  unrelated permissions. If retrieval is unavailable, label profile XML as a\n  proposed fragment and leave deployment blocked.\n- Show matching app and utility-bar names and their deployment dependency.\n- Distinguish XML shown in a plan from files actually written. List only actual\n  saved paths as generated files. Do not claim app visibility, deployment or\n  runtime verification without authenticated evidence.\n'
    if skill_id == "sf_bypass_patterns" and "## Required bypass review checks" not in body:
        body += '\n\n## Required bypass review checks\n\nKeep three concerns separate: a custom permission for an authorized business\nbypass, an optional test-only override, and a production recursion guard. Check\nthe recursion flag in every execution, not only inside `Test.isRunningTest()`;\ninitialize it to false and reset it in a finally block. Do not leave SOQL or DML\ninside loops when adapting a supplied handler.\n\nInclude test scenarios for a user with the permission, an ordinary user without\nit, and a recursive production call. State which scenarios are only proposed.\nDo not claim a test ran without its execution output.\n'
    if skill_id == "sf_custom_metadata_driven_config" and "## Required evaluator review checks" not in body:
        body += '\n\n## Required evaluator review checks\n\nState the supported comparison operators and reject unsupported or malformed\nconfiguration explicitly. Sort rules by ascending priority and a stable unique\nname; retain the first matching rule. Do not overwrite the winning rule with a\nlater entry when populating a map. Make the no-match result explicit without\nassuming a queue or custom object exists in the target org.\n\nSave proposed tests covering a match, no match, equal-priority tie, and invalid\nconfiguration. Review the code against those examples before returning it. Label\nmetadata names as proposed when they have not been confirmed in the project.\n'
    if skill_id == 'sf_core_logic' and '## Required handler review checks' not in body:
        body += '\n\n## Required handler review checks\n\nFor a corrected handler, preserve least privilege: use an explicit sharing mode\nappropriate to the caller and enforce object/field permissions for user-initiated\noperations. Do not assume the caller is an administrator to justify missing FLS.\nA proposed system-mode exception needs explicit project authorization; otherwise\nleave it unresolved. A recursion flag must be checked, set and reset around the\nwork. Distinguish before-trigger field assignment from after-trigger DML; never\nupdate read-only trigger records in place. Review the example for recursive DML\nand for authorization checks, not merely for moving DML outside a loop.\n'
    if skill_id == 'sf_devops' and '## Required deployment review checks' not in body:
        body += '\n\n## Required deployment review checks\n\nFor trigger or sharing changes and production promotion, state RunLocalTests\nexplicitly. RunSpecifiedTests is not a substitute for that requirement in this\nskill. Use the supplied API version and exact component names; mark unknown\nmetadata as proposed. Include a validation step, retained test/deploy evidence\nand a rollback plan. Local fixture output is an unexecuted runbook; do not claim\nvalidation or deployment has run. Save the actual package manifest alongside the\nrunbook and list the saved paths.\n'
    if skill_id == "sf_dynamic_forms_field_visibility" and "## Required visibility review checks" not in body:
        body += '\n\n## Required visibility review checks\n\nUse only statuses and fields supplied by the current project; do not add a new\nstatus or treat an example field/action as existing org metadata. If Dynamic\nActions support is unknown, label action visibility as a conditional design that\nrequires verification in Lightning App Builder; do not present it as deployed.\nField visibility is not a replacement for FLS or record access.\n\nFor a local fixture exercise, every QA result must say NOT RUN. Give expected\noutcomes separately. Never print PASS ticks for proposed UI tests. Include the\nunknown platform support and verification step in the saved summary.\n'
    if skill_id == "sf_profile_permissionset_patterns" and "## Required permission review checks" not in body:
        body += '\n\n## Required permission review checks\n\nSave explicit, unexecuted persona test cases: the intended reviewer can read/edit\nonly the requested fields; an ordinary user without the assignment does not gain\nthat access; Delete, View All and Modify All remain unavailable. Distinguish\npermission-set definitions, group membership and user assignment.\n\nExplain app visibility separately. If a profile must be edited, retrieve its full\ncurrent definition first and preserve unrelated permissions; otherwise state that\nno profile edit is proposed. Label metadata snippets as proposed and list only\nfiles actually written. Do not claim permissions were verified in an org.\n'
    if skill_id == "sf_quick_action_flow_pattern" and "## Required quick-action review checks" not in body:
        body += '\n\n## Required quick-action review checks\n\nDescribe field-level and object access for the user launching the flow, not just\nbutton visibility. Keep the input recordId, flow/action references and qualified\nlayout action name consistent. A screen flow must retrieve the record using its\ninput ID before referencing record fields; do not assume a record-triggered\n$Record variable exists in that screen flow.\n\nFor local snippets, label XML as an unvalidated proposal. Explain the metadata\nvalidation and runtime checks still needed; do not say ready for deployment or\nverified without those results. Include first-screen navigation and least-\nprivilege access checks in the saved summary.\n'
    if skill_id == 'sf_email_notification_pattern' and '## Required notification review checks' not in body:
        body += '\n\n## Required notification review checks\n\nMake recipient selection and all unknown provider/template/queue configuration\nexplicit. Include proposed test cases for a successful send, missing recipient,\nand an actual send failure (for example a rejected address or provider error).\nDescribe the failure/fault path separately from a guard that skips sending.\nFor local fixtures, mark every test NOT RUN and delivery unverified. Do not\nclaim a draft was sent or an email configuration was validated.\n'
    if skill_id == 'sf_ui_framework' and '## Required UI review checks' not in body:
        body += '\n\n## Required UI review checks\n\nUse only fields supplied by the current project schema. If FirstName and\nLastName are supplied but Name is not, build the display value from those supplied\nfields or request verification; do not silently expand the assumed schema.\nInclude proposed loading, error and keyboard tests with expected outcomes.\nLabel tests NOT RUN unless a test runner actually executed them. SLDS use alone\nis not evidence that this component passed contrast or accessibility testing;\nstate the remaining verification steps.\n'
    if skill_id == "sf_integration" and "## Required integration review checks" not in body:
        body += '\n\n## Required integration review checks\n\nEnforce the stated batch bound in the actual code. A BATCH_SIZE constant that is\nnever used does not bound callouts. Split or chain remaining work explicitly and\naccount for retries within the chosen per-job limit.\n\nShow how the idempotency key reaches the provider request (when the provider\nsupports such a contract) or how it is checked in a durable deduplication store.\nLogging a key is not idempotency. If provider support is unknown, state that\nverification requirement instead of claiming duplicate suppression.\n\nInclude proposed tests for oversized input, retries, and repeated requests in\naddition to success, timeout and error responses. Assert observable behavior,\nnot merely that a key is deterministic. Mark tests unexecuted in local fixtures.\n'
    return body


def adapt_reference(body, kind):
    marker = '> Astound marketplace adaptation:'
    if marker in body:
        return body
    if kind == 'rules':
        body = adapt_reviewed_content('reference', body)
        body = body.replace('.cursor/rules/story-intake.md', 'story-intake.md (beside this file)')
        body = body.replace('`docs/BRANCH_AND_DEPLOY_PRACTICE.md`', '`../docs/BRANCH_AND_DEPLOY_PRACTICE.md` (relative to this file)')
        note = ('These are project-template defaults, subordinate to the current repository instructions and user authorization. '
                'The marketplace ships skills, not the named agent roster below: those names describe review perspectives '
                'for the current assistant, not agents to spawn. Load the corresponding installed skill. '
                'Do not create branches, deploy, send messages or claim QA execution merely because a template describes those steps. '
                'Use the current project\'s branching and approval rules; missing org access leaves live validation unverified.')
    else:
        note = ('Historical upstream project template, not Claude Code installation instructions or facts about your org. '
                'Cursor paths and agent rosters describe the source kit and are not installed by this marketplace. '
                'Use the installed sf-salesforce-project-context skill and the current repository instructions for this client.')
    return marker + ' ' + note + '\n\n' + body


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-root', type=Path, default=ROOT.parent)
    args = parser.parse_args()
    inventory = {'schema_version': 1, 'sources': [], 'skills': []}
    for prefix, repo, folder, plugin_id in SOURCES:
        checkout = args.source_root / repo
        commit = subprocess.check_output(['git', '-C', str(checkout), 'rev-parse', 'HEAD'], text=True).strip()
        url = subprocess.check_output(['git', '-C', str(checkout), 'remote', 'get-url', 'origin'], text=True).strip()
        inventory['sources'].append({'repository': url, 'commit': commit, 'directory': folder, 'plugin': plugin_id})
        config_path = ROOT / 'services/plugins' / plugin_id / 'config.yaml'
        config = yaml.safe_load(config_path.read_text())
        plugin = config['plugin']
        old_ids = plugin['skills']['include']
        new_ids = []
        for source in sorted((checkout / folder).glob('*.md')):
            raw = source.read_bytes()
            body = raw.decode()
            skill_id = prefix + '_' + re.sub(r'[^a-z0-9]+', '_', source.stem.lower()).strip('_')
            new_ids.append(skill_id)
            title = body.splitlines()[0].removeprefix('# Skill: ').strip()
            summary = re.search(r'^\*\*One-liner\*\*: (.+)$', body, re.M)
            description = summary.group(1) if summary else 'Use when implementing or reviewing ' + title + ' in Salesforce.'
            description = DESCRIPTIONS.get(skill_id, description)
            if prefix == 'ba':
                body = body.replace('`atlassian-rovo`', '`atlassian`')
                body = re.sub(r'^> \*\*Tool sources:\*\*.*$', '> Use the authenticated `atlassian` connector for remote operations and workspace tools for local artifacts.', body, flags=re.M)
                for old, new in ADAPTERS.items():
                    body = body.replace('`' + old + '`', '`' + new + '`' if re.match(r'^[a-zA-Z]+$', new) else new)
                for old in DISCOVER:
                    body = body.replace('`' + old + '`', old.replace('_', ' ') + ' via `discover` on `atlassian` (stop and report if no capability matches)')
                body = body.replace(' **(Rovo — approval required)**', ' (approval required)').replace(' **(Rovo)**', '')
                body = body.replace(' (Rovo MCP — explicit approval required)', ' (explicit approval required)').replace(' (Rovo MCP)', '')
                body = body.replace('| Tool | Purpose |', '| Capability | Purpose |')
                for old, new in LEGACY_TOOLS.items():
                    body = body.replace('`' + old + '`', new)
                body = body.replace('**(BA MCP)**', '(workspace adaptation)').replace('**(BA MCP — approval required)**', '(approval required)')
                body = body.replace('Store a new transcript in PostgreSQL', 'Store a new local Markdown transcript')
                body = body.replace('Transcripts are stored server-side in PostgreSQL and accessible only via the BA platform tools.', 'Transcripts are stored in this workspace; report the saved path and preserve prior entries.')
                body = body.replace('Corrections are stored permanently and influence future agent behaviour.', 'Corrections persist in this workspace; explicitly read them when resuming a session.')
                body = body.replace('Results are stored in `scheduled_reports` and sent via Slack/email if the integrations are configured.', 'Save report drafts locally. Sending and unattended scheduling are unsupported in this marketplace.')
                body = body.replace('## Available scheduled jobs', '## Schedule proposals (not installed jobs)')
                body = body.replace('Content must use Confluence wiki markup for proper rendering — not Markdown.', 'Use the content format required by the advertised Confluence tool schema.')
                body = body.replace('**Jira push**: Jira Wiki Markup (`*bold*`, `#` numbered lists, `*` bullets, double newline between sections)', '**Jira push**: Use the content format required by the advertised Jira tool schema.')
            if prefix == 'sf':
                body = body.replace('.cursor/skills/Core_Logic.md', 'the sf-core-logic skill')
                body = body.replace('.cursor/rules/always-active.md', 'the project standing rules (bundled reference in sf-salesforce-project-context)')
                body = body.replace('.cursor/rules/story-intake.md', 'the project story-intake rules (bundled reference in sf-salesforce-project-context)')
                if source.stem == 'Salesforce_Project_Context':
                    body += '\n## Bundled project templates\n\nRead references/rules/always-active.md and references/rules/story-intake.md. The bundled references/docs files are upstream templates, not verified facts about the current project. Resolve project identity from the current repository and authenticated Salesforce metadata before filling any placeholders. Never assume the example schema is the target org.\n'
            body = adapt_reviewed_content(skill_id, body)
            front = yaml.safe_dump({'name': skill_id.replace('_', '-'), 'description': description}, sort_keys=False).strip()
            output = '---\n' + front + '\n---\n\n' + COMMON + (BA if prefix == 'ba' else '') + body
            dest = ROOT / 'services/skills' / skill_id
            dest.mkdir(parents=True, exist_ok=True)
            (dest / 'SKILL.md').write_text(output)
            if prefix == 'sf' and source.stem == 'Salesforce_Project_Context':
                for supporting in ['docs', 'rules']:
                    shutil.copytree(source.parent.parent / supporting, dest / 'references' / supporting, dirs_exist_ok=True)
                    for reference in (dest / 'references' / supporting).glob('*.md'):
                        reference.write_text(adapt_reference(reference.read_text(), supporting))

            (dest / 'config.yaml').write_text(yaml.safe_dump({'id': skill_id, 'name': title, 'description': description, 'enabled': True, 'file': 'SKILL.md', 'tags': ['india', 'business-analysis' if prefix == 'ba' else 'salesforce'], 'category': 'business' if prefix == 'ba' else 'development', 'display_category': 'Business Analysis' if prefix == 'ba' else 'Salesforce Core'}, sort_keys=False))
            inventory['skills'].append({'id': skill_id, 'plugin': plugin_id, 'source': str(source.relative_to(checkout)), 'source_sha256': sha(raw), 'destination': str((dest/'SKILL.md').relative_to(ROOT)), 'destination_sha256': sha(output.encode()), 'adaptations': ['authenticated-mcp-contract', 'workspace-ba-operations'] if prefix == 'ba' else ['authenticated-mcp-contract']})
            inventory['skills'][-1]['adaptations'].append('claude-code-reference-and-capability-review')
            if (dest / 'references').exists():
                inventory['skills'][-1]['reference_sha256'] = {
                    str(p.relative_to(dest)): sha(p.read_bytes())
                    for p in sorted((dest / 'references').rglob('*')) if p.is_file()
                }
        for old in set(old_ids) - set(new_ids):
            shutil.rmtree(ROOT / 'services/skills' / old)
        plugin['skills'] = {'source': 'explicit', 'include': new_ids}
        plugin['mcp_servers'] = {'source': 'explicit', 'include': ['atlassian', 'salesforce', 'github']}
        plugin['version'] = '2.0.0'
        plugin['description'] = f"All {len(new_ids)} upstream {'business-analysis' if prefix == 'ba' else 'Salesforce Core'} skills, with authenticated Atlassian, Salesforce and GitHub connectors."
        config_path.write_text(yaml.safe_dump(config, sort_keys=False, allow_unicode=True))
    dest = ROOT/'docs/india-skills-inventory.json'
    dest.write_text(json.dumps(inventory, indent=2) + '\n')

if __name__ == '__main__':
    main()
