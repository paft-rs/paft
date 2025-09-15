# Best Practices for paft Extensible Enums

This document provides specific best practices for working with paft's extensible enum pattern, focusing on practical implementation strategies for library authors and consumers.

## Table of Contents

1. [Consumer Best Practices](#consumer-best-practices)
2. [Library Author Best Practices](#library-author-best-practices)
3. [Provider Adapter Patterns](#provider-adapter-patterns)
4. [Migration Strategies](#migration-strategies)
5. [Performance Considerations](#performance-considerations)
6. [Testing Strategies](#testing-strategies)

## Consumer Best Practices

### 1. Always Handle Other Variants

**❌ Compilation Error:**
```rust
fn process_currency(currency: Currency) -> &'static str {
    match currency {
        Currency::USD => "US Dollar",
        Currency::EUR => "Euro",
        // Missing Other variant - compiler error!
    }
}
```

**✅ Correct Approach:**
```rust
fn process_currency(currency: Currency) -> String {
    match currency {
        Currency::USD => "US Dollar".to_string(),
        Currency::EUR => "Euro".to_string(),
        Currency::Other(code) => format!("Unknown currency: {}", code),
    }
}
```

### 2. Use Helper Methods When Available

Many paft enums provide helper methods to check for canonical variants:

```rust
fn analyze_asset(asset: AssetKind) {
    if asset.is_canonical() {
        // Safe to use exhaustive matching
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

### 3. Create Mapping Functions for Common Cases

```rust
/// Maps provider-specific currency codes to canonical variants or common Other values
fn normalize_currency(provider_code: &str) -> Currency {
    match provider_code.to_uppercase().as_str() {
        // Map to canonical variants when possible
        "DOLLAR" | "US_DOLLAR" => Currency::USD,
        "EURO" => Currency::EUR,
        "POUND" | "GBP" => Currency::GBP,
        
        // Normalize crypto currencies to common Other values
        "BITCOIN" | "XBT" => Currency::Other("BTC".to_string()),
        "ETHEREUM" => Currency::Other("ETH".to_string()),
        
        // Preserve other values as-is
        _ => Currency::from(provider_code.to_string()),
    }
}
```

### 4. Use Default Values Appropriately

```rust
fn get_currency_info(currency: Currency) -> CurrencyInfo {
    match currency {
        Currency::USD => CurrencyInfo::usd(),
        Currency::EUR => CurrencyInfo::euro(),
        Currency::Other(code) => {
            // Log unknown currency for monitoring
            log::warn!("Unknown currency encountered: {}", code);
            
            // Return a generic currency info
            CurrencyInfo::generic(code)
        }
    }
}
```

## Library Author Best Practices

### 1. Encourage Ecosystem Convergence

When building libraries on top of paft, map provider-specific strings to canonical variants whenever possible:

```rust
// In your provider adapter
impl From<GenericProviderCurrency> for Currency {
    fn from(gp_currency: GenericProviderCurrency) -> Self {
        match yf_currency.code.as_str() {
            // Map to canonical variants
            "USD" => Currency::USD,
            "EUR" => Currency::EUR,
            "GBP" => Currency::GBP,
            
            // Normalize crypto currencies
            "BTC" | "BITCOIN" => Currency::Other("BTC".to_string()),
            "ETH" | "ETHEREUM" => Currency::Other("ETH".to_string()),
            
            // Preserve other values
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
            "DOLLAR" | "US_DOLLAR" => Currency::USD,
            "EURO" => Currency::EUR,
            "BITCOIN" | "XBT" => Currency::Other("BTC".to_string()),
            _ => Currency::from(code.to_string()),
        }
    }
    
    /// Returns true if the currency is commonly used
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
/// - "US_DOLLAR" -> USD (canonical)
/// - "EURO" -> EUR (canonical)
/// - "BITCOIN" -> Other("BTC") (normalized)
/// 
/// Unmapped values are preserved as Other(String) in uppercase
impl From<AlphaVantageCurrency> for Currency {
    fn from(av_currency: AlphaVantageCurrency) -> Self {
        match av_currency.code.to_uppercase().as_str() {
            "US_DOLLAR" => Currency::USD,
            "EURO" => Currency::EUR,
            "BITCOIN" => Currency::Other("BTC".to_string()),
            _ => Currency::Other(av_currency.code.to_uppercase()),
        }
    }
}
```

### 4. Add Validation and Logging

```rust
pub struct CurrencyProcessor {
    unknown_currencies: std::collections::HashSet<String>,
}

impl CurrencyProcessor {
    pub fn process_currency(&mut self, currency: Currency) -> Result<CurrencyInfo, Error> {
        match currency {
            Currency::USD => Ok(CurrencyInfo::usd()),
            Currency::EUR => Ok(CurrencyInfo::euro()),
            Currency::Other(code) => {
                // Track unknown currencies for analysis
                self.unknown_currencies.insert(code.clone());
                
                // Log for monitoring
                log::warn!("Unknown currency encountered: {}", code);
                
                // Return error or generic info based on your needs
                Err(Error::UnsupportedCurrency(code))
            }
        }
    }
    
    /// Get statistics about unknown currencies encountered
    pub fn unknown_currency_stats(&self) -> &std::collections::HashSet<String> {
        &self.unknown_currencies
    }
}
```

## Provider Adapter Patterns

### 1. Comprehensive Mapping Strategy

```rust
pub struct ProviderAdapter {
    currency_mappings: std::collections::HashMap<String, Currency>,
    exchange_mappings: std::collections::HashMap<String, Exchange>,
}

impl ProviderAdapter {
    pub fn new() -> Self {
        let mut currency_mappings = std::collections::HashMap::new();
        let mut exchange_mappings = std::collections::HashMap::new();
        
        // Populate mappings
        currency_mappings.insert("DOLLAR".to_string(), Currency::USD);
        currency_mappings.insert("EURO".to_string(), Currency::EUR);
        currency_mappings.insert("BITCOIN".to_string(), Currency::Other("BTC".to_string()));
        
        exchange_mappings.insert("NASDAQ-GS".to_string(), Exchange::NASDAQ);
        exchange_mappings.insert("NYSE-ARCA".to_string(), Exchange::NYSE);
        
        Self {
            currency_mappings,
            exchange_mappings,
        }
    }
    
    pub fn normalize_currency(&self, provider_code: &str) -> Currency {
        self.currency_mappings
            .get(&provider_code.to_uppercase())
            .cloned()
            .unwrap_or_else(|| Currency::Other(provider_code.to_uppercase()))
    }
    
    pub fn normalize_exchange(&self, provider_code: &str) -> Exchange {
        self.exchange_mappings
            .get(&provider_code.to_uppercase())
            .cloned()
            .unwrap_or_else(|| Exchange::Other(provider_code.to_uppercase()))
    }
}
```

### 2. Lazy Loading Pattern

```rust
pub struct LazyProviderAdapter {
    mappings: std::sync::OnceLock<std::collections::HashMap<String, Currency>>,
}

impl LazyProviderAdapter {
    fn get_mappings(&self) -> &std::collections::HashMap<String, Currency> {
        self.mappings.get_or_init(|| {
            let mut mappings = std::collections::HashMap::new();
            mappings.insert("DOLLAR".to_string(), Currency::USD);
            mappings.insert("EURO".to_string(), Currency::EUR);
            mappings.insert("BITCOIN".to_string(), Currency::Other("BTC".to_string()));
            mappings
        })
    }
    
    pub fn normalize_currency(&self, provider_code: &str) -> Currency {
        self.get_mappings()
            .get(&provider_code.to_uppercase())
            .cloned()
            .unwrap_or_else(|| Currency::Other(provider_code.to_uppercase()))
    }
}
```

## Migration Strategies

### 1. Gradual Migration from String-based Code

**Before (string-based):**
```rust
fn process_currency(currency_code: &str) -> String {
    match currency_code {
        "USD" => "US Dollar".to_string(),
        "EUR" => "Euro".to_string(),
        _ => format!("Unknown: {}", currency_code),
    }
}
```

**After (paft-based):**
```rust
fn process_currency(currency: Currency) -> String {
    match currency {
        Currency::USD => "US Dollar".to_string(),
        Currency::EUR => "Euro".to_string(),
        Currency::Other(code) => format!("Unknown: {}", code),
    }
}

// Migration helper
fn migrate_currency_code(code: &str) -> Currency {
    match code {
        "USD" => Currency::USD,
        "EUR" => Currency::EUR,
        _ => Currency::Other(code.to_uppercase()),
    }
}
```

### 2. Backward Compatibility Layer

```rust
pub struct CurrencyCompat {
    currency: Currency,
}

impl CurrencyCompat {
    pub fn from_string(code: &str) -> Self {
        Self {
            currency: Currency::from(code.to_string()),
        }
    }
    
    pub fn to_string(&self) -> String {
        match &self.currency {
            Currency::Other(s) => s.clone(),
            _ => self.currency.to_string(),
        }
    }
    
    pub fn is_known(&self) -> bool {
        !matches!(self.currency, Currency::Other(_))
    }
}
```

## Performance Considerations

### 1. Avoid Repeated String Allocations

**❌ Inefficient:**
```rust
fn process_currencies(currencies: Vec<Currency>) {
    for currency in currencies {
        match currency {
            Currency::Other(code) => {
                // Creates new string allocation
                let upper_code = code.to_uppercase();
                println!("Processing: {}", upper_code);
            }
            _ => {}
        }
    }
}
```

**✅ Efficient:**
```rust
fn process_currencies(currencies: Vec<Currency>) {
    for currency in currencies {
        match currency {
            Currency::Other(code) => {
                // Use the string directly
                println!("Processing: {}", code);
            }
            _ => {}
        }
    }
}
```

### 2. Use String Interning for Common Other Values

```rust
use std::sync::OnceLock;

pub struct CurrencyCache {
    btc: OnceLock<Currency>,
    eth: OnceLock<Currency>,
}

impl CurrencyCache {
    pub fn btc() -> Currency {
        static CACHE: OnceLock<Currency> = OnceLock::new();
        *CACHE.get_or_init(|| Currency::Other("BTC".to_string()))
    }
    
    pub fn eth() -> Currency {
        static CACHE: OnceLock<Currency> = OnceLock::new();
        *CACHE.get_or_init(|| Currency::Other("ETH".to_string()))
    }
}
```

## Testing Strategies

### 1. Test Both Canonical and Other Variants

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_currency_processing() {
        // Test canonical variants
        assert_eq!(process_currency(Currency::USD), "US Dollar");
        assert_eq!(process_currency(Currency::EUR), "Euro");
        
        // Test Other variants
        assert_eq!(
            process_currency(Currency::Other("BTC".to_string())),
            "Unknown currency: BTC"
        );
        assert_eq!(
            process_currency(Currency::Other("UNKNOWN".to_string())),
            "Unknown currency: UNKNOWN"
        );
    }
    
    #[test]
    fn test_currency_normalization() {
        assert_eq!(normalize_currency("DOLLAR"), Currency::USD);
        assert_eq!(normalize_currency("BITCOIN"), Currency::Other("BTC".to_string()));
        assert_eq!(normalize_currency("UNKNOWN"), Currency::Other("UNKNOWN".to_string()));
    }
}
```

### 2. Property-based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_currency_roundtrip(currency in any::<Currency>()) {
        let string_repr = currency.to_string();
        let parsed = Currency::from(string_repr);
        assert_eq!(currency, parsed);
    }
    
    #[test]
    fn test_currency_normalization_preserves_other(
        code in "[A-Z]{1,10}"
    ) {
        let currency = Currency::Other(code.clone());
        assert_eq!(currency.to_string(), code);
    }
}
```

### 3. Integration Testing with Real Provider Data

```rust
#[test]
fn test_generic_provider_currency_mapping() {
    let test_cases = vec![
        ("USD", Currency::USD),
        ("EUR", Currency::EUR),
        ("BTC", Currency::Other("BTC".to_string())),
        ("UNKNOWN", Currency::Other("UNKNOWN".to_string())),
    ];
    
    for (provider_code, expected) in test_cases {
        let result = normalize_generic_provider_currency(provider_code);
        assert_eq!(result, expected, "Failed for provider code: {}", provider_code);
    }
}
```

## Conclusion

The extensible enum pattern in paft provides a robust foundation for handling the fragmented nature of financial data providers. By following these best practices:

1. **Always handle Other variants** in your match statements
2. **Map provider-specific strings** to canonical variants when possible
3. **Use helper methods** like `is_canonical()` for cleaner code
4. **Document your mappings** for future maintainers
5. **Test both canonical and Other variants** thoroughly
6. **Consider performance implications** of string operations

You can build reliable, maintainable financial applications that work seamlessly across different data providers while encouraging ecosystem convergence over time.
