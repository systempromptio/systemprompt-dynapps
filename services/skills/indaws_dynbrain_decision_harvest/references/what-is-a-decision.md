# What counts as a Dyn Brain novelty

This is the hard part of the skill. If the filter is loose, the page fills with noise, the owner
opens it once and never comes back. **Discard generously.**

Most novelties are of type `decision` — somebody decided we work differently. The test below is
written for that case; the other five types in `decision-record.md` follow the same spirit, with
the authority condition met by the external fact rather than by a person.

---

## The test, in one sentence

> **Does it change what Dynapps will do by default next time, without anybody having to argue it
> again?**

If yes → it is a novelty.
If "it depends on the case" → it is not.

---

## The three conditions

**All three** must hold. Miss one and it is not a Dyn Brain novelty.

### 1 · It is prescriptive
It says what to do, not what we think or what happened.

✅ "A partir de ahora los GAP se cubren con desarrollo asistido por IA, no con Studio."
❌ "Studio nos está dando bastantes problemas últimamente."

### 2 · It is general, not about one case
It applies to every future case of its kind, not to one customer or project.

✅ "No aceptamos proyectos por debajo de 15.000 €."
❌ "Con Metapharma hacemos una excepción y bajamos a 12.000 €."

The second may be correct and even important — but it belongs to the project, not to the doctrine.

### 3 · Somebody with authority has closed it
The discussion is over. There is a person or a group who can decide it, and they have.

✅ "Julio y Desarrollo: no más Studio."
❌ "Estaría bien dejar de usar Studio, ¿lo hablamos?"

Watch the conditional and the subjunctive: *"deberíamos"*, *"habría que"*, *"estaría bien"* almost
always mark a proposal, not a decision.

---

## Signals that it is one

In the language:

- "A partir de ahora…", "de aquí en adelante", "ya no…", "dejamos de…"
- "Queda decidido", "lo cerramos así", "pues hacemos X", "adelante con X"
- "Por defecto…", "salvo excepción…", "como norma…"
- A change in a criterion, a price, a threshold, a limit or a priority
- Somebody corrects an established practice in front of others and nobody argues

In the context:

- The conversation ends afterwards: nobody keeps arguing
- Who applies it, or from when, gets mentioned
- It shows up in several sources the same week (a meeting, then Teams) → high confidence

---

## Signals that it is not

- **Conditional or uncertain future**: "deberíamos", "habría que", "cuando podamos"
- **A task**: "Andrea actualiza la ficha" is work, not doctrine
- **One specific case**: a customer or project name inside the decision itself
- **A personal preference**: "yo lo haría así"
- **Information**: "Odoo 19 ya soporta X" is a fact, not a decision — it may cause one, but it is
  not one. (If the fact itself changes what we do, it is an `external_source` novelty.)
- **Still under debate**: two positions and the conversation does not close
- **Zero scope**: true, but changes nothing we were already doing

---

## Borderline cases

**"Volvemos a lo de antes."** It is a novelty, and an important one: it reverts another. Mark it as
such and link to the one it supersedes.

**"De momento, hasta que veamos."** It is a **provisional** novelty. Register it with an explicit
expiry date. The provisional nobody reviews is what does the most damage.

**A novelty taken outside the owner's territory.** If Development decides something affecting
Sales and you detect it in Víctor's session: propose it anyway, but **the confirmer must be whoever
took it**. Mark the candidate as "pending confirmation by Julio". Nobody registers other people's
decisions.

**It was decided months ago and nobody registered it.** Valid. Use the real date, not today's, and
flag that it is retroactive — the scope may be far larger than usual.

**Two novelties contradicting each other in the same week.** Do not choose. Present them together,
with both pieces of evidence, and let the owner resolve it. That is valuable information: it means
two parts of the house believe different things.

---

## Confidence levels

| Level | When | What to do |
|---|---|---|
| **High** | The three conditions clear + literal evidence + recognisable authority | Candidate to publish |
| **Medium** | The three conditions, but the evidence is indirect or from a single source | Candidate, marked "review wording" |
| **Low** | A condition is missing or the language is ambiguous | **Do not propose it.** At the foot of the page, in a "we saw this, is it anything?" list with no record attached |

That final low-confidence list has a concrete purpose: it trains the filter. If the owner rescues
something from it two weeks running, the criterion is miscalibrated and needs adjusting.

---

## The field that really costs: `invalidates_terms`

The terms which, appearing in a document in a prescriptive tone, probably leave it stale. It is
what makes Monday's sweep find anything.

Think about **how whoever wrote the old documentation would have put it**, not about how the
novelty is stated.

Real example: the novelty is "we stop proposing Odoo Studio".

✅ `["Studio", "reconstruir en Studio", "personalización sin código", "sin programar"]`
❌ `["desarrollo asistido por IA"]` — that is the new thing; it does not appear in the old
   documentation.

And always anticipate the false positives. Here: `x_studio_price` and friends are field names that
genuinely exist in the database. Note them in the record so the sweep does not propose deleting
them — an agent told to "remove Studio" would have edited production.

One mechanical detail worth knowing, because it decides how you write these terms: the sweep
matches **by whole word, ignoring accents**. So `Studio` does **not** match the Spanish word
"estudio", and it does not match inside `x_studio_price` either, because the underscore counts as
a word character. You do not need to defend against those two; you do need to think about real
synonyms.

**If you cannot write these terms, the novelty is not ready to publish.**
