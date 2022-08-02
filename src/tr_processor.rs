use crate::data::*;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub struct TrProcessor {
    tx_to_tr_info: HashMap<u32, TrInfo>,
    client_to_account: HashMap<u16, Account>,
}

impl TrProcessor {
    pub fn new() -> Self {
        TrProcessor {
            tx_to_tr_info: HashMap::new(),
            client_to_account: HashMap::new(),
        }
    }

    pub fn try_process<'a>(
        &'a mut self,
        trs: impl Iterator<Item = Tr> + 'a,
    ) -> impl Iterator<Item = Result<()>> + 'a {
        trs.map(|tr| {
            let tx = tr.tx;
            let client = tr.client;
            let account = self
                .client_to_account
                .entry(client)
                .or_insert_with(Account::new);
            if account.locked {
                return Err(anyhow!("Account is blocked! tx: {tx}"));
            }

            match tr.tp {
                TrType::Deposit(amount) => {
                    if amount.is_sign_negative() {
                        return Err(anyhow!("Amount is negative!: tx: {tx}; amount: {amount})"));
                    }
                    let is_unique = self
                        .tx_to_tr_info
                        .insert(tx, TrInfo::new(client, amount))
                        .is_none();
                    if !is_unique {
                        return Err(anyhow!("Not unique tx! tx: {tx}"));
                    }
                    account.available += amount;
                }
                TrType::Withdrawal(amount) => {
                    if amount.is_sign_negative() {
                        return Err(anyhow!("Amount is negative!: tx: {tx}; amount: {amount})"));
                    }
                    if account.available < amount {
                        let available = account.available;
                        return Err(anyhow!(
                            "Not enough funds!: tx: {tx}; client: {client}; \
                            available: {available}; amount: {amount})"
                        ));
                    }

                    let is_unique = self
                        .tx_to_tr_info
                        .insert(tx, TrInfo::new(client, -amount))
                        .is_none();
                    if !is_unique {
                        return Err(anyhow!("Not unique tx! tx: {tx}"));
                    }
                    account.available -= amount;
                }
                TrType::Dispute => match self.tx_to_tr_info.get_mut(&tx) {
                    Some(TrInfo {
                        client: tr_client,
                        amount,
                        has_disputed,
                    }) => {
                        if *has_disputed {
                            return Err(anyhow!("Already under dispute! tx: {tx}"));
                        }
                        if *tr_client != client {
                            return Err(anyhow!(
                                "Invalid client id! tx: {tx}; client: {tr_client}"
                            ));
                        }

                        *has_disputed = true;
                        if amount.is_sign_positive() {
                            account.available -= *amount;
                            account.held += *amount;
                        }
                    }
                    None => {
                        return Err(anyhow!("Tx not found for dispute! tx: {tx}"));
                    }
                },
                TrType::Resolve => match self.tx_to_tr_info.get(&tx) {
                    Some(&TrInfo {
                        client: tr_client,
                        amount,
                        has_disputed,
                    }) => {
                        if !has_disputed {
                            return Err(anyhow!("Resolving not disputed transaction! tx: {tx}"));
                        }
                        if tr_client != client {
                            return Err(anyhow!(
                                "Invalid client id! tx: {tx}; client: {tr_client}"
                            ));
                        }

                        if amount.is_sign_positive() {
                            account.available += amount;
                            account.held -= amount;
                        }
                        self.tx_to_tr_info.remove(&tx);
                    }
                    None => {
                        return Err(anyhow!("Tx not found for resolve! tx: {tx}"));
                    }
                },
                TrType::Chargeback => match self.tx_to_tr_info.get(&tx) {
                    Some(&TrInfo {
                        client: tr_client,
                        amount,
                        has_disputed,
                    }) => {
                        if !has_disputed {
                            return Err(anyhow!(
                                "Trying to chargeback not disputed transaction! tx: {tx}"
                            ));
                        }
                        if tr_client != client {
                            return Err(anyhow!(
                                "Invalid client id! tx: {tx}; client: {tr_client}"
                            ));
                        }

                        if amount.is_sign_positive() {
                            account.held -= amount;
                        } else {
                            account.available -= amount;
                        }
                        account.locked = true;
                        self.tx_to_tr_info.remove(&tx);
                    }
                    None => {
                        return Err(anyhow!("Tx not found for chargeback! tx: {tx}"));
                    }
                },
            }
            Ok(())
        })
    }

    pub fn get_client_records(&self) -> impl Iterator<Item = ClientRecord> + '_ {
        self.client_to_account.iter().map(ClientRecord::from)
    }

    pub fn process(&mut self, trs: impl Iterator<Item = Tr>) {
        for _ in self.try_process(trs) {}
    }

    fn process_single(&mut self, tr: Tr) {
        self.process([tr].into_iter());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::TrType::*;
    use rust_decimal_macros::dec;

    #[test]
    fn deposit_withdrawal() {
        let mut processor = TrProcessor::new();
        let trs = [
            Tr::new(Deposit(dec!(100.0)), 1, 1),
            Tr::new(Withdrawal(dec!(50.0)), 1, 2),
            Tr::new(Deposit(dec!(200.0)), 2, 3),
            Tr::new(Deposit(dec!(200.0)), 1, 4),
        ];

        processor.process(trs.into_iter());
        let info = get_client_info(&processor, 1);
        assert_eq!(info.available, dec!(250.0));

        processor.process_single(Tr::new(Withdrawal(dec!(251.0)), 1, 5));
        assert_eq!(info.total, dec!(250.0), "Not sufficient funds");
    }

    #[test]
    fn dispute_resolve() {
        let mut processor = TrProcessor::new();
        processor.process(
            [
                Tr::new(Deposit(dec!(200.0)), 1, 1),
                Tr::new(Withdrawal(dec!(100.0)), 1, 2),
            ]
            .into_iter(),
        );

        processor.process_single(Tr::new(Dispute, 1, 2));
        let info = get_client_info(&processor, 1);
        assert_eq!(info.available, dec!(100.0));
        assert_eq!(info.held, dec!(0.0));

        processor.process_single(Tr::new(Resolve, 1, 2));
        let info = get_client_info(&processor, 1);
        assert_eq!(info.available, dec!(100.0));
        assert_eq!(info.held, dec!(0.0));

        processor.process_single(Tr::new(Resolve, 1, 1));
        let info = get_client_info(&processor, 1);
        assert_eq!(
            info.available,
            dec!(100.0),
            "Can not resolve not disputed transaction"
        );

        processor.process_single(Tr::new(Dispute, 1, 1));
        let info = get_client_info(&processor, 1);
        assert_eq!(info.available, dec!(-100.0));
        assert_eq!(info.held, dec!(200.0));

        processor.process_single(Tr::new(Resolve, 1, 1));
        let info = get_client_info(&processor, 1);
        assert_eq!(info.available, dec!(100.0));
        assert_eq!(info.held, dec!(0.0));
    }

    #[test]
    fn dispute_chargeback() {
        let mut processor = TrProcessor::new();
        processor.process(
            [
                Tr::new(Deposit(dec!(200.0)), 1, 1),
                Tr::new(Withdrawal(dec!(100.0)), 1, 2),
            ]
            .into_iter(),
        );

        processor.process_single(Tr::new(Chargeback, 1, 2));
        let info = get_client_info(&processor, 1);
        assert_eq!(
            info.available,
            dec!(100.0),
            "Can not chargeback not disputed transaction"
        );

        processor.process_single(Tr::new(Dispute, 1, 2));
        let info = get_client_info(&processor, 1);
        assert_eq!(info.available, dec!(100.0));
        assert_eq!(info.held, dec!(0.0));

        processor.process_single(Tr::new(Chargeback, 1, 2));
        let info = get_client_info(&processor, 1);
        assert_eq!(info.available, dec!(200.0));
        assert_eq!(info.held, dec!(0.0));
        assert!(info.locked);

        processor.process_single(Tr::new(Dispute, 1, 1));
        let info = get_client_info(&processor, 1);
        assert_eq!(
            info.available,
            dec!(200.0),
            "Account is locked. Same balance"
        );
    }

    #[test]
    fn negative_amount_test() {
        let mut processor = TrProcessor::new();

        processor.process_single(Tr::new(Deposit(dec!(-100)), 1, 1));
        let client = get_client_info(&processor, 1);
        assert_eq!(client.available, dec!(0));
        assert_eq!(client.held, dec!(0));

        processor.process_single(Tr::new(Withdrawal(dec!(-100)), 1, 2));
        let client = get_client_info(&processor, 1);
        assert_eq!(client.available, dec!(0));
        assert_eq!(client.held, dec!(0));
    }

    fn get_client_info(processor: &TrProcessor, client: u16) -> ClientRecord {
        processor
            .get_client_records()
            .find(|ci| ci.client == client)
            .expect("Client not found!")
    }
}
