use rust_decimal::Decimal;

pub type ClientId = u16;
pub type TransactionId = u32;

type TransactionHistory = std::collections::HashMap<TransactionId, Transaction>;

#[derive(Debug)]
pub enum Movement {
    Deposit(Decimal),
    Withdrawal(Decimal),
}

#[derive(Debug)]
pub enum DisputeAction {
    Open,
    Resolve,
    Chargeback,
}

#[derive(Debug)]
pub enum TransactionCommand {
    ProcessMovement {
        tx_id: TransactionId,
        client_id: ClientId,
        movement: Movement,
    },
    ProcessDispute {
        target_tx_id: TransactionId,
        client_id: ClientId,
        action: DisputeAction,
    },
}

#[derive(Debug)]
pub enum TransactionError {
    AccountLocked(ClientId),
    InsufficientFunds,
    TransactionAlreadyDisputed(TransactionId),
    TransactionNotDisputed(TransactionId),
    TransactionNotFound(TransactionId),
}

#[derive(Debug)]
pub struct Transaction {
    amount: Decimal,
    disputed: bool,
}

#[derive(Debug, Default)]
pub struct ClientAccount {
    pub available: Decimal,
    pub held: Decimal,
    pub locked: bool,
}

#[derive(Debug, Default)]
pub struct Ledger {
    data: std::collections::HashMap<ClientId, (ClientAccount, TransactionHistory)>,
}

impl Ledger {
    /// Processes a transaction command and updates the ledger state accordingly.
    ///
    /// # Errors
    /// Returns a `TransactionError` if the transaction cannot be processed.
    pub fn process_transaction(
        &mut self,
        command: TransactionCommand,
    ) -> Result<(), TransactionError> {
        tracing::debug!(command = ?command, "Processing system command");
        match command {
            TransactionCommand::ProcessMovement {
                tx_id,
                client_id,
                movement,
            } => {
                let (account, transaction_history) = self
                    .data
                    .entry(client_id)
                    .or_insert_with(|| (ClientAccount::default(), TransactionHistory::new()));

                match movement {
                    Movement::Deposit(amount) => {
                        account.available += amount;
                        transaction_history.insert(
                            tx_id,
                            Transaction {
                                amount,
                                disputed: false,
                            },
                        );
                    }
                    Movement::Withdrawal(amount) => {
                        if account.locked {
                            return Err(TransactionError::AccountLocked(client_id));
                        }
                        if account.available < amount {
                            return Err(TransactionError::InsufficientFunds);
                        }
                        account.available -= amount;
                        transaction_history.insert(
                            tx_id,
                            Transaction {
                                amount: -amount,
                                disputed: false,
                            },
                        );
                    }
                }
            }
            TransactionCommand::ProcessDispute {
                target_tx_id,
                client_id,
                action,
            } => {
                let (account, transaction_history) = self
                    .data
                    .entry(client_id)
                    .or_insert_with(|| (ClientAccount::default(), TransactionHistory::new()));

                let transaction = transaction_history
                    .get_mut(&target_tx_id)
                    .ok_or(TransactionError::TransactionNotFound(target_tx_id))?;

                match action {
                    DisputeAction::Open => {
                        if transaction.disputed {
                            return Err(TransactionError::TransactionAlreadyDisputed(target_tx_id));
                        }
                        // I won't raise insufficient funds error for disputes, as the transaction was already processed and the funds were available at that time. Instead, I'll just move the disputed amount from available to held and allow for negative balances.
                        account.available -= transaction.amount;
                        account.held += transaction.amount;
                        transaction.disputed = true;
                    }
                    DisputeAction::Resolve => {
                        if !transaction.disputed {
                            return Err(TransactionError::TransactionNotDisputed(target_tx_id));
                        }
                        account.available += transaction.amount;
                        account.held -= transaction.amount;
                        transaction.disputed = false;
                    }
                    DisputeAction::Chargeback => {
                        if !transaction.disputed {
                            return Err(TransactionError::TransactionNotDisputed(target_tx_id));
                        }
                        account.held -= transaction.amount;
                        account.locked = true;
                        transaction.disputed = false;
                    }
                }
            }
        }

        Ok(())
    }

    /// Returns an iterator over all client accounts in the ledger.
    pub fn get_accounts(&self) -> impl Iterator<Item = (&ClientId, &ClientAccount)> {
        self.data
            .iter()
            .map(|(client_id, (account, _))| (client_id, account))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_account_state(
        ledger: &Ledger,
        client_id: ClientId,
        expected_accessible: Decimal,
        expected_held: Decimal,
        expected_locked: bool,
    ) {
        let accounts: Vec<_> = ledger.get_accounts().collect();
        assert_eq!(accounts.len(), 1);
        let (id, account) = accounts[0];
        assert_eq!(*id, client_id);
        assert_eq!(account.available, expected_accessible);
        assert_eq!(account.held, expected_held);
        assert_eq!(account.locked, expected_locked);
    }

    #[test]
    fn test_successful_deposit_and_withdrawal() {
        let mut ledger = Ledger::default();
        let deposit_command = TransactionCommand::ProcessMovement {
            tx_id: 1,
            client_id: 1,
            movement: Movement::Deposit(100.into()),
        };
        assert!(matches!(
            ledger.process_transaction(deposit_command),
            Ok(())
        ));

        let withdrawal_command = TransactionCommand::ProcessMovement {
            tx_id: 2,
            client_id: 1,
            movement: Movement::Withdrawal(50.into()),
        };
        assert!(matches!(
            ledger.process_transaction(withdrawal_command),
            Ok(())
        ));

        assert_account_state(&ledger, 1, 50.into(), 0.into(), false);
    }

    #[test]
    fn test_withdrawal_insufficient_funds() {
        let mut ledger = Ledger::default();
        let deposit_command = TransactionCommand::ProcessMovement {
            tx_id: 1,
            client_id: 1,
            movement: Movement::Deposit(100.into()),
        };
        assert!(matches!(
            ledger.process_transaction(deposit_command),
            Ok(())
        ));
        let withdrawal_command = TransactionCommand::ProcessMovement {
            tx_id: 2,
            client_id: 1,
            movement: Movement::Withdrawal(150.into()),
        };
        assert!(matches!(
            ledger.process_transaction(withdrawal_command),
            Err(TransactionError::InsufficientFunds)
        ));

        assert_account_state(&ledger, 1, 100.into(), 0.into(), false);
    }

    #[test]
    fn test_dispute_and_resolution_flow() {
        let mut ledger = Ledger::default();
        let deposit_command = TransactionCommand::ProcessMovement {
            tx_id: 1,
            client_id: 1,
            movement: Movement::Deposit(100.into()),
        };
        assert!(matches!(
            ledger.process_transaction(deposit_command),
            Ok(())
        ));
        let dispute_command = TransactionCommand::ProcessDispute {
            target_tx_id: 1,
            client_id: 1,
            action: DisputeAction::Open,
        };
        assert!(matches!(
            ledger.process_transaction(dispute_command),
            Ok(())
        ));

        assert_account_state(&ledger, 1, 0.into(), 100.into(), false);

        let resolve_command = TransactionCommand::ProcessDispute {
            target_tx_id: 1,
            client_id: 1,
            action: DisputeAction::Resolve,
        };
        assert!(matches!(
            ledger.process_transaction(resolve_command),
            Ok(())
        ));

        assert_account_state(&ledger, 1, 100.into(), 0.into(), false);
    }

    #[test]
    fn test_dispute_and_chargeback_flow() {
        let mut ledger = Ledger::default();
        let deposit_command = TransactionCommand::ProcessMovement {
            tx_id: 1,
            client_id: 1,
            movement: Movement::Deposit(100.into()),
        };
        assert!(matches!(
            ledger.process_transaction(deposit_command),
            Ok(())
        ));
        let dispute_command = TransactionCommand::ProcessDispute {
            target_tx_id: 1,
            client_id: 1,
            action: DisputeAction::Open,
        };
        assert!(matches!(
            ledger.process_transaction(dispute_command),
            Ok(())
        ));

        assert_account_state(&ledger, 1, 0.into(), 100.into(), false);

        let chargeback_command = TransactionCommand::ProcessDispute {
            target_tx_id: 1,
            client_id: 1,
            action: DisputeAction::Chargeback,
        };
        assert!(matches!(
            ledger.process_transaction(chargeback_command),
            Ok(())
        ));

        assert_account_state(&ledger, 1, 0.into(), 0.into(), true);
    }

    #[test]
    fn test_lock_account() {
        let mut ledger = Ledger::default();
        let deposit_command = TransactionCommand::ProcessMovement {
            tx_id: 1,
            client_id: 1,
            movement: Movement::Deposit(100.into()),
        };
        assert!(matches!(
            ledger.process_transaction(deposit_command),
            Ok(())
        ));

        let dispute_command = TransactionCommand::ProcessDispute {
            target_tx_id: 1,
            client_id: 1,
            action: DisputeAction::Open,
        };
        assert!(matches!(
            ledger.process_transaction(dispute_command),
            Ok(())
        ));

        let chargeback_command = TransactionCommand::ProcessDispute {
            target_tx_id: 1,
            client_id: 1,
            action: DisputeAction::Chargeback,
        };
        assert!(matches!(
            ledger.process_transaction(chargeback_command),
            Ok(())
        ));

        assert_account_state(&ledger, 1, 0.into(), 0.into(), true);

        // Deposits should be accepted when account is locked
        let deposit_command = TransactionCommand::ProcessMovement {
            tx_id: 2,
            client_id: 1,
            movement: Movement::Deposit(50.into()),
        };
        assert!(matches!(
            ledger.process_transaction(deposit_command),
            Ok(())
        ));

        assert_account_state(&ledger, 1, 50.into(), 0.into(), true);

        // Withdrawals should be rejected when account is locked
        let withdrawal_command = TransactionCommand::ProcessMovement {
            tx_id: 3,
            client_id: 1,
            movement: Movement::Withdrawal(10.into()),
        };
        assert!(matches!(
            ledger.process_transaction(withdrawal_command),
            Err(TransactionError::AccountLocked(1))
        ));

        assert_account_state(&ledger, 1, 50.into(), 0.into(), true);

        // Disputes should be accepted when account is locked
        let dispute_command = TransactionCommand::ProcessDispute {
            target_tx_id: 2,
            client_id: 1,
            action: DisputeAction::Open,
        };
        assert!(matches!(
            ledger.process_transaction(dispute_command),
            Ok(())
        ));

        assert_account_state(&ledger, 1, 0.into(), 50.into(), true);
    }

    #[test]
    fn test_invalid_dispute() {
        let mut ledger = Ledger::default();
        let deposit_command = TransactionCommand::ProcessMovement {
            tx_id: 1,
            client_id: 1,
            movement: Movement::Deposit(100.into()),
        };
        assert!(matches!(
            ledger.process_transaction(deposit_command),
            Ok(())
        ));

        assert_account_state(&ledger, 1, 100.into(), 0.into(), false);

        let dispute_command_1 = TransactionCommand::ProcessDispute {
            target_tx_id: 999,
            client_id: 1,
            action: DisputeAction::Open,
        };
        assert!(matches!(
            ledger.process_transaction(dispute_command_1),
            Err(TransactionError::TransactionNotFound(999))
        ));

        assert_account_state(&ledger, 1, 100.into(), 0.into(), false);

        let dispute_command_2 = TransactionCommand::ProcessDispute {
            target_tx_id: 1,
            client_id: 1,
            action: DisputeAction::Open,
        };
        assert!(matches!(
            ledger.process_transaction(dispute_command_2),
            Ok(())
        ));

        assert_account_state(&ledger, 1, 0.into(), 100.into(), false);

        let dispute_command_3 = TransactionCommand::ProcessDispute {
            target_tx_id: 1,
            client_id: 1,
            action: DisputeAction::Open,
        };
        assert!(matches!(
            ledger.process_transaction(dispute_command_3),
            Err(TransactionError::TransactionAlreadyDisputed(1))
        ));

        assert_account_state(&ledger, 1, 0.into(), 100.into(), false);
    }
}
