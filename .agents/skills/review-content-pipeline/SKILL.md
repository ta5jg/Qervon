---
name: review-content-pipeline
description: Apply the Content Pipeline Review workflow for content packs, CONTENT_PIPELINE.md, and scripts/check-content.js; use it before proposing or validating a change.
---

# Content Pipeline Review

The project authors game content (champions, regions, relics, LiveOps event
text, alliance-facing strings) as data under `content/`, validated by
`scripts/check-content.js` against the contract in `CONTENT_PIPELINE.md`. This
pipeline is the actual mechanism for the "large, non-repeating LiveOps
calendar" ambition in `ROADMAP.md`, so its correctness gates content volume the
same way the rules engine gates gameplay correctness.

## Procedure

1. Confirm every content file conforms to the schema/closed vocabulary defined
   in `CONTENT_PIPELINE.md` — reject free-form fields that `check-content.js`
   cannot validate, since an unvalidated field is a silent way to break the
   client or introduce unescaped strings later (see **Web Client Review** for
   the XSS angle).
2. Confirm content data carries no executable payload and no field that a
   downstream consumer would `eval`, inject via `innerHTML`, or otherwise treat
   as code rather than data.
3. Confirm every player-facing string is structured so it can move into a
   locale file without a schema change — `ROADMAP.md` Phase 4 requires every
   player-facing string to be localizable, and retrofitting that later is more
   expensive than designing for it now.
4. Confirm new content is versioned or additive rather than silently
   overwriting or renaming existing IDs referenced elsewhere (save data, rules
   engine lookups, other content files) — a renamed champion or region ID can
   silently break a save or a cross-reference.
5. Confirm tone consistency against `GAME_CONCEPT.md` ("dark, elegant and
   strategic... not horror for horror's sake") and against the "avoid clone
   perception" constraint — flag content that reads as generic Dark War/Last
   War reskinning rather than the project's own voice.

## Required Verification

```bash
node scripts/check-content.js
```

Run against every changed or added file under `content/`. A clean run is
necessary evidence that the schema validates; it is not evidence that the
content is balanced (see **SLG Economy and LiveOps Review** for balance) or
that strings are final copy.

## Completion Gate

Do not report a review as clean while an unvalidated field, an executable
payload in content data, a silently renamed/removed ID with live references, or
a non-localizable player-facing string remains unresolved.

## Trigger

Use **Content Pipeline Review** for any change under `content/`,
`CONTENT_PIPELINE.md`, or `scripts/check-content.js`.

## Scope Boundary

This skill validates content structure, safety, and localization-readiness. It
does not judge gameplay balance, monetization fairness, or retention design —
route those to **SLG Economy and LiveOps Review**.

## Deliverable

A Content Pipeline Review finding set with scope, severity or priority,
affected contract (schema, ID stability, localization-readiness), evidence,
minimal remediation, and verification status.
