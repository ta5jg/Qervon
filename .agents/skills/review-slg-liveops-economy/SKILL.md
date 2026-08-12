---
name: review-slg-liveops-economy
description: Apply the SLG Economy and LiveOps Review workflow to any change touching queues, timers, progression, hero acquisition, alliances, or monetization; use it before proposing or validating a change.
---

# SLG Economy and LiveOps Review

Eclipse Dominion's target format (`GAME_CONCEPT.md`, `ROADMAP.md`, as of
2026-07-31) is a persistent, live-operated SLG in the Dark War class. In this
genre the retention mechanism and the monetization mechanism are the same
mechanism (`MONETIZATION.md`). This skill is the gate for whether an economy or
LiveOps change actually serves that model, and whether it earns its place
against genre incumbents rather than merely copying them — `GAME_CONCEPT.md`
explicitly lists "clone perception" as a thing to avoid.

## Procedure

1. **Fairness rules as code, not copy.** For any change touching combat odds,
   randomised acquisition (hero summons), pricing, ranked standings, or defeat
   consequences, verify the relevant fairness rule from `MONETIZATION.md` is
   enforced in the rules engine and asserted as a `scripts/simulate.js`
   invariant — not merely stated in a document or UI label:
   - Combat maths and odds are computed and shown before commitment, not
     revealed only after.
   - Randomised-purchase odds are disclosed in-client at the point of
     purchase, not in a separate document.
   - Pricing is legible: no layered/chained currencies whose function is to
     obscure real-money cost.
   - Ranked competition is bracketed so a non-paying account is not
     permanently ranked against maximum-spend accounts.
   - Defeat is costly but leaves a recoverable floor for a non-payer — verify
     the floor exists as a rule, not as an intention.
   - Spend limits / self-exclusion are reachable in-client where purchases
     exist.
2. **Return triggers are engineered, not assumed.** Every mechanism in
   `MONETIZATION.md`'s return-trigger table (queue completion, energy
   regeneration, alliance obligation, attack-risk/shield expiry, timed events)
   that a change touches must produce a genuine reason to return within the
   same day — check the actual timer/threshold values against the stated
   target (6-12 sessions/day, 40-90 minutes/day from short visits), not a
   single long-sitting design.
3. **Permanent progression discipline.** Confirm a change respects "remove the
   full-account reset; the world and the account persist" (`ROADMAP.md` Phase
   3) — no code path should silently wipe or reset base level, hero roster,
   research, or territory outside an explicitly designed and disclosed
   mechanic (e.g., a declared ranked-season reset over a persistent account).
4. **Real loss without ruin.** For any change to combat resolution or raid
   outcomes, confirm losses are real (troops, resources, holdings at genuine
   risk) and simultaneously confirm the recoverable-floor rule from point 1
   still holds after the change — these two properties must both be true at
   once, not traded off against each other.
5. **Alliance obligation, not alliance decoration.** For any alliance-facing
   feature, confirm it creates genuine reciprocal obligation (shared
   objectives, help requests, reinforcement, alliance war) rather than a
   cosmetic social layer — `ROADMAP.md` Phase 4 is explicit that this is the
   retention mechanism, not a social feature.
6. **Differentiation check.** For a new genre-standard mechanic (timer, gacha,
   battle pass, alliance war), require a one-line answer to "what does this do
   that a Dark War/Last War clone would not, or what fairness guarantee does
   this enforce in code that they only claim in marketing?" A mechanic that
   cannot answer this is not blocked, but must be flagged as clone-risk per
   `GAME_CONCEPT.md`'s explicit avoid-list.
7. **Metrics are instrumented, not aspirational.** If a change is justified by
   a retention or monetization target from `MONETIZATION.md`'s metrics table
   (D1/D7/D30, payer conversion, LTV/CPI), confirm there is an actual
   measurement path (event, log, harness assertion) rather than a claim that
   the design "should" hit the number.

## Required Verification

```bash
node scripts/simulate.js
```

Extend or point to the specific invariants in `scripts/simulate.js` that assert
the fairness rules above; a fairness rule with no harness assertion is a
promise not yet kept, per `MONETIZATION.md`'s own standard.

## Completion Gate

Do not report a review as clean while a fairness rule lacks a harness
assertion, a return-trigger change lacks a stated session-count/time rationale,
a progression change risks a silent full reset, a loss/recovery change breaks
either "real loss" or "recoverable floor," or a new genre-standard mechanic has
no stated differentiation and no clone-risk flag.

## Trigger

Use **SLG Economy and LiveOps Review** for any change to queues/timers, hero
acquisition, pricing, ranked standings, alliance mechanics, or any design
change justified by a retention/monetization metric.

## Scope Boundary

This skill evaluates economic and retention design against `MONETIZATION.md`
and `ROADMAP.md`. It does not evaluate content schema or client security —
route those to **Content Pipeline Review** and **Web Client Review**. It does
not replace **Perform Game Security Assessment** for server-authority questions
once Phase 4 exists.

## Deliverable

An SLG Economy and LiveOps Review finding set with scope, severity or
priority, affected fairness rule or retention mechanism, evidence, minimal
remediation, and verification status (including which harness invariant
confirms it).
