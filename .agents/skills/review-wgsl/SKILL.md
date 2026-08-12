---
name: review-wgsl
description: Apply the WGSL and WGPU Review workflow for relevant review-wgsl work; use it before proposing or validating a change.
---

# WGSL and WGPU Review

Review host and shader code together. A shader review without the matching bind
group layouts and Rust structures is incomplete.

## Contract Checks

1. Match every `@group`/`@binding`, visibility flag, resource type, access mode,
   minimum binding size, texture format, sampler type, and vertex location.
2. Validate WGSL alignment, padding, array stride, matrix representation, and
   dynamic-offset alignment against host-side buffer definitions. `Pod` is not
   proof that WGSL layout is correct.
3. Check workgroup size against device limits and guard every storage-buffer or
   texture index against invocation count and resource length.
4. Make coordinate transitions explicit: object, world, view, clip, texture,
   depth range, handedness, normal matrix, and camera-relative origin.
5. Protect divisions and normalization from zero or near-zero values; define
   NaN/Infinity behavior and large-world precision strategy.

## Performance Checks

Identify per-frame uploads, pipeline churn, repeated bind-group construction,
oversized uniform traffic, divergent branches, repeated texture samples,
transcendentals in hot fragments, uncoalesced storage access, and avoidable
CPU/GPU synchronization. Propose a measurement mechanism, not a guessed gain.

## Verification

Compile shaders through the real pipeline, run validation layers where
available, test at boundary dimensions, capture at least one representative
frame, and verify visual output against an explicit expected property. For
Q-Verse, test deterministic visual parameters separately from canonical
simulation state.

## Inputs

Identify the target files or runtime path, acceptance criteria, applicable profile, current repository state, and authorization boundary before acting.

## Task Execution

Inspect first. Validate host/WGSL offsets, padding, binding types, workgroup bounds, device limits, coordinate and depth conventions, zero and NaN behavior, resource lifetime, upload frequency, and an actual render or compute result. Make the smallest safe change or finding, then verify the affected contract before closing.

## Guardrails

Do not exceed authorization, expose secrets, claim unrun checks passed, conceal a breaking change, or perform destructive, financial, production, or external actions without explicit approval.

## Trigger

Use **WGSL and WGPU Review** when the task requires a bounded engineering review and the affected artifact, acceptance criteria, and authorization boundary are known.

## Scope Boundary

This skill reports findings and remediation evidence; it does not claim the complete system is defect-free or approve unrelated components.

## Deliverable

A WGSL and WGPU Review finding set with scope, severity or priority, affected contract, evidence, minimal remediation, and verification status.
