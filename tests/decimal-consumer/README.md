# Facade-only decimal consumer

This private fixture uses `paft` with default features disabled as its only
PAFT dependency. Serde dependencies belong to the consumer's own payloads.
It verifies the public parser, error classification, all four exact arithmetic
helpers, and canonical serde attributes without direct `paft-decimal` access.

Run `cargo test --locked -p paft-decimal-consumer`. The fixture also runs through
`just check-decimal-contract` in CI and through the workspace test suite.
