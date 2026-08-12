---
name: audit-anti-cheat
description: Apply the Audit Anti Cheat workflow for relevant audit-anti-cheat work; use it before proposing or validating a change.
---

# Audit Anti Cheat

## Trigger

Use **Audit Anti Cheat** when the task requires an adversarial assessment and the affected artifact, acceptance criteria, and authorization boundary are known.

## Inputs

- Target files, affected runtime path, and explicit acceptance criteria
- Existing architecture, tests, deployment/rollback boundary, and applicable profile
- Authorization and stop conditions when the task has security, production, financial, or external effects

## Specialist Procedure

Define threat model, trusted authority, telemetry minimization, tamper detection limits, server-side validation, appeals, false-positive safeguards, version rollout, and monitoring. Do not collect intrusive data without necessity and consent analysis.

1. Model the cheat classes that matter for this game: client memory edits, injected input, network manipulation, and automation, each with its economic motive.
2. Verify that every competitive outcome is decided by trusted authority, and that detection is a supplement rather than the enforcement mechanism.
3. Assess telemetry against necessity and consent: what is collected, how long it is kept, who can read it, and what a false positive costs a player.
4. Deliver detection changes with rollout, appeal, and false-positive handling, plus the measurement that shows the change worked.

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

A Audit Anti Cheat finding set with scope, severity or priority, affected contract, evidence, minimal remediation, and verification status.
