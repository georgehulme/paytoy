# Safety and Robustness:
 - I don't use any unsafe code, but I do use std::process::exit in some error cases where I believe it's appropriate to terminate immediately.
   - Suitable for CLI context but not in server model unless issue occurs at startup.
 - I am skipping erroneous transactions, which probably should be added to a dead-letter queue for manual resolution in a real system, but I don't have access to one of those here, and felt that adding one would not be in scope, so I just log the error and move on.
 - Disputes can result in negative account balances, which may or may not be desirable in a real system.
 - How am I handling errors?
   - I am exiting  with a non-zero error code for cases I believe to be fatal and I am using the `tracing` crate to log errors to stderr in non-fatal cases.
     - This could easily be piped to a logfile or redirected to a log ingestion service in a real system.
 - I made use of the `rust_decimal::Decimal` type to ensure that floating point errors would not impact data integrity when dealing with large balances and transaction amounts.
   - Would need to enable `db-postgres` feature, or similar, if persisting to DB. Need to look into effects on decimal precision.
 - I am allowing deposits and further disputes when an account is locked/frozen. Behaviour seems to differ between account types. So, this needs reviewed.

# Efficiency:
 - I am streaming the transaction records, using the `csv` crate's deserialization capabilities to avoid loading the entire file into memory at once. This crate also uses buffering under the hood to avoid I/O bottlenecks, which is nice.
 - The ledger is sored in memory, which is not ideal for a real system. I would prefer to offload this to a database or at least use some sort of on-disk storage to avoid memory issues with large datasets, but this does not make much sense in a CLI context unless we expect to be working with those sorts of dataset sizes.
   - Move to client -> API <- Server design
 - The output is collected before being written to stdout, which could result in high memory usage for large datasets. In a real system, I would want to stream the output as well, but I wanted to ensure that the output is sorted by client ID for repeatable CLI testing.

# Maintainability:
 - The code is organised into a library crate for the core ledger logic and a binary application that handles the CLI interface and file I/O. This separation of responsibilities allows for easier testing and reuse of the core logic in other contexts.
 - I made sure to make use of `cargo fmt --all` and `cargo clippy --all-targets -- -W clippy::pedantic` to ensure that the code formatting here matches Rust community standards.
