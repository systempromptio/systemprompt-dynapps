# Lens catalog — artifacts

Process lenses (constraint, handoff audit, FMEA, exception load, Monday morning, blast radius,
instrumentation, 10× load, incentive alignment) live in `process-descent.md` §5.

Each round uses **2 lenses**, never repeating until the catalog is exhausted.
Rotation is the stochastic part of the method: each lens produces a
differently-biased gradient estimate, and averaging over biased estimates is
what prevents overfitting to a single critic's taste.

## The contract every lens must satisfy

A lens does not produce prose. It produces findings in this shape:

```
knob:      <knob name, must exist in rubric.md>
direction: increase | decrease | restructure
magnitude: 0.1–1.0   (expected ΔS if fully applied)
location:  <line ref or quoted anchor>
why:       <one sentence, tied to a rubric anchor>
```

A finding that cannot name a knob is **rejected** and logged as a possible
rubric gap. This is the difference between a gradient and a complaint.

## Catalog

### 1. Steel-man
Construct the strongest good-faith objection a smart, hostile reader would
raise. Not nitpicks — the objection that would actually lose the deal or sink
the argument. Then locate where the artifact fails to pre-empt it.

### 2. First-thirty-seconds
Read only the title, first paragraph, and any headings. Decide what the reader
now believes the artifact is about and whether they would keep reading. Gradient
points at opening structure, title, and the promise-to-payoff gap.

### 3. Evidence audit
List every claim. Mark each: proven in-text / provable but unproven / assertion.
Gradient is the ratio and the position of the unproven ones — an assertion in
the opening costs more than one in an appendix.

### 4. Spine
Ignore wording entirely. Extract the skeleton: what is the sequence of moves?
Does each section earn its position? Would swapping two sections improve or
damage the argument? Gradient points at `section-order` and at sections that
justify their own existence only weakly.

### 5. Precision
Hunt vague quantifiers ("significant", "several", "robust"), hedges ("may",
"could potentially"), and filler. Each is a place where a specific claim was
available and not made. Gradient magnitude scales with how load-bearing the
vague phrase is.

### 6. So-what
For each section, ask what decision it changes for the reader. Sections that
change no decision are candidates for cutting regardless of quality. Gradient
often points at `compression` and `cta-specificity`.

### 7. Alternative framing
Assume the reader is comparing this against the obvious alternative (a
competitor, the status quo, doing nothing). Is the comparison made explicitly,
implicitly, or avoided? Avoided comparisons read as weakness.

### 8. Compression
What can be deleted with zero loss of meaning? Target a specific percentage
(start at 15%). Anything that survives a genuine 15% cut attempt was
load-bearing; anything cut was decoration.

## Pairing guidance

Pair one **structural** lens (2, 4, 6, 7) with one **local** lens (1, 3, 5, 8)
in early rounds — this gives the gradient both a direction and a step.

From round 4 on, when η is low and structural edits are forbidden, use two
local lenses. Assigning a structural lens at low η produces findings you are
not permitted to act on, which wastes the round.
