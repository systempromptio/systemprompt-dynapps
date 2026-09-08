## What the user asked for — ISD diagram

> project-context result metadata — source_type: `user request`

Beyond generating the ISD, the architect asked for one extra diagram in a
specific section:

> "In the ISD, under the Geocoding & Proximity integration point, add a small
> flow diagram that makes the resolution outcome obvious: a shopper's postcode
> or city goes to the capability, and depending on whether Cartora can resolve
> it the shopper either gets proximity-ranked results or a clear error. The
> sequence at the top of the doc already shows the call order — here I want the
> branch (resolved vs unresolvable) called out visually inside that section."

This is a request for an activity-style flow diagram (one decision with two
outcomes) in that one sub-section, IN ADDITION to the mandatory Integration
Flow Overview sequence — not a replacement for it. It adds no scope: the
resolvable / unresolvable behaviour is already bounded by this change's delta
specs (an unresolvable input yields a clear error result instead of
coordinates); the diagram only visualises that existing branch. No other
section was requested to carry a diagram.
