# The Extensible Enum Pattern in paft

## Overview

paft uses a consistent "Other(String)" extensible enum pattern across all enum types (`Currency`, `Exchange`, `AssetKind`, `MarketState`, `Period`, `RecommendationGrade`, etc.). This pattern provides a pragmatic solution to the fragmentation problem in financial data, allowing the library to gracefully handle provider-specific or unknown variants without failing.

Important serde and normalization rules:

- Emission is a single canonical token per known variant (UPPERCASE ASCII, no spaces), and the parser accepts a superset of aliases (case-insensitive).
- For `Other(s)`, serialization emits exactly the `Display`/`code()` string (no escape prefix). Deserialization routes through `try_from_str`, which first checks aliases and then uppercases unknown tokens into `Other(UPPERCASE)`.
- This maintains round-trip identity for canonical variants and consistent normalization for unknowns: canonical "BTC" ↔ `BTC`; unknown "bitcoin" ↔ `Other("BITCOIN")`.

## The Philosophy

### The Problem: Provider Fragmentation

Financial data providers use inconsistent naming conventions and often introduce new values that don't fit existing categories. For example:

- **Currencies**: Some providers use "BTC" for Bitcoin, others use "BITCOIN" or "XBT"
- **Exchanges**: "NASDAQ" vs "NASDAQ-GS" vs "NASDAQ-GM" 
- **Asset Types**: "EQUITY" vs "STOCK" vs "COMMON_STOCK"
- **Market States**: "PRE" vs "PREMARKET" vs "PRE-MARKET"

### The Solution: Extensible Enums

Instead of failing when encountering unknown values, paft's extensible enums:

1. **Try to parse** the input as a known canonical variant first
2. **Fall back gracefully** to `Other(String)` for unknown values
3. **Normalize** unknown values to uppercase for consistency
4. **Preserve** the normalized string for debugging and provider compatibility

## Implementation Pattern

Every extensible enum in paft follows this consistent pattern:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, AsRefStr, EnumString, Default)]
#[strum(ascii_case_insensitive)]
pub enum ExampleEnum {
    /// Known variant with multiple serialization aliases
    #[strum(serialize = "VARIANT1", serialize = "ALT_NAME")]
    Variant1,

    /// Another known variant
    Variant2,

    /// Unknown or provider-specific value
    Other(String),
}

impl ExampleEnum {
    /// Parses a string, returning `Other(UPPERCASE)` for unknown values.
    pub fn try_from_str(input: &str) -> Result<Self, String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err("example enum cannot be empty".to_string());
        }

        if let Ok(parsed) = Self::from_str(trimmed) {
            return Ok(parsed);
        }

        Ok(Self::Other(trimmed.to_ascii_uppercase()))
    }
}

impl std::convert::TryFrom<String> for ExampleEnum {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_str(&value)
    }
}

impl From<ExampleEnum> for String {
    fn from(enum_value: ExampleEnum) -> Self {
        format!("{}", enum_value)
    }
}

impl serde::Serialize for ExampleEnum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for ExampleEnum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::try_from_str(&raw).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for ExampleEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            ExampleEnum::Variant1 => "VARIANT1",
            ExampleEnum::Variant2 => "VARIANT2",
            ExampleEnum::Other(code) => code,
        };
        f.write_str(value)
    }
}

impl std::convert::TryFrom<String> for ExampleEnum {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_str(&value)
    }
}
```

## The Trade-Offs

### Benefits

✅ **Graceful Degradation**: Never fails on unknown values  
✅ **Provider Compatibility**: Works with any data source  
✅ **Future-Proof**: Handles new values without code changes  
✅ **Debugging Friendly**: Preserves normalized provider strings  
✅ **Type Safety**: Still provides compile-time checks for known variants  

### Costs

⚠️ **Runtime Checks**: What could be compile-time errors become runtime string comparisons  
⚠️ **Performance**: String matching is slower than enum matching  
⚠️ **Error-Prone**: Consumers must handle `Other` variants explicitly  
⚠️ **Maintenance**: Requires careful handling in match statements  

## Usage Guidelines for Library Consumers

### 1. Always Handle the Other Variant

**❌ Don't do this:**
```rust
use paft_money::{Currency, IsoCurrency};

fn process_currency(currency: Currency) -> String {
    match currency {
        Currency::Iso(IsoCurrency::USD) => "US Dollar".to_string(),
        Currency::Iso(IsoCurrency::EUR) => "Euro".to_string(),
        // Missing Other variant - will cause compiler error
    }
}
```

**✅ Do this:**
```rust
use paft_money::{Currency, IsoCurrency};

fn process_currency(currency: Currency) -> String {
    match currency {
        Currency::Iso(IsoCurrency::USD) => "US Dollar".to_string(),
        Currency::Iso(IsoCurrency::EUR) => "Euro".to_string(),
        Currency::Other(code) => format!("Unknown currency: {}", code),
    }
}
```

### 2. Map Provider-Specific Strings to Canonical Variants

**❌ Don't do this:**
```rust
fn handle_provider_currency(provider_code: &str) -> Currency {
    // Always falls back to Other, even for known variants
    Currency::try_from_str(provider_code)
        .unwrap_or_else(|_| Currency::Other(provider_code.trim().to_uppercase()))
}
```

**✅ Do this:**
```rust
fn handle_provider_currency(provider_code: &str) -> Currency {
    // Map provider-specific codes to canonical variants
    match provider_code {
        "BITCOIN" | "XBT" => Currency::BTC,
        "DOLLAR" => Currency::Iso(IsoCurrency::USD),
        other => Currency::try_from_str(other)
            .unwrap_or_else(|_| Currency::Other(other.trim().to_uppercase())),
    }
}
```

### 3. Use Helper Methods When Available

Many paft enums provide helper methods to check for canonical variants:

```rust
fn analyze_asset(asset: AssetKind) {
    if asset.is_canonical() {
        // Handle known asset types with full type safety
        match asset {
            AssetKind::Equity => println!("Stock analysis"),
            AssetKind::Crypto => println!("Crypto analysis"),
            _ => unreachable!(), // Safe because is_canonical() returned true
        }
    } else {
        // Handle unknown asset types
        if let AssetKind::Other(unknown_type) = asset {
            println!("Unknown asset type: {}", unknown_type);
        }
    }
}
```

### 4. Prefer Canonical Variants in Your Code

**❌ Don't do this:**
```rust
// Creates Other variant unnecessarily and breaks round-trip
let currency = Currency::Other("USD".to_string());
```

**✅ Do this:**
```rust
// Uses canonical variant
let currency = Currency::Iso(IsoCurrency::USD);
```

### 5. Handle Other Variants Gracefully in APIs

```rust
pub fn get_currency_info(currency: Currency) -> Result<CurrencyInfo, Error> {
    match currency {
        Currency::Iso(IsoCurrency::USD) => Ok(CurrencyInfo::usd()),
        Currency::Iso(IsoCurrency::EUR) => Ok(CurrencyInfo::euro()),
        Currency::Other(code) => {
            // Log unknown currency for monitoring
            log::warn!("Unknown currency encountered: {}", code);
            
            // Return a generic currency info or error
            Err(Error::UnsupportedCurrency(code))
        }
    }
}
```

## Best Practices for Library Authors

### 1. Encourage Ecosystem Convergence

When building libraries on top of paft, encourage mapping provider-specific strings to canonical variants:

```rust
// In your provider adapter
impl From<GenericProviderCurrency> for Currency {
    fn from(gp_currency: GenericProviderCurrency) -> Self {
        match gp_currency.code.as_ref() {
            "BTC" | "BITCOIN" => Currency::BTC,
            "USD" => Currency::Iso(IsoCurrency::USD),
            "EUR" => Currency::Iso(IsoCurrency::EUR),
            // Map as many as possible to canonical variants
            other => Currency::Other(other.trim().to_uppercase()),
        }
    }
}
```

### 2. Provide Conversion Utilities

```rust
pub mod currency_utils {
    use super::Currency;
    use paft_money::IsoCurrency;
    
    /// Attempts to normalize a currency code to a canonical variant
    pub fn normalize_currency_code(code: &str) -> Currency {
        let trimmed = code.trim();
        if trimmed.is_empty() {
            return Currency::Other("UNKNOWN".to_string());
        }

        match trimmed.to_uppercase().as_ref() {
            "BITCOIN" | "XBT" => Currency::BTC,
            "DOLLAR" | "US_DOLLAR" | "USD" => Currency::Iso(IsoCurrency::USD),
            other => Currency::Other(other.to_string()),
        }
    }
    
    /// Returns true if the currency code is commonly used
    pub fn is_common_currency(currency: &Currency) -> bool {
        match currency {
            Currency::Iso(IsoCurrency::USD | IsoCurrency::EUR | IsoCurrency::GBP | IsoCurrency::JPY) => true,
            Currency::Other(code) => matches!(code.as_ref(), "BTC" | "ETH"),
            _ => false,
        }
    }
}
```

### 3. Document Your Mappings

```rust
/// Currency mapping for Alpha Vantage provider
/// 
/// Known mappings:
/// - "BITCOIN" -> Other("BTC")
/// - "DOLLAR" -> Currency::Iso(IsoCurrency::USD)
/// - "EURO" -> Currency::Iso(IsoCurrency::EUR)
/// 
/// Unmapped values are preserved as Other(String)
impl From<AlphaVantageCurrency> for Currency {
    // ... implementation
}
```

## Migration Strategy

### For Existing Code

1. **Add Other handling** to all match statements
2. **Map common provider values** to canonical variants
3. **Add logging** for unknown values to identify patterns
4. **Gradually expand** canonical variants based on usage

### For New Code

1. **Always handle Other** from the start
2. **Use helper methods** like `is_canonical()` when available
3. **Prefer canonical variants** in your own code
4. **Document your mappings** for future maintainers

## Examples

### Complete Example: Currency Processing

```rust
use paft::domain::{Currency, IsoCurrency};

fn process_currencies(currencies: Vec<Currency>) {
    for currency in currencies {
        match currency {
            // Handle known currencies with full type safety
            Currency::Iso(IsoCurrency::USD) => {
                println!("Processing US Dollar");
                // USD-specific logic
            }
            Currency::Iso(IsoCurrency::EUR) => {
                println!("Processing Euro");
                // EUR-specific logic
            }
            
            // Handle unknown currencies gracefully
            Currency::Other(code) => {
                println!("Processing unknown currency: {}", code);
                
                // Try to handle common unknown currencies
                match code.as_ref() {
                    "BTC" => {
                        println!("Bitcoin detected - using crypto logic");
                        // Crypto-specific logic
                    }
                    "ETH" => {
                        println!("Ethereum detected - using crypto logic");
                        // Crypto-specific logic
                    }
                    _ => {
                        println!("Truly unknown currency: {}", code);
                        // Generic handling or error
                    }
                }
            }
        }
    }
}

// Helper function to normalize provider data
fn normalize_provider_currency(provider_code: &str) -> Currency {
    // Prefer canonical variants, never produce Other for values we model canonically
    match provider_code.to_uppercase().as_ref() {
        "DOLLAR" | "US_DOLLAR" | "USD" => Currency::Iso(IsoCurrency::USD),
        "EURO" | "EUR" => Currency::Iso(IsoCurrency::EUR),
        "BITCOIN" | "XBT" | "BTC" => Currency::BTC,
        "ETHEREUM" | "ETH" => Currency::ETH,
        _ => Currency::try_from_str(provider_code)
            .unwrap_or_else(|_| Currency::Other(provider_code.trim().to_uppercase())),
    }
}
```

## Conclusion

The extensible enum pattern in paft is a pragmatic solution to the real-world problem of provider fragmentation. While it introduces some complexity, it enables the library to work with any financial data provider without breaking.

**Key takeaways:**

1. **Always handle `Other` variants** in your match statements
2. **Map provider-specific strings** to canonical variants when possible
3. **Use helper methods** like `is_canonical()` for cleaner code
4. **Document your mappings** for future maintainers
5. **Log unknown values** to identify patterns for future canonical variants

This pattern enables ecosystem convergence over time while maintaining compatibility with the fragmented reality of financial data providers.
