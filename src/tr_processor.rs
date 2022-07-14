use crate::data::*;
use std::collections::HashMap;
use std::slice::Iter;

pub struct TrProcessor {
    tx_to_amount: HashMap<u32, f64>,
    client_to_account: HashMap<u16, Account>,
}

impl TrProcessor {
    pub fn new() -> Self {
        TrProcessor {
            tx_to_amount: HashMap::new(),
            client_to_account: HashMap::new(),
        }
    }

    pub fn process(&mut self, trs: impl Iterator<Item = Tr>) {
        for tr in trs {
            match tr {
                Tr::Deposit { client, tx, amount } => {
                    let account = self
                        .client_to_account
                        .entry(client) // TODO: and_modify ?
                        .or_insert_with(Account::new);

                    if !account.locked {
                        account.available += amount;
                        if self.tx_to_amount.insert(tx, amount).is_some() {
                            panic!("Not unique tx! {tx}");
                        };
                    }
                }
                Tr::Withdrawal { client, tx, amount } => {
                    let account = self
                        .client_to_account
                        .entry(client)
                        .or_insert_with(Account::new);

                    if !account.locked && account.available >= amount {
                        account.available -= amount;
                        if self.tx_to_amount.insert(tx, -amount).is_some() {
                            panic!("Not unique tx! {tx}");
                        };
                    }
                }
                Tr::Dispute { client, tx } => {
                    let account = self
                        .client_to_account
                        .entry(client)
                        .or_insert_with(Account::new);

                    unimplemented!()
                }
                // TODO: it's necessary to check whether a transaction been under dispute
                Tr::Resolve { client, tx } => {
                    unimplemented!()
                }
                Tr::Chargeback { client, tx } => {
                    unimplemented!()
                }
            }
        }
    }

    pub fn get_client_infos(&self) -> impl Iterator<Item = ClientInfo> + '_ {
        self.client_to_account.iter().map(ClientInfo::from)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deposit_withdrawal() {
        let mut processor = TrProcessor::new();
        let trs = [
            Tr::Deposit {
                client: 1,
                tx: 1,
                amount: 100.0,
            },
            Tr::Withdrawal {
                client: 1,
                tx: 2,
                amount: 50.0,
            },
            Tr::Deposit {
                client: 2,
                tx: 3,
                amount: 200.0,
            },
            Tr::Deposit {
                client: 1,
                tx: 4,
                amount: 200.0,
            },
        ];

        processor.process(trs.into_iter());
        let info = processor
            .get_client_infos()
            .find(|ci| ci.client == 1)
            .expect("Client not found!");
        assert_eq!(info.available, 250.0)
    }
}
