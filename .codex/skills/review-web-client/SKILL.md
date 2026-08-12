---
name: review-web-client
description: Apply the Web Client Review workflow for browser JS/DOM/Canvas client work (game.js, rules.js, cinematic.js, worldmap.js, seat.js); use it before proposing or validating a change.
---

# Web Client Review

This project's shipping client (as of Stage 1) is dependency-free static
HTML/CSS/JS: `index.html`, `game.js`, `rules.js`, `cinematic.js`,
`worldmap.js`, `seat.js`, `styles.css`. There is no build step, no
`package.json`, and no server. Treat the browser as the actual runtime and the
player's machine as hostile.

## Procedure

1. Identify which files are **rules** (must be pure: no DOM, no `localStorage`,
   no `Math.random`, no wall-clock reads) versus **presentation** (renders and
   dispatches, must not decide outcomes). `rules.js` must stay in the first
   category; a change that lets `game.js`/`cinematic.js`/`worldmap.js`/`seat.js`
   read or mutate rules state directly, or that lets rendering code branch on
   an outcome the rules engine hasn't produced, is a contract violation.
2. Grep every touched file for `innerHTML`, `outerHTML`, `document.write`,
   `eval`, `new Function`, and template-literal HTML construction. Any player-
   or content-authored string (champion names, region flavor text, content
   packs under `content/`) reaching the DOM through one of these without
   escaping is an XSS path.
3. Check every `localStorage`/`sessionStorage` read and write site. Until
   Phase 4 (server-authoritative state) lands, the RNG seed and season state
   live in client storage and are directly editable — this is a **documented,
   accepted exploit** for save-scumming, state editing, and outcome
   precomputation (see `PROJECT_CONTEXT.md`). Do not treat it as a bug to fix
   silently; treat it as a boundary to keep visible and not worsen (e.g., don't
   add new authoritative values to client storage without flagging it).
4. Confirm no hidden network calls, analytics, or third-party script tags are
   introduced — this is a fully offline static client today, and anything that
   reaches out to a network is a scope change requiring explicit sign-off.
5. Confirm keyboard operability and contrast are preserved for any new UI
   (per the Accessibility rule); a mouse-only interaction added to the core
   loop is a regression, not a style choice.
6. For anything touching `rules.js`, confirm determinism survives: same seed,
   same command sequence, same output. Non-deterministic iteration order
   (unordered object/Map iteration used as if ordered) or a newly introduced
   wall-clock/timer dependency breaks replay and the future C# port (Phase 5).

## Required Verification

```bash
node scripts/simulate.js
node scripts/check-ui.js
grep -rn "innerHTML\|outerHTML\|document.write\|eval(\|new Function" game.js rules.js cinematic.js worldmap.js seat.js
```

Re-run `scripts/simulate.js` with the same seed before and after a `rules.js`
change and diff the output; a change that alters results for an unrelated seed
is a regression.

## Completion Gate

Do not report a review as clean while an XSS-reachable content path, a rules/
presentation boundary violation, a new hidden network call, or a determinism
break remains unresolved. The `localStorage` client-trust gap is a known,
accepted-until-Phase-4 exception — flag it when a change touches it, but it is
not itself a blocking finding unless the change makes it worse.

## Trigger

Use **Web Client Review** for any change to `game.js`, `rules.js`,
`cinematic.js`, `worldmap.js`, `seat.js`, `index.html`, or `styles.css`.

## Scope Boundary

This skill covers the static browser client only. Server-authoritative state,
network reconciliation, and account security become relevant starting Phase 4
and are out of scope until that architecture exists.

## Deliverable

A Web Client Review finding set with scope, severity or priority, affected
contract (rules purity, XSS surface, storage trust boundary, determinism),
evidence, minimal remediation, and verification status.
