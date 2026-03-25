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
        tx_id: TransactionId,
        client_id: ClientId,
        action: DisputeAction,
    },
}

#[derive(Debug)]
pub enum TransactionError {
    AccountLocked(ClientId),
    AccountNotFound(ClientId),
    InsufficientFunds,
    TransactionAlreadyDisputed(TransactionId),
    TransactionNotFound(TransactionId),
}

#[derive(Debug)]
pub struct Transaction {
    tx_id: TransactionId,
    amount: Decimal,
    disputed: bool,
}

#[derive(Debug)]
pub struct ClientAccount {
    pub available: Decimal,
    pub held: Decimal,
    pub locked: bool,
}

pub struct Ledger {
    data: std::collections::HashMap<ClientId, (ClientAccount, TransactionHistory)>,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    pub fn process_transaction(&mut self, command: TransactionCommand) -> Result<(), TransactionError> {
        tracing::debug!(command = ?command, "Processing system command");
        match command {
            TransactionCommand::ProcessMovement {
                tx_id,
                client_id,
                movement,
            } => todo!(),
            TransactionCommand::ProcessDispute {
                tx_id,
                client_id,
                action,
            } => todo!(),
        }

        Ok(())
    }

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

    // TODO: Test Successful Deposit and Withdrawal
    #[test]
    fn test_successful_deposit_and_withdrawal() {
        let mut ledger = Ledger::new();
        let deposit_command = TransactionCommand::ProcessMovement {
            tx_id: 1,
            client_id: 1,
            movement: Movement::Deposit(100.into()),
        };
        assert!(matches!(ledger.process_transaction(deposit_command), Ok(())));

        let withdrawal_command = TransactionCommand::ProcessMovement {
            tx_id: 2,
            client_id: 1,
            movement: Movement::Withdrawal(50.into()),
        };
        assert!(matches!(ledger.process_transaction(withdrawal_command), Ok(())));

        assert_account_state(&ledger, 1, 50.into(), 0.into(), false);
    }

    // TOOD: Test withdrawal with insufficient funds
    #[test]
    fn test_withdrawal_insufficient_funds() {
        let mut ledger = Ledger::new();
        let deposit_command = TransactionCommand::ProcessMovement {
            tx_id: 1,
            client_id: 1,
            movement: Movement::Deposit(100.into()),
        };
        assert!(matches!(ledger.process_transaction(deposit_command), Ok(())));
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

    // TODO: Test dispute and resolution flow
    #[test]
    fn test_dispute_and_resolution_flow() {
        let mut ledger = Ledger::new();
        let deposit_command = TransactionCommand::ProcessMovement {
            tx_id: 1,
            client_id: 1,
            movement: Movement::Deposit(100.into()),
        };
        assert!(matches!(ledger.process_transaction(deposit_command), Ok(())));
        let dispute_command = TransactionCommand::ProcessDispute {
            tx_id: 1,
            client_id: 1,
            action: DisputeAction::Open,
        };
        assert!(matches!(ledger.process_transaction(dispute_command), Ok(())));

        assert_account_state(&ledger, 1, 0.into(), 100.into(), false);

        let resolve_command = TransactionCommand::ProcessDispute {
            tx_id: 1,
            client_id: 1,
            action: DisputeAction::Resolve,
        };
        assert!(matches!(ledger.process_transaction(resolve_command), Ok(())));

        assert_account_state(&ledger, 1, 100.into(), 0.into(), false);
    }

    // TODO: Test dispute and chargeback flow
    #[test]
    fn test_dispute_and_chargeback_flow() {
        let mut ledger = Ledger::new();
        let deposit_command = TransactionCommand::ProcessMovement {
            tx_id: 1,
            client_id: 1,
            movement: Movement::Deposit(100.into()),
        };
        assert!(matches!(ledger.process_transaction(deposit_command), Ok(())));
        let dispute_command = TransactionCommand::ProcessDispute {
            tx_id: 1,
            client_id: 1,
            action: DisputeAction::Open,
        };
        assert!(matches!(ledger.process_transaction(dispute_command), Ok(())));

        assert_account_state(&ledger, 1, 0.into(), 100.into(), false);

        let chargeback_command = TransactionCommand::ProcessDispute {
            tx_id: 1,
            client_id: 1,
            action: DisputeAction::Chargeback,
        };
        assert!(matches!(ledger.process_transaction(chargeback_command), Ok(())));

        assert_account_state(&ledger, 1, 0.into(), 0.into(), true);
    }

    // TODO: Test lock account
    #[test]
    fn test_lock_account() {
        let mut ledger = Ledger::new();
        let deposit_command = TransactionCommand::ProcessMovement {
            tx_id: 1,
            client_id: 1,
            movement: Movement::Deposit(100.into()),
        };
        assert!(matches!(ledger.process_transaction(deposit_command), Ok(())));

        let dispute_command = TransactionCommand::ProcessDispute {
            tx_id: 1,
            client_id: 1,
            action: DisputeAction::Open,
        };
        assert!(matches!(ledger.process_transaction(dispute_command), Ok(())));

        let chargeback_command = TransactionCommand::ProcessDispute {
            tx_id: 1,
            client_id: 1,
            action: DisputeAction::Chargeback,
        };
        assert!(matches!(ledger.process_transaction(chargeback_command), Ok(())));

        assert_account_state(&ledger, 1, 0.into(), 0.into(), true);

        // Deposits should be accepted when account is locked
        let deposit_command = TransactionCommand::ProcessMovement {
            tx_id: 2,
            client_id: 1,
            movement: Movement::Deposit(50.into()),
        };
        assert!(matches!(ledger.process_transaction(deposit_command), Ok(())));

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
            tx_id: 2,
            client_id: 1,
            action: DisputeAction::Open,
        };
        assert!(matches!(ledger.process_transaction(dispute_command), Ok(())));

        assert_account_state(&ledger, 1, 0.into(), 50.into(), true);
    }

    // TODO: Test invalid dispute
    #[test]
    fn test_invalid_dispute() {
        let mut ledger = Ledger::new();
        let deposit_command = TransactionCommand::ProcessMovement {
            tx_id: 1,
            client_id: 1,
            movement: Movement::Deposit(100.into()),
        };
        assert!(matches!(ledger.process_transaction(deposit_command), Ok(())));

        assert_account_state(&ledger, 1, 100.into(), 0.into(), false);

        let dispute_command_1 = TransactionCommand::ProcessDispute {
            tx_id: 999,
            client_id: 1,
            action: DisputeAction::Open,
        };
        assert!(matches!(
            ledger.process_transaction(dispute_command_1),
            Err(TransactionError::TransactionNotFound(999))
        ));

        assert_account_state(&ledger, 1, 100.into(), 0.into(), false);

        let dispute_command_2 = TransactionCommand::ProcessDispute {
            tx_id: 1,
            client_id: 999,
            action: DisputeAction::Open,
        };
        assert!(matches!(
            ledger.process_transaction(dispute_command_2),
            Err(TransactionError::AccountNotFound(999))
        ));

        assert_account_state(&ledger, 1, 100.into(), 0.into(), false);

        let dispute_command_3 = TransactionCommand::ProcessDispute {
            tx_id: 1,
            client_id: 1,
            action: DisputeAction::Open,
        };
        assert!(matches!(ledger.process_transaction(dispute_command_3), Ok(())));

        assert_account_state(&ledger, 1, 0.into(), 100.into(), false);

        let dispute_command_4 = TransactionCommand::ProcessDispute {
            tx_id: 1,
            client_id: 1,
            action: DisputeAction::Open,
        };
        assert!(matches!(
            ledger.process_transaction(dispute_command_4),
            Err(TransactionError::TransactionAlreadyDisputed(1))
        ));

        assert_account_state(&ledger, 1, 0.into(), 100.into(), false);
    }
}
