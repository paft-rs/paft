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

# Test over every valid feature combo in the workspace. This takes way too long.
test-powerset:
  cargo hack -q nextest run --workspace \
    --all-targets \
    --feature-powerset \
    --mutually-exclusive-features rust-decimal,bigdecimal \
    --at-least-one-of rust-decimal,bigdecimal \
    --exclude-no-default-features \
    --status-level none --final-status-level none --success-output never \
    --failure-output final --hide-progress-bar --no-tests pass

# Clippy over the two core feature permutations (workspace or one crate)
lint crate='':
  cargo hack clippy {{ if crate != "" { "-p " + crate } else { "--workspace" } }} \
    --all-targets \
    --ignore-unknown-features \
    --no-default-features --features full,rust-decimal,panicking-money-ops \
    -- -W clippy::all -W clippy::cargo -W clippy::pedantic -W clippy::nursery -A clippy::multiple-crate-versions -D warnings
  cargo hack clippy {{ if crate != "" { "-p " + crate } else { "--workspace" } }} \
    --all-targets \
    --ignore-unknown-features \
    --no-default-features --features full,bigdecimal,panicking-money-ops \
    -- -W clippy::all -W clippy::cargo -W clippy::pedantic -W clippy::nursery -A clippy::multiple-crate-versions -D warnings

# Clippy over every valid feature combo in the workspace. This takes way too long.
lint-powerset:
  cargo hack clippy --workspace \
    --all-targets \
    --feature-powerset \
    --mutually-exclusive-features rust-decimal,bigdecimal \
    --at-least-one-of rust-decimal,bigdecimal \
    --exclude-no-default-features -- -W clippy::all -W clippy::cargo -W clippy::pedantic -W clippy::nursery -A clippy::multiple-crate-versions -D warnings

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