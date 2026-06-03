# paft Documentation

This directory contains comprehensive documentation for the paft (Provider Agnostic Financial Types) library.

## Core Documentation

### [EXTENSIBLE_ENUMS.md](EXTENSIBLE_ENUMS.md)
**The Extensible Enum Pattern in paft**

Comprehensive guide to understanding and using paft's typed `Other` extensible enum pattern. Covers:

- Philosophy and rationale behind the pattern
- Implementation details and trade-offs
- Usage guidelines for library consumers
- Best practices for handling unknown variants
- Migration strategies from string-based code

### [BEST_PRACTICES.md](BEST_PRACTICES.md)
**Best Practices for paft Extensible Enums**

Detailed best practices guide with practical implementation strategies:

- Consumer best practices
- Library author guidelines
- Provider adapter patterns
- Migration strategies
- Performance considerations
- Testing strategies

## Quick Reference

### Key Concepts

1. **Extensible Enums**: Provider-facing paft enums use typed `Other` wrappers for unknown values
2. **Graceful Degradation**: Never fails on unknown provider values
3. **Ecosystem Convergence**: Map provider-specific strings to canonical variants
4. **Type Safety**: Compile-time checks for known variants, runtime handling for unknown
5. **Contextual Amounts**: High-cardinality market records carry denomination
   once and use `PriceAmount` / `QuantityAmount` for contained values

### Essential Pattern

```rust
use paft_money::{Currency, IsoCurrency};

let label: String = match currency {
    Currency::Iso(IsoCurrency::USD) => "US Dollar".to_string(),
    Currency::Iso(IsoCurrency::EUR) => "Euro".to_string(),
    Currency::Other(code) => match code.as_ref() {
        "BTC" => "Bitcoin".to_string(),
        _ => format!("Unknown: {}", code),
    },
    other => other.code().to_string(),
};
```

### Helper Methods

Many paft enums provide helper methods:

```rust
match asset {
    AssetKind::Equity => "Stock",
    AssetKind::Crypto => "Crypto",
    AssetKind::Other(_) => "Other",
    _ => "Other",
}
```

## Examples

Start with [examples/v09_ergonomics.rs](../examples/v09_ergonomics.rs) for a
small tour of the current market/fundamentals shapes, then see
[examples/extensible_enums.rs](../examples/extensible_enums.rs) for comprehensive enum examples showing:

- Proper handling of Other variants
- Provider data normalization
- Utility functions for common mappings
- Testing strategies

## Philosophy

The extensible enum pattern in paft addresses the fundamental challenge of financial data fragmentation. Every provider uses different naming conventions, introduces new values, and evolves their APIs over time.

Instead of failing when encountering unknown values, paft's extensible enums:

1. **Try to parse** as known canonical variants first
2. **Fall back gracefully** to typed `Other` values for unknown values  
3. **Normalize** unknown values to uppercase for consistency
4. **Preserve** original strings for debugging and compatibility

This approach enables:

- **Provider Compatibility**: Works with any data source
- **Future-Proofing**: Handles new values without code changes
- **Ecosystem Convergence**: Encourages mapping to canonical variants
- **Type Safety**: Compile-time checks for known variants

## Contributing

When contributing to paft:

1. **Follow the pattern**: Use enum-specific typed `Other` wrappers for all new extensible enums
2. **Document mappings**: Clearly document provider-specific mappings
3. **Add tests**: Test both canonical and Other variants
4. **Consider performance**: Avoid unnecessary string allocations
5. **Encourage convergence**: Map provider values to canonical variants when possible

## Related Resources

- [Main README](https://github.com/paft-rs/paft/blob/main/README.md) - Quick start and overview
- [Crates.io](https://crates.io/crates/paft) - Package information
- [Documentation](https://docs.rs/paft) - API reference
- [GitHub](https://github.com/paft-rs/paft) - Source code and issues
