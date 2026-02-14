# Payments

This is a CLI that processes transactions in a CSV format and outputs it in a CSV format.
```
cargo run -- transactions.csv > accounts.csv
```

The input file is the first and only argument to the binary. Output is written to stdout.

## Input
```
type, client, tx, amount
deposit, 1, 1, 5.0
deposit, 4, 2, 5.0
withdraw, 1, 1, 1.0
```

## Output
```
client,available,held,total,locked
2,2,0,2,false
```

## Transactions
There are five types of transactions in this system.
### Deposit
TODO
### Withdrawal
TODO
### Dispute
TODO
### Resolve
TODO
### Chargeback
TODO