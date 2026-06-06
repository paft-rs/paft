# Repository Guidelines

We are currently working on v0.9.0, last tagged version is 0.8.0 on main, so breaking changes are allowed and recommended. Keep the changelog updated under v0.9.0 as you work.

## PAFT

provider-agnostic financial types: a stable foundation for arbitrary financial
applications, not an adapter layer for provider's quirks.

`paft` models reusable financial concepts, not provider wire shapes: required
fields are the concept's valid minimum, incomplete providers fail to map, and
provider-specific extras belong in generic metadata.

Requests, configuration, and semantic metadata shapes are strict when silently dropping fields could change meaning. A tagged data payload is not strict solely because it has a `kind` discriminator. Provider/data payload structs are forward-compatible unless validation requires rejecting unknown fields; serde-flattened provider metadata shares the owning JSON namespace, so colliding JSON field names are unsupported rather than universally detected. DataFrame export namespaces provider metadata under `provider.*`.

## Internal “validated serde” checklist
- Public fields are fine for plain data bags.
- Private fields plus builder for validated requests.
- Manual/shadow deserialization for any type whose constructor enforces invariants.
- No derived Deserialize for invariant-bearing structs unless all fields are already validated newtypes and no cross-field invariant exists.

## Clippy suppression policy
- Prefer fixing lints. For intentional suppressions, use `#[expect(..., reason = "...")]` so stale suppressions fail linting; use `allow(..., reason = "...")` only when `expect` cannot model the feature/cfg shape.
