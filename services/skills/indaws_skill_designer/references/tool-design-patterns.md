# Tool & Skill Design Patterns
Source: Thariq Shihipar (Claude Code lead, Anthropic) — "Seeing like an Agent" (2026-03)

## Core Principle
"Designing the tools for your models is as much an art as it is a science. The only reliable method is reading the model's outputs and watching where it struggles. You have to learn to see like an agent."

## Pattern 1: Match Tools to Model Capabilities
Don't give a model tools it can't use well. Like giving a student paper vs calculator vs computer:
- Simpler tools → more predictable but limited
- Complex tools → powerful but harder for model to use correctly

**Applied to skills:** A skill's scripts should match what Claude can actually compose. A 500-line library Claude can't understand is worse than 5 simple functions it chains together.

## Pattern 2: Subtract as Models Improve
"As model capabilities increase, the tools that your models once needed might now be constraining them."

Claude Code example: TodoWrite tool worked well initially. When models got smarter, it became the constraint — Claude stuck to the list instead of adapting. They removed it and replaced with Task tool for inter-agent coordination.

**Applied to skills:** Review skills periodically. Instructions that were needed 6 months ago may now railroad Claude. Remove constraints that smarter models no longer need.

## Pattern 3: Progressive Disclosure over Tool Proliferation
"Rather than providing context upfront through RAG, Claude excels at building its own context when given search tools."

Claude Code has ~20 tools and resists adding more. When new capability was needed, they created Agent Skills — recursive file discovery without new tools. More capability, zero tools added.

**Applied to skills:** Don't put everything in SKILL.md. Point to references/ and let Claude discover what it needs. The file system IS the progressive disclosure mechanism.

## Pattern 4: Fewer, More Expressive Tools Win
Apple's CodeAct research: a single code execution primitive outperformed specialized tool collections by up to 20% on complex tasks. Deep tools beat broad menus.

**Applied to skills:** One good script that Claude can compose > 10 narrow scripts for specific cases. Give Claude building blocks, not pre-built solutions.

## Pattern 5: Structured Output for Elicitation
Claude Code team iterated 3 times on the AskUserQuestion tool:
1. Adding questions to ExitPlanTool → confusion about conflicting outputs
2. Markdown formatting instructions → unreliable ("Claude would append extra sentences, omit options")
3. Dedicated AskUserQuestion tool with structured output → worked

**Applied to skills:** When a skill needs user input, use structured tools (AskUserQuestion) not free-form text. The model needs to know EXACTLY what format the input should be in.

## Pattern 6: Design for Model Observation
"Tool effectiveness depends on whether the model actually uses it correctly, not just theoretical design."

The only way to know if a tool works is to watch Claude use it. Read the outputs. See where it struggles. Iterate.

**Applied to skills:** After publishing a skill, watch the first 10 uses. Where does Claude misinterpret? Where does it skip steps? Those are your gotchas.
