test_default_excludes := 'paft paft-core'
lint_default_excludes := 'paft'
clippy_flags := '-W clippy::all -W clippy::cargo -W clippy::pedantic -W clippy::nursery -A clippy::multiple-crate-versions -D warnings'

# Fast checks - runs in ~1-4 compilations, catches most issues early
test crate='':
  @echo "Running fast test suite..."
  cargo nextest run --locked {{ if crate != "" { "-p " + crate } else { "--workspace" } }} --all-features --all-targets
  cargo test --locked {{ if crate != "" { "-p " + crate } else { "--workspace" } }} --all-features --doc

# Fast lint - mirrors the fast test strategy
lint:
  @echo "Running fast lint..."
  cargo clippy --locked --workspace --all-features --all-targets \
    -- {{ clippy_flags }}

# Checks the fixed decimal type, capabilities, and ingestion under feature combinations.
check-decimal-contract:
  cargo test --locked -p paft-decimal-consumer
  cargo test --locked -p paft --test decimal_contract --no-default-features
  cargo test --locked -p paft --test decimal_contract
  cargo test --locked -p paft --test decimal_contract --all-features

# Test each independently published crate without workspace feature unification.
# Cargo test also runs that package's doctests in each supported configuration.
test-crate-configs crate='':
  cargo hack test --locked {{ if crate != "" { "-p " + crate } else { "--workspace --ignore-private" } }} --no-default-features
  cargo hack test --locked {{ if crate != "" { "-p " + crate } else { "--workspace --ignore-private" } }}
  cargo hack test --locked {{ if crate != "" { "-p " + crate } else { "--workspace --ignore-private" } }} --all-features

# Exhaustive testing strategy
test-full:
  @echo "Running exhaustive test suite..."
  @echo "Step 1/2: Testing paft facade with key configurations..."
  @just test-paft-critical
  @echo "Step 2/2: Testing all workspace crates..."
  @just test-powerset
  cargo test --locked --workspace --all-features --doc

# Exhaustive linting strategy  
lint-full:
  @echo "Running exhaustive lint suite..."
  @echo "Step 1/2: Linting paft facade with key configurations..."
  @just lint-paft-critical
  @echo "Step 2/2: Linting all workspace crates..."
  @just lint-powerset

# === Internal recipes ===

# Test workspace crates with feature powerset, or exactly one selected package
test-powerset crate='':
  #!/usr/bin/env bash
  set -euo pipefail
  PACKAGE_FLAGS=()
  if [[ -n '{{crate}}' ]]; then
    PACKAGE_FLAGS=(-p '{{crate}}')
  else
    PACKAGE_FLAGS=(--workspace)
    for excluded in {{test_default_excludes}}; do
      PACKAGE_FLAGS+=(--exclude "$excluded")
    done
  fi
  cargo hack nextest run --locked "${PACKAGE_FLAGS[@]}" \
    --all-targets \
    --feature-powerset \
    --no-tests pass

# Test paft facade with critical feature combinations
test-paft-critical:
  cargo nextest run --locked -p paft --all-targets --no-default-features --no-tests pass
  cargo nextest run --locked -p paft --all-targets
  cargo nextest run --locked -p paft --all-targets --all-features

# Lint all workspace crates except paft with feature powerset
lint-powerset crate='':
  #!/usr/bin/env bash
  set -euo pipefail
  PACKAGE_FLAGS=()
  if [[ -n '{{crate}}' ]]; then
    PACKAGE_FLAGS=(-p '{{crate}}')
  else
    PACKAGE_FLAGS=(--workspace)
    for excluded in {{lint_default_excludes}}; do
      PACKAGE_FLAGS+=(--exclude "$excluded")
    done
  fi
  cargo hack clippy --locked "${PACKAGE_FLAGS[@]}" \
    --all-targets \
    --feature-powerset \
    -- {{ clippy_flags }}

# Lint paft facade with critical feature combinations
lint-paft-critical:
  cargo clippy --locked -p paft --all-targets --no-default-features -- {{ clippy_flags }}
  cargo clippy --locked -p paft --all-targets --all-features -- {{ clippy_flags }}
  cargo clippy --locked -p paft --all-targets -- {{ clippy_flags }}
  
# Run benchmarks
bench crate='':
  cargo bench {{ if crate != "" { "-p " + crate } else { "--workspace" } }} \
    --all-features

# Format all code
fmt:
  cargo fmt --all

# Generate docs.rs documentation
docrs crate='':
  RUSTDOCFLAGS="--cfg docsrs -Z unstable-options -Dwarnings" \
    cargo +nightly doc {{ if crate != "" { "-p " + crate } else { "--workspace" } }} \
    --all-features --no-deps
