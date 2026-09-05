# Built-in non-ISO denominations

Audited for v0.10.0 on 2026-09-06. `CurrencyMetadata::minor_units` is the
settlement exponent: one integer minor unit is `10^-minor_units` major units.
It controls integer conversion, settlement rounding, and exact `Money`
validation. Venue quantity increments and preferred display precision are
separate concepts.

The defaults below describe the named native asset or original token contract.
They do not identify bridged tokens, wrapped assets, or unrelated assets sharing
a ticker. Adapters must check the asset identity before using a default.
PAFT's currency registry is keyed only by canonical code; it has no network,
contract, or historical denomination discriminator.

## Sourced defaults

Every retained non-ISO entry is covered by
`every_builtin_non_iso_denomination_matches_its_native_unit` in
`src/currency_utils.rs`. The test checks table coverage, the metadata exponent,
`Currency::decimal_places()`, and the numeric result of
`Money::from_minor_units(1, currency)` against independent literals.

| Code | Native denomination / scope | Exponent | One minor unit | Primary definition |
| --- | --- | ---: | --- | --- |
| BTC | Bitcoin satoshi | 8 | `0.00000001` | [Bitcoin Core's `COIN`](https://github.com/bitcoin/bitcoin/blob/master/src/consensus/amount.h) is 100,000,000 satoshis. |
| ETH | Ethereum wei | 18 | `0.000000000000000001` | [Ethereum's ether denominations](https://ethereum.org/developers/docs/intro-to-ether) define wei as `10^-18` ETH. |
| XMR | Monero atomic unit (piconero) | 12 | `0.000000000001` | [Moneropedia: atomic units](https://www.getmonero.org/resources/moneropedia/atomic-units.html). |
| ADA | Cardano lovelace | 6 | `0.000001` | [Cardano's native currency definition](https://docs.cardano.org/about-cardano/new-to-cardano/what-is-a-cryptocurrency) defines 1,000,000 lovelaces per ADA. |
| SOL | Solana lamport | 9 | `0.000000001` | [Solana's lamport definition](https://solana.com/docs/references/terminology#lamport). |
| XRP | XRP Ledger drop | 6 | `0.000001` | [XRP Ledger currency formats](https://xrpl.org/docs/references/protocol/data-types/currency-formats) define native XRP amounts in drops. |
| DOT | Current Polkadot planck denomination | 10 | `0.0000000001` | [Polkadot's DOT definition](https://wiki.polkadot.com/learn/learn-dot/) defines `10^10` plancks per DOT. This is the denomination after the 2020 redenomination; historical pre-redenomination values need explicit handling. |
| DOGE | Native Dogecoin unit | 8 | `0.00000001` | [Dogecoin Core's `COIN`](https://github.com/dogecoin/dogecoin/blob/master/src/amount.h) is 100,000,000 base units. |
| LINK | Native LINK Juel | 18 | `0.000000000000000001` | [Chainlink's token definition](https://docs.chain.link/resources/link-token-contracts) defines `10^18` Juels per LINK. |
| LTC | Native Litecoin unit | 8 | `0.00000001` | [Litecoin Core's `COIN`](https://github.com/litecoin-project/litecoin/blob/master/src/amount.h) is 100,000,000 base units. |
| MATIC | Original MATIC token on Ethereum | 18 | `0.000000000000000001` | [Polygon's contract directory](https://docs.polygon.technology/pos/reference/contracts/genesis-contracts) identifies `0x7D1AfA7B718fb893dB30A3aBc0Cfc608AaCfeBB0`; its [verified constructor arguments](https://etherscan.io/token/0x7d1afa7b718fb893db30a3abc0cfc608aacfebb0#code) set `decimals` to 18. This code retains MATIC's identity; it does not alias the replacement POL token. |
| UNI | Original Uniswap governance token | 18 | `0.000000000000000001` | [Uniswap's `Uni.sol`](https://github.com/Uniswap/governance/blob/master/contracts/Uni.sol) declares `decimals = 18`. |

## Codes requiring explicit metadata

The audit removed these entries because their ticker alone cannot choose a
denomination. Their currency codes still parse, including the `Currency::USDC`
and `Currency::USDT` variants. Metadata lookup returns `None` until registration,
and constructors that require a currency scale return
`MoneyError::MetadataNotFound`. `Price` and `MonetaryAmount` do not require a
settlement scale.

| Code | Why no default is supplied | Primary definitions |
| --- | --- | --- |
| USDC | Stellar uses 7 decimal places; other CCTP-supported chains use 6. Even CCTP message units differ from Stellar's native units. | [Circle's Stellar precision rules](https://developers.circle.com/cctp/references/stellar). |
| USDT | The Ethereum denomination uses 6 decimal places; Tether USD on Liquid uses 8. | [Tether's Ethereum integration example](https://github.com/tetherto/wdk-protocol-bridge-usdt0-evm), [Tether's supported protocols](https://tether.to/en/supported-protocols/), and [Liquid's issued-asset denominations](https://docs.liquid.net/docs/liquid-assets). |
| BNB | Native Beacon Chain BNB uses 8 decimal places; native BNB Smart Chain BNB uses 18. Beacon Chain recovery and historical data retain the old units. | [BNB's `TokenHub` contract](https://github.com/bnb-chain/bsc-genesis-contract/blob/master/contracts/TokenHub.sol) explicitly converts between the two native denominations. |
| AVAX | P-Chain/X-Chain and C-Chain atomic operations use 9 decimal places (nAVAX); C-Chain EVM amounts use 18 (wei). | [Avalanche SDK unit conversions](https://build.avax.network/docs/tooling/avalanche-sdk/client/utils). |

Use `set_currency_metadata` after resolving the relevant network and asset.
When different denominations must coexist, register distinct application-defined
currency codes, such as `USDC_ETHEREUM` and `USDC_STELLAR`, instead of changing
one process-wide entry between records. Those names are application choices,
not additional PAFT built-ins. See the [registration example](README.md#currency-metadata).

## Migrating captured money values

LINK, UNI, and MATIC defaults changed from 8 to 18. New
`Money::from_minor_units(1, currency)` values therefore represent `10^-18`
tokens, where the old default produced `10^-8`. New exact settlement ingestion
accepts 18 fractional places and settlement rounding uses that scale.

Existing `Money` values keep and serialize their captured `minor_units`; that
scale participates in equality, hashing, integer conversion, and arithmetic
compatibility. An old LINK/UNI/MATIC payload with `minor_units: 8` now conflicts
with the built-in scale and fails deserialization. Values with different
captured scales cannot be added or subtracted.

Migrate with knowledge of the original input:

- If the stored major-unit amount is correct, reconstruct it at scale 18 without
  changing its numeric value. An old amount of `0.00000001` remains that amount
  and now corresponds to 10,000,000,000 native minor units.
- If an original native integer count was decoded using the wrong scale,
  reconstruct from that original count using the corrected denomination.
  Merely changing the serialized exponent preserves the already-wrong amount.
- If legacy scale-8 settlement semantics are intentional, preserve them under
  a separately registered application code, or explicitly override metadata in
  a process dedicated to those legacy semantics. The corrected default does
  not rewrite existing values.

For USDC, USDT, BNB, and AVAX, register the intended scale before constructing
new `Money`. Serialized values still contain enough information to restore
their captured scale when metadata is absent; this does not verify their native
denomination. When metadata is registered, mismatching serialized scales fail.
Review old amounts and integer conversions against their source network,
especially AVAX values previously built with the unsupported scale 8.
