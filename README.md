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

## Overview
TODO

## Transactions
There are five types of transactions recorded. Deposits and withdraws represent money flowing in and out of the system, while disputes, resolves and chargebacks are related to dispute claims.
### Deposit
A credit to a client's asset account from an external source. Processing a deposit increases both the client's available funds and total funds by the specified amount.

### Withdrawal
A debit from a client's asset account to an external destination. Processing a withdrawal decreases both the client's available funds and total funds by the specified amount. A withdrawal should fail if the client does not have sufficient available funds.

### Dispute
A dispute is a claim that a previously processed transaction (specifically a deposit) was erroneous or fraudulent and should be reversed. When a dispute is filed, **the disputed funds are moved from available to held, keeping the total unchanged.** A dispute references the original transaction by ID and can be followed by either a resolve (releasing the held funds back to available) or a chargeback (removing the held funds and freezing the account).

### Resolve
A resolution to an ongoing dispute, indicating that the disputed transaction was valid after all. Processing a resolve moves the disputed funds from held back to available, leaving the total unchanged. A resolve is ignored if the referenced transaction does not exist or is not currently under dispute.

### Chargeback
The final state of a dispute, representing a reversal of the original transaction. Processing a chargeback removes the disputed funds from both held and total, and immediately freezes the client's account. A chargeback is ignored if the referenced transaction does not exist or is not currently under dispute.