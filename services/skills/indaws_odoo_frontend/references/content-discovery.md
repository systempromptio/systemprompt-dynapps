# Content Discovery (Phase 1)

Ask the user via AskUserQuestion to understand requirements.

## Step 1.1: Context Questions

**Question 1 -- Purpose:**
- Header: "Purpose"
- "What is this content for?"
- Options: "Website landing page", "Blog article", "Email campaign", "Presentation / Pitch deck"

**Question 2 -- Content readiness:**
- Header: "Content"
- "Do you have the content ready?"
- Options: "Content ready (text/images)", "Rough notes to organize", "Topic only -- help me create it"

**Question 3 -- Images:**
- Header: "Images"
- "Do you have images for this content?"
- Options: "Yes -- I'll provide URLs or paths", "Use free stock photos (Unsplash)", "Generate descriptive placeholders for now"

**Question 4 -- Audience:**
- Header: "Audience"
- "Who is the target audience?"
- Options: "Potential clients (commercial)", "Existing clients (retention)", "Internal team", "General public"

If the user has content or images, ask them to share before proceeding.

## Step 1.2: Image Discovery

Based on the user's image choice:

- **"Yes -- I'll provide URLs or paths"**: Ask user to provide image files or URLs, note their purpose (hero, section 1, etc.)
- **"Use free stock photos (Unsplash)"**: Read `references/image-workflow.md` > "Unsplash API" section, search for 3-5 content-relevant keywords, present top 3 results to user
- **"Generate descriptive placeholders"**: Use inline SVG placeholders with brand colors (see `references/image-workflow.md` > "Descriptive Placeholders")

**IMPORTANT:** Never use generic placeholder URLs (placeholder.com, lorempixel). Always use semantic inline SVG placeholders with Indaws brand colors and descriptive labels.

## Presentation-Only Questions

**Additional question -- Length:**
- Header: "Length"
- "How many slides?"
- Options: "Short (5-10)", "Medium (10-20)", "Long (20+)"

**Additional question -- Mood:**
- Header: "Vibe"
- "What feeling should the audience have?"
- Options: "Impressed/Confident", "Excited/Energized", "Calm/Focused", "Inspired/Moved"
- multiSelect: true (up to 2)
