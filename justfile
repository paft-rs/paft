# Requires: cargo install cargo-hack

# Test across the two core feature permutations (workspace or one crate)
test crate='':
  cargo hack -q nextest run {{ if crate != "" { "-p " + crate } else { "--workspace" } }} \
    --all-targets \
    --ignore-unknown-features \
    --no-default-features --features full,rust-decimal,panicking-money-ops \
    --status-level none --final-status-level none --success-output never \
    --failure-output final --hide-progress-bar --no-tests pass
  cargo hack -q nextest run {{ if crate != "" { "-p " + crate } else { "--workspace" } }} \
    --all-targets \
    --ignore-unknown-features \
    --no-default-features --features full,bigdecimal,panicking-money-ops \
    --status-level none --final-status-level none --success-output never \
    --failure-output final --hide-progress-bar --no-tests pass

# Clippy over the two core feature permutations (workspace or one crate)
lint crate='':
  cargo hack clippy {{ if crate != "" { "-p " + crate } else { "--workspace" } }} \
    --all-targets \
    --ignore-unknown-features \
    --no-default-features --features full,rust-decimal,panicking-money-ops \
    -- -D warnings
  cargo hack clippy {{ if crate != "" { "-p " + crate } else { "--workspace" } }} \
    --all-targets \
    --ignore-unknown-features \
    --no-default-features --features full,bigdecimal,panicking-money-ops \
    -- -D warnings

# Benches across the two core feature permutations (workspace or one crate)
bench crate='':
  cargo hack bench {{ if crate != "" { "-p " + crate } else { "--workspace" } }} \
    --ignore-unknown-features \
    --no-default-features --features full,rust-decimal,panicking-money-ops
  cargo hack bench {{ if crate != "" { "-p " + crate } else { "--workspace" } }} \
    --ignore-unknown-features \
    --no-default-features --features full,bigdecimal,panicking-money-ops

fmt:
  cargo fmt --all