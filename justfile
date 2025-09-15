lint crate='':
  cargo clippy --workspace --all-features {{ if crate != "" { "-p " + crate } else { "" } }} -- \
    -W clippy::all -W clippy::cargo -W clippy::pedantic -W clippy::nursery -A clippy::multiple-crate-versions -D warnings

test crate='':
  cargo test --workspace --all-features {{ if crate != "" { "-p " + crate } else { "" } }}

bench crate='':
  cargo bench {{ if crate != "" { "-p " + crate } else { "" } }}
