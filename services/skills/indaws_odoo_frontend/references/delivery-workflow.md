# Delivery Workflow (Phase 5)

## Step 5.1: Generate Preview

Read `references/preview-workflow.md` for the complete preview system. Summary:

1. Create the preview directory: `mkdir -p /tmp/iw-preview`
2. **Blogs / Landing pages** (HTML fragments): wrap in the preview template -- read `assets/css/indaws-frontend.css` and `assets/js/iw-reveal.js`, embed them in a full HTML document with Bootstrap 5 CDN, paste the content fragment in `<body>`
3. **Emails** (full HTML documents): save the generated HTML directly (already has inline styles and document structure)
4. **Presentations** (self-contained files): save as-is (already has embedded CSS/JS)
5. Write to `/tmp/iw-preview/[type]-preview.html`
6. Open in browser: `open /tmp/iw-preview/[type]-preview.html`

## Step 5.2: Review & Iterate

Ask the user via AskUserQuestion:
- Header: "Preview"
- "How does the preview look?"
- Options: "Looks good -- proceed to delivery", "Needs adjustments", "Start over with different approach"

If adjustments needed: ask what to change, apply edits, regenerate preview, re-open. Repeat until approved.

## Step 5.3: Create Image Manifest

Before final delivery, create an Image Manifest following the template in `references/image-workflow.md` > "Image Manifest Template". This manifest lists:

- All images in the content (numbered)
- Description and dimensions of each
- Source (user-provided, Unsplash with attribution, or placeholder)
- Status (ready or replace needed)
- Replacement instructions for placeholders

Include the manifest as a markdown section in your delivery message to the user.

## Step 5.4: Deliver

Provide a summary of what was created:
- Content type and structure
- Word count / slide count
- Number of images (ready vs. placeholders)
- Any Unsplash attributions required

Then offer publishing options per content type.

### Website Pages / Landing Pages

The output is a single self-contained HTML file with CSS inlined in `<style>`. Tell the user:
- The file is ready to paste inside their QWeb template `<t t-call="website.layout">`
- They need to upload images to Odoo and replace `/web/image/placeholder.png` paths with actual Odoo attachment paths (e.g., `/web/image/ir.attachment/123/datas`)
- List all image placeholders with their intended screenshot/image descriptions

### Blog Publishing via odoo-pilot

For blog posts, ask the user whether to publish the blog as a draft in Odoo:

1. Use the `odoo-pilot` skill to authenticate and create a `blog.post` record with `is_published: false`
2. Send the full HTML (BLUF through Schema) as the `content` field
3. Set `website_meta_title` and `website_meta_description` from the SEO data
4. Provide the user with the Odoo backend URL and preview URL for revision

See `references/blog-creation.md` > "Publishing to Odoo" for the full workflow, field mapping, and odoo-pilot script usage.

### Email Publishing via odoo-pilot

For emails, ask the user whether to create the mailing as a draft in Odoo:

1. Use the `odoo-pilot` skill to authenticate and create a `mailing.mailing` record
2. Send the full HTML as the `body_html` field
3. Set `subject` from the email subject line
4. Optionally link to a `mailing.list` contact list
5. Provide the user with the Odoo backend URL for testing and sending

See `references/email-creation.md` > "Publishing to Odoo" for the full workflow, field mapping, and odoo-pilot script usage.

### Landing Page Publishing via odoo-pilot

For landing pages, ask the user whether to create the page in Odoo:

1. **Primary method**: Provide paste-ready self-contained HTML for the QWeb template (see `references/self-contained-output.md`)
2. **Advanced method**: Use the `odoo-pilot` skill to create a `website.page` record programmatically with `is_published: false`

See `references/landing-page-creation.md` > "Publishing to Odoo" for both methods.
