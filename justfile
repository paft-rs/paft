# Requires: cargo install cargo-hack

# Test every valid feature combo (workspace or one crate)
test crate='':
  cargo hack test {{ if crate != "" { "-p " + crate } else { "--workspace" } }} \
    --all-targets \
    --feature-powerset \
    --mutually-exclusive-features rust-decimal,bigdecimal \
    --at-least-one-of rust-decimal,bigdecimal \
    --exclude-no-default-features

# Clippy over every valid feature combo (workspace or one crate)
lint crate='':
  cargo hack clippy {{ if crate != "" { "-p " + crate } else { "--workspace" } }} \
    --all-targets \
    --feature-powerset \
    --mutually-exclusive-features rust-decimal,bigdecimal \
    --at-least-one-of rust-decimal,bigdecimal \
    --exclude-no-default-features -- -D warnings

# Benches across every valid feature combo (workspace or one crate)
bench crate='':
  cargo hack bench {{ if crate != "" { "-p " + crate } else { "--workspace" } }} \
    --feature-powerset \
    --mutually-exclusive-features rust-decimal,bigdecimal \
    --at-least-one-of rust-decimal,bigdecimal \
    --exclude-no-default-features

fmt:
  cargo fmt --all
