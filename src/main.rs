use clap::Parser;
use paytoy_engine::{ClientId, DisputeAction, Ledger, Movement, TransactionCommand, TransactionId};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::error;
use tracing_subscriber::{EnvFilter, prelude::*};

// Define DTO types for CSV parsing and output formatting
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize)]
struct TransactionRecord {
    #[serde(rename = "type")]
    tx_type: TransactionType,
    #[serde(rename = "tx")]
    tx_id: TransactionId,
    #[serde(rename = "client")]
    client_id: ClientId,
    amount: Option<Decimal>,
}

#[derive(Debug, Serialize)]
struct AccountRecord {
    #[serde(rename = "client")]
    id: ClientId,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

#[derive(clap::Parser)]
struct Cli {
    input_path: std::path::PathBuf,
}

impl TryFrom<TransactionRecord> for TransactionCommand {
    type Error = String;

    fn try_from(record: TransactionRecord) -> Result<Self, Self::Error> {
        match record.tx_type {
            TransactionType::Deposit => {
                let amount = record.amount.ok_or("Amount is required for deposit")?;
                Ok(TransactionCommand::ProcessMovement {
                    tx_id: record.tx_id,
                    client_id: record.client_id,
                    movement: Movement::Deposit(amount),
                })
            }
            TransactionType::Withdrawal => {
                let amount = record.amount.ok_or("Amount is required for withdrawal")?;
                Ok(TransactionCommand::ProcessMovement {
                    tx_id: record.tx_id,
                    client_id: record.client_id,
                    movement: Movement::Withdrawal(amount),
                })
            }
            TransactionType::Dispute => Ok(TransactionCommand::ProcessDispute {
                target_tx_id: record.tx_id,
                client_id: record.client_id,
                action: DisputeAction::Open,
            }),
            TransactionType::Resolve => Ok(TransactionCommand::ProcessDispute {
                target_tx_id: record.tx_id,
                client_id: record.client_id,
                action: DisputeAction::Resolve,
            }),
            TransactionType::Chargeback => Ok(TransactionCommand::ProcessDispute {
                target_tx_id: record.tx_id,
                client_id: record.client_id,
                action: DisputeAction::Chargeback,
            }),
        }
    }
}

fn init_logging() {
    let format = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(false);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(format)
        .with(filter)
        .init();
}

fn process_transactions(
    ledger: &mut Ledger,
    transactions: impl IntoIterator<Item = TransactionRecord>,
) -> Vec<AccountRecord> {
    for transaction in transactions {
        match TransactionCommand::try_from(transaction) {
            Ok(command) => {
                if let Err(e) = ledger.process_transaction(command) {
                    error!(err=?e, "Error processing transaction");
                }
            }
            Err(e) => error!(err=?e, "Error converting transaction record to system command"),
        }
    }

    let mut output_records = Vec::new();
    for (id, account) in ledger.get_accounts() {
        output_records.push(AccountRecord {
            id: *id,
            available: account.available,
            held: account.held,
            total: account.available + account.held,
            locked: account.locked,
        });
    }

    output_records
}

fn main() {
    init_logging();

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            error!(err=?e, "Error parsing command line arguments");
            std::process::exit(1);
        }
    };

    let reader = match csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(cli.input_path)
    {
        Ok(reader) => reader,
        Err(e) => {
            error!(err=?e, "Error opening input file");
            std::process::exit(1);
        }
    };

    let transactions = reader
        .into_deserialize::<TransactionRecord>()
        .map(|result| match result {
            Ok(record) => record,
            Err(e) => {
                // CSV is malformed, we shouldn't process this.
                error!(err=?e, "Error parsing CSV record");
                std::process::exit(1);
            }
        });

    let mut ledger = Ledger::default();
    let account_records = {
        let mut records = process_transactions(&mut ledger, transactions);
        records.sort_by_key(|r| r.id);
        records
    };

    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_writer(std::io::stdout());

    for record in account_records {
        if let Err(e) = writer.serialize(record) {
            error!(err=?e, "Error writing output record");
            std::process::exit(1);
        }
    }
}
