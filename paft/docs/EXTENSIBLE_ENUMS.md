# The Extensible Enum Pattern in paft

## Overview

paft uses a consistent "Other(String)" extensible enum pattern across all enum types (`Currency`, `Exchange`, `AssetKind`, `MarketState`, `Period`, `RecommendationGrade`, etc.). This pattern provides a pragmatic solution to the fragmentation problem in financial data, allowing the library to gracefully handle provider-specific or unknown variants without failing.

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
4. **Preserve** the original string for debugging and provider compatibility

## Implementation Pattern

Every extensible enum in paft follows this consistent pattern:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, AsRefStr, EnumString, Default)]
#[strum(ascii_case_insensitive)]
#[serde(from = "String", into = "String")]
pub enum ExampleEnum {
    /// Known variant with multiple serialization aliases
    #[strum(serialize = "VARIANT1", serialize = "ALT_NAME")]
    Variant1,
    
    /// Another known variant
    Variant2,
    
    /// Unknown or provider-specific value
    Other(String),
}

impl From<String> for ExampleEnum {
    fn from(s: String) -> Self {
        // Try to parse as a known variant first
        match Self::from_str(&s) {
            Ok(variant) => variant,
            Err(_) => Self::Other(s.to_uppercase()),
        }
    }
}

impl From<ExampleEnum> for String {
    fn from(enum_value: ExampleEnum) -> Self {
        match enum_value {
            ExampleEnum::Other(s) => s,
            _ => enum_value.to_string(),
        }
    }
}
```

## The Trade-Offs

### Benefits

✅ **Graceful Degradation**: Never fails on unknown values  
✅ **Provider Compatibility**: Works with any data source  
✅ **Future-Proof**: Handles new values without code changes  
✅ **Debugging Friendly**: Preserves original provider strings  
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
fn process_currency(currency: Currency) -> String {
    match currency {
        Currency::USD => "US Dollar".to_string(),
        Currency::EUR => "Euro".to_string(),
        // Missing Other variant - will cause compiler error
    }
}
```

**✅ Do this:**
```rust
fn process_currency(currency: Currency) -> String {
    match currency {
        Currency::USD => "US Dollar".to_string(),
        Currency::EUR => "Euro".to_string(),
        Currency::Other(code) => format!("Unknown currency: {}", code),
    }
}
```

### 2. Map Provider-Specific Strings to Canonical Variants

**❌ Don't do this:**
```rust
fn handle_provider_currency(provider_code: &str) -> Currency {
    // Always falls back to Other, even for known variants
    Currency::from(provider_code.to_string())
}
```

**✅ Do this:**
```rust
fn handle_provider_currency(provider_code: &str) -> Currency {
    // Map provider-specific codes to canonical variants
    match provider_code {
        "BITCOIN" | "XBT" => Currency::Other("BTC".to_string()),
        "DOLLAR" => Currency::USD,
        _ => Currency::from(provider_code.to_string()),
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
// Creates Other variant unnecessarily
let currency = Currency::Other("USD".to_string());
```

**✅ Do this:**
```rust
// Uses canonical variant
let currency = Currency::USD;
```

### 5. Handle Other Variants Gracefully in APIs

```rust
pub fn get_currency_info(currency: Currency) -> Result<CurrencyInfo, Error> {
    match currency {
        Currency::USD => Ok(CurrencyInfo::usd()),
        Currency::EUR => Ok(CurrencyInfo::euro()),
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
        match yf_currency.code.as_str() {
            "BTC" | "BITCOIN" => Currency::Other("BTC".to_string()),
            "USD" => Currency::USD,
            "EUR" => Currency::EUR,
            // Map as many as possible to canonical variants
            _ => Currency::Other(yf_currency.code.to_uppercase()),
        }
    }
}
```

### 2. Provide Conversion Utilities

```rust
pub mod currency_utils {
    use super::Currency;
    
    /// Attempts to normalize a currency code to a canonical variant
    pub fn normalize_currency_code(code: &str) -> Currency {
        match code.to_uppercase().as_str() {
            "BITCOIN" | "XBT" => Currency::Other("BTC".to_string()),
            "DOLLAR" | "US_DOLLAR" => Currency::USD,
            _ => Currency::from(code.to_string()),
        }
    }
    
    /// Returns true if the currency code is commonly used
    pub fn is_common_currency(currency: &Currency) -> bool {
        match currency {
            Currency::USD | Currency::EUR | Currency::GBP | Currency::JPY => true,
            Currency::Other(code) => matches!(code.as_str(), "BTC" | "ETH"),
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
/// - "DOLLAR" -> USD
/// - "EURO" -> EUR
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
use paft::domain::Currency;

fn process_currencies(currencies: Vec<Currency>) {
    for currency in currencies {
        match currency {
            // Handle known currencies with full type safety
            Currency::USD => {
                println!("Processing US Dollar");
                // USD-specific logic
            }
            Currency::EUR => {
                println!("Processing Euro");
                // EUR-specific logic
            }
            
            // Handle unknown currencies gracefully
            Currency::Other(code) => {
                println!("Processing unknown currency: {}", code);
                
                // Try to handle common unknown currencies
                match code.as_str() {
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
    match provider_code.to_uppercase().as_str() {
        "DOLLAR" | "US_DOLLAR" => Currency::USD,
        "EURO" => Currency::EUR,
        "BITCOIN" | "XBT" => Currency::Other("BTC".to_string()),
        _ => Currency::from(provider_code.to_string()),
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
