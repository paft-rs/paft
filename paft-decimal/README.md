# paft-decimal

Lightweight decimal facade used across the `paft` workspace. The crate wraps
`rust_decimal` by default and can switch to `bigdecimal` via the optional
`bigdecimal` feature, providing common helpers for parsing, rounding, and
canonical string rendering without depending on higher-level money types.

