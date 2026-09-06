# Provider mapping fixtures

`deribit_option_quantities.json` is a reduced excerpt of the option example in
[Deribit's book summary documentation](https://docs.deribit.com/api-reference/market-data/public-get_book_summary_by_currency),
retrieved 2026-09-06. The numeric tokens are unchanged. The source describes
volume over 24 hours and open interest in underlying base-currency units for
options. Tests supply explicit hypothetical contract sizes separately; those
sizes are not captured provider metadata or claims about the listed contract.
