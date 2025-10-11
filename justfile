crates := 'paft paft-core paft-utils paft-domain paft-aggregates paft-market paft-fundamentals paft-money'
test_default_excludes := 'paft paft-core'
lint_default_excludes := 'paft'
clippy_flags := '-W clippy::all -W clippy::cargo -W clippy::pedantic -W clippy::nursery -A clippy::multiple-crate-versions -D warnings'

# Fast checks - runs in ~1-4 compilations, catches most issues early
test crate='':
  @echo "Running fast test suite..."
  cargo nextest run {{ if crate != "" { "-p " + crate } else { "--workspace" } }} --no-default-features --features "rust-decimal,dataframe"
  cargo nextest run {{ if crate != "" { "-p " + crate } else { "--workspace" } }} --no-default-features --features "bigdecimal"

# Fast lint - mirrors the fast test strategy
lint:
  @echo "Running fast lint..."
  cargo clippy --workspace --no-default-features --features "full,rust-decimal,dataframe,panicking-money-ops,ident-validate" \
    -- {{ clippy_flags }}

# Exhaustive testing strategy
test-full:
  @echo "Running exhaustive test suite..."
  @echo "Step 1/2: Testing all workspace crates (84 permutations)..."
  @just test-powerset
  @echo "Step 2/2: Testing paft facade with key configurations..."
  @just test-paft-critical

# Exhaustive linting strategy  
lint-full:
  @echo "Running exhaustive lint suite..."
  @echo "Step 1/2: Linting all workspace crates (84 permutations)..."
  @just lint-powerset
  @echo "Step 2/2: Linting paft facade with key configurations..."
  @just lint-paft-critical

# === Internal recipes ===

# Test all workspace crates except paft with feature powerset (84 permutations)
test-powerset crate='':
  #!/usr/bin/env bash
  EXCLUDES=''
  if [[ -n '{{crate}}' ]]; then
    for c in {{crates}}; do
      if [[ "$c" != '{{crate}}' ]]; then EXCLUDES="$EXCLUDES $c"; fi
    done
  else
    EXCLUDES='{{test_default_excludes}}'
  fi
  EXCLUDE_FLAGS=()
  for e in $EXCLUDES; do EXCLUDE_FLAGS+=("--exclude" "$e"); done
  cargo hack nextest run --workspace "${EXCLUDE_FLAGS[@]}" \
    --all-targets \
    --feature-powerset \
    --mutually-exclusive-features rust-decimal,bigdecimal \
    --at-least-one-of rust-decimal,bigdecimal \
    --exclude-no-default-features \
    --no-tests pass

# Test paft facade with critical feature combinations
test-paft-critical:
  cargo nextest run -p paft --no-default-features \
    --features "full,rust-decimal,dataframe,panicking-money-ops,money-formatting,ident-validate"
  cargo nextest run -p paft --no-default-features \
    --features "full,bigdecimal,dataframe,panicking-money-ops,money-formatting,ident-validate"
  cargo nextest run -p paft --no-default-features --features "domain,rust-decimal"
  cargo nextest run -p paft --no-default-features --features "full,rust-decimal"

# Lint all workspace crates except paft with feature powerset (84 permutations)
lint-powerset crate='':
  #!/usr/bin/env bash
  EXCLUDES=''
  if [[ -n '{{crate}}' ]]; then
    for c in {{crates}}; do
      if [[ "$c" != '{{crate}}' ]]; then EXCLUDES="$EXCLUDES $c"; fi
    done
  else
    EXCLUDES='{{lint_default_excludes}}'
  fi
  EXCLUDE_FLAGS=()
  for e in $EXCLUDES; do EXCLUDE_FLAGS+=("--exclude" "$e"); done
  cargo hack clippy --workspace "${EXCLUDE_FLAGS[@]}" \
    --all-targets \
    --feature-powerset \
    --mutually-exclusive-features rust-decimal,bigdecimal \
    --at-least-one-of rust-decimal,bigdecimal \
    --exclude-no-default-features \
    -- {{ clippy_flags }}

# Lint paft facade with critical feature combinations
lint-paft-critical:
  cargo clippy -p paft --no-default-features \
    --features "full,rust-decimal,dataframe,panicking-money-ops,money-formatting,ident-validate" \
    -- {{ clippy_flags }}
  cargo clippy -p paft --no-default-features \
    --features "full,bigdecimal,dataframe,panicking-money-ops,money-formatting,ident-validate" \
    -- {{ clippy_flags }}
  cargo clippy -p paft --no-default-features --features "domain,rust-decimal" \
    -- {{ clippy_flags }}
  cargo clippy -p paft --no-default-features --features "full,rust-decimal" \
    -- {{ clippy_flags }}

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
