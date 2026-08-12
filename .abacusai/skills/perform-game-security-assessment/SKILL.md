---
name: perform-game-security-assessment
description: Apply the Perform Game Security Assessment workflow for relevant perform-game-security-assessment work; use it before proposing or validating a change.
---

# Perform Game Security Assessment

## Trigger

Use **Perform Game Security Assessment** when the task requires an explicitly authorized operational assessment and the affected artifact, acceptance criteria, and authorization boundary are known.

## Inputs

- Target files, affected runtime path, and explicit acceptance criteria
- Existing architecture, tests, deployment/rollback boundary, and applicable profile
- Authorization and stop conditions when the task has security, production, financial, or external effects

## Specialist Procedure

Model hostile clients and economy abuse. Verify server or trusted authority for rewards, inventory, matchmaking, purchases, saved state, replay/rate controls, telemetry privacy, anti-cheat false-positive handling, and safe remediation paths.

1. Map every state transition that carries value — rewards, currency, inventory, ranking, purchases — and where it is decided.
2. Attempt authorized, non-destructive abuse on a test shard: replay, reordering, rate, and client-asserted results.
3. Assess anti-cheat and telemetry for privacy, false-positive cost, and appeal handling as part of the security posture.
4. Deliver server-side validation changes and a retest scenario for each confirmed path.

## Required Evidence

- A source-level explanation of the affected contract and invariant
- At least one focused automated or reproducible verification appropriate to the capability
- Explicit treatment of boundary, failure, and unauthorized/invalid paths where applicable
- A measurement rather than intuition for every material performance claim

## Scope Boundary

This skill evaluates game trust, economy, and anti-cheat boundaries; it does not justify invasive telemetry, bypassing client protections, or testing players without authorization.

## Guardrails

Never expose secrets, claim unrun checks passed, broaden authorized scope, hide a breaking change, or perform destructive/production actions without explicit approval.

## Deliverable

A game threat model covering authority, economy, client trust, abuse paths, telemetry/privacy implications, remediation owner, and retest evidence.
