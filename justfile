# Fast checks - runs in ~1-4 compilations, catches most issues early
test:
  @echo "Running fast test suite..."
  cargo nextest run --workspace --no-default-features --features "full,rust-decimal,dataframe,panicking-money-ops,ident-validate"
  cargo nextest run -p paft-money -p paft-domain -p paft-market -p paft-fundamentals --no-default-features --features "bigdecimal"

# Fast lint - mirrors the fast test strategy
lint:
  @echo "Running fast lint..."
  cargo clippy --workspace --no-default-features --features "full,rust-decimal,dataframe,panicking-money-ops,ident-validate" \
    -- -W clippy::all -W clippy::cargo -W clippy::pedantic -W clippy::nursery -A clippy::multiple-crate-versions -D warnings

# Exhaustive testing strategy
test-full:
  @echo "Running exhaustive test suite..."
  @echo "Step 1/2: Testing all workspace crates (84 permutations)..."
  @just test-powerset-no-paft
  @echo "Step 2/2: Testing paft facade with key configurations..."
  @just test-paft-critical

# Exhaustive linting strategy  
lint-full:
  @echo "Running exhaustive lint suite..."
  @echo "Step 1/2: Linting all workspace crates (84 permutations)..."
  @just lint-powerset-no-paft
  @echo "Step 2/2: Linting paft facade with key configurations..."
  @just lint-paft-critical

# === Internal recipes ===

# Test all workspace crates except paft with feature powerset (84 permutations)
test-powerset-no-paft:
  cargo hack nextest run --workspace --exclude paft \
    --all-targets \
    --feature-powerset \
    --mutually-exclusive-features rust-decimal,bigdecimal \
    --at-least-one-of rust-decimal,bigdecimal \
    --exclude-no-default-features \
    --no-tests pass

# Test paft facade with critical feature combinations
test-paft-critical:
  # Full features + rust-decimal (most common config)
  cargo nextest run -p paft --no-default-features \
    --features "full,rust-decimal,dataframe,panicking-money-ops,money-formatting,ident-validate"
  # Full features + bigdecimal (alternate backend)
  cargo nextest run -p paft --no-default-features \
    --features "full,bigdecimal,dataframe,panicking-money-ops,money-formatting,ident-validate"
  # Minimal config (ensures facade works without extras)
  cargo nextest run -p paft --no-default-features --features "domain,rust-decimal"
  # No dataframe (common constraint)
  cargo nextest run -p paft --no-default-features --features "full,rust-decimal"

# Lint all workspace crates except paft with feature powerset (84 permutations)
lint-powerset-no-paft:
  cargo hack clippy --workspace --exclude paft \
    --all-targets \
    --feature-powerset \
    --mutually-exclusive-features rust-decimal,bigdecimal \
    --at-least-one-of rust-decimal,bigdecimal \
    --exclude-no-default-features \
    -- -W clippy::all -W clippy::cargo -W clippy::pedantic -W clippy::nursery \
       -A clippy::multiple-crate-versions -D warnings

# Lint paft facade with critical feature combinations
lint-paft-critical:
  # Full features + rust-decimal
  cargo clippy -p paft --no-default-features \
    --features "full,rust-decimal,dataframe,panicking-money-ops,money-formatting,ident-validate" \
    -- -W clippy::all -W clippy::cargo -W clippy::pedantic -W clippy::nursery \
       -A clippy::multiple-crate-versions -D warnings
  # Full features + bigdecimal  
  cargo clippy -p paft --no-default-features \
    --features "full,bigdecimal,dataframe,panicking-money-ops,money-formatting,ident-validate" \
    -- -W clippy::all -W clippy::cargo -W clippy::pedantic -W clippy::nursery \
       -A clippy::multiple-crate-versions -D warnings
  # Minimal config
  cargo clippy -p paft --no-default-features --features "domain,rust-decimal" \
    -- -W clippy::all -W clippy::cargo -W clippy::pedantic -W clippy::nursery \
       -A clippy::multiple-crate-versions -D warnings
  # No dataframe
  cargo clippy -p paft --no-default-features --features "full,rust-decimal" \
    -- -W clippy::all -W clippy::cargo -W clippy::pedantic -W clippy::nursery \
       -A clippy::multiple-crate-versions -D warnings

# Run benchmarks (useful for performance regression testing)
bench crate='':
  cargo bench {{ if crate != "" { "-p " + crate } else { "--workspace" } }} \
    --no-default-features --features "full,rust-decimal,panicking-money-ops"
  cargo bench {{ if crate != "" { "-p " + crate } else { "--workspace" } }} \
    --no-default-features --features "full,bigdecimal,panicking-money-ops"

# Format all code
fmt:
  cargo fmt --all

# Generate docs.rs documentation
docrs crate='':
  RUSTDOCFLAGS="--cfg docsrs -Z unstable-options -Dwarnings" \
    cargo +nightly doc {{ if crate != "" { "-p " + crate } else { "--workspace" } }} \
    --no-default-features --features "full,rust-decimal" --no-deps
  RUSTDOCFLAGS="--cfg docsrs -Z unstable-options -Dwarnings" \
    cargo +nightly doc {{ if crate != "" { "-p " + crate } else { "--workspace" } }} \
    --no-default-features --features "full,bigdecimal" --no-deps
