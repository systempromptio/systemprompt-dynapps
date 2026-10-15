# Evidence behind each rule (maintainers only — never loaded at runtime)

Strength: **S** strong (meta-analyses or replications), **M** moderate, **W** weak or practitioner consensus.
Change a rule only if you can point to better evidence or to a failing eval case.

| Rule in easy-read | Evidence | Strength |
|---|---|---|
| Ask and deadline first; findings before detail (BLUF, inverted pyramid, Minto) | AR 25-50 (BLUF); Nielsen 1996 inverted pyramid; Hartley, structured abstracts (~31 studies) | W–M |
| Cut padding and promotional tone | Morkes & Nielsen 1997: concise +58%, objective +27%, combined +124% usability. Mayer coherence principle d≈0.86 | M–S |
| Headings state the finding | NN/g layer-cake scanning (Pernice 2017); signaling meta-analysis, Schneider et al. 2018, g=0.53 | M–S |
| L1 / L2 / L3 layers, L2 collapsible | Progressive disclosure (Nielsen 2006); segmenting d≈0.79; expertise reversal (Kalyuga et al. 2003) | M–S |
| Domain terms allowed, no glossary for experts | Expertise reversal effect (Kalyuga et al. 2003) | S |
| STE-lite: ≤20/25 words, one idea per sentence, active voice, one term per concept | ASD-STE100 Issue 9 (2025); Shubert et al. 1995: comprehension better, and the gain looked larger for L2 readers | M |
| One term per concept, explicit connectives (not Flesch scores) | Crossley et al. 2017: cohesion and word frequency predict comprehension better than formulas | S |
| Claim table: quote + location per claim | Liu et al. 2023: only 51.5% of generative-search sentences are fully supported by their citations. HoT 2025: linking statements to source facts cut verification time ~25% | S / M |
| Separate Fact / Inference / Opinion | ICD 203: distinguish underlying information from assumptions and judgments | M |
| Never invent probabilities; keep the source hedge; ICD terms only next to source numbers | Kent 1964 ("probable" read as 30–75%); Budescu et al. 2009: verbal term + numeric range improves consistency (used only when the source gives the number) | S |
| Confidence as one word on the verdict only; flag only uncertain claims | Kim et al. FAccT 2024 (first-person uncertainty lowers overreliance); CHI 2025 "To Rely or Not to Rely?" (highlighting everything causes under-reliance) | M |
| Coverage "N de M datos comprobados", never a bare "verified" | Same as above, plus automation-bias concerns raised in the design review | M |
| Table for exact values, diagram only for process, dependency, org or timeline | Vessey 1991 (cognitive fit); Larkin & Simon 1987 (diagrams win through locational indexing) | S |
| Labels on the diagram itself | Mayer spatial contiguity d≈1.10 | S |
| Video only on request | Reading ~238 wpm (Brysbaert 2019) vs speech ~150 wpm; video is hard to scan or verify | M |
| Mechanical gate (check.py) instead of self-grading | LLMs miscount words and grade their own summaries leniently (design review). Liu et al. 2023 on unreliable self-attribution | M |
| Treat input as untrusted; quote embedded instructions | Prompt-injection risk in AI-generated documents (design review) | W |
| Give the shape, not only fewer words: mind map of findings in `html` | Concept/knowledge-map meta-analysis, Nesbit & Adesope 2006: maps beat reading text passages for retention (moderate effect). Larkin & Simon 1987 | M |
| Pick the visual by the content's shape (flow, timeline, breakdown, comparison) | Vessey 1991 cognitive fit; real use 2026-10-02: a process shown as a responsibility table was judged "muy complicado", the numbered prose sequence was right | S / W |
| One-sentence gist first | Gist-first comprehension (fuzzy-trace theory, Reyna & Brainerd 1995); BLUF | M |
| Findings must be statements, not topics | Real use 2026-10-02: branches "Hosting." / "Servicios." told the reader nothing; signaling meta-analysis (Schneider et al. 2018) | W / M |
| Worked example and glossary only for non-expert audiences | Worked-example effect (Sweller); expertise reversal (Kalyuga et al. 2003) | S |
| Status as ✓ / ✗ / ? symbols | Pre-attentive scanning of a small symbol set; six text statuses were read as jargon (design review round 4) | W |
| The problem exists | Niederhoffer, Hancock et al., HBR Sept 2025, "workslop": ~40% of workers received it in the last month, ~2 h rework each | M |

Not verified: the Nesbit & Adesope effect size (quoted from memory, check before citing a number), the "80% of the way to STE" heuristic and the speech-rate figure. Mayer's effect sizes come from a secondary summary.
