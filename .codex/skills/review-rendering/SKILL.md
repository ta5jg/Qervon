---
name: review-rendering
description: Apply the Rendering Review workflow for relevant review-rendering work; use it before proposing or validating a change.
---

# Rendering Review

Review the rendering pipeline as a data-flow system from canonical simulation
state through transient scene data, GPU resources, passes, presentation, and
observable frames.

## Procedure

1. Document coordinate spaces, origin policy, handedness, depth convention,
   color space, alpha convention, and ownership of simulation versus render
   data. Rendering must not mutate canonical game or simulation truth.
2. Match host and GPU contracts: binding groups, buffer alignment/padding,
   array stride, vertex layout, texture/sampler compatibility, dynamic offsets,
   resource visibility, format support, and device limits.
3. Inspect resource lifetimes and frame scheduling: allocation/reuse policy,
   pipeline and bind-group churn, staging uploads, synchronization points,
   resize/device-loss handling, and destruction on scene changes.
4. Identify hot-stage cost: draw/dispatch count, overdraw, visibility and LOD,
   material switches, buffer traffic, shader divergence, texture sampling,
   transient allocations, and CPU/GPU wait. Tie each optimization proposal to a
   measurable frame-time or memory mechanism.
5. Test rendering at boundary dimensions, zero/large values, missing assets,
   low-end limits, camera extremes, and recovery after resize or device loss.
   Capture a representative frame or a precise visual assertion; a clean log is
   not proof of visual correctness.

## Completion Gate

Report correctness risks, performance hypotheses, measurements actually taken,
resource-lifetime findings, and the visual evidence produced. Keep large-world
precision and deterministic visual parameters separate from canonical simulation.

## Required Evidence

Record the affected contract, commands and tool versions when material, observed result, unresolved risk, and a regression test or reproducible inspection appropriate to the task.

## Trigger

Use **Review Rendering** when the task requires a bounded engineering review and the affected artifact, acceptance criteria, and authorization boundary are known.

## Scope Boundary

This skill reports findings and remediation evidence; it does not claim the complete system is defect-free or approve unrelated components.

## Deliverable

A Review Rendering finding set with scope, severity or priority, affected contract, evidence, minimal remediation, and verification status.
