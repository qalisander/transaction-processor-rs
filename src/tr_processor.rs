use crate::data::*;
use std::collections::HashMap;
use std::slice::Iter;

pub struct TrProcessor {
    tx_to_tr_infos: HashMap<u32, TrInfo>,
    client_to_account: HashMap<u16, Account>,
}

impl TrProcessor {
    pub fn new() -> Self {
        TrProcessor {
            tx_to_tr_infos: HashMap::new(),
            client_to_account: HashMap::new(),
        }
    }

    pub fn process(&mut self, trs: impl Iterator<Item = Tr>) {
        for tr in trs {
            match tr {
                Tr::Deposit { client, tx, amount } => {
                    let account = self
                        .client_to_account
                        .entry(client)
                        .or_insert_with(Account::new);
                    if account.locked {
                        continue;
                    }

                    account.available += amount;
                    self.tx_to_tr_infos
                        .insert(tx, TrInfo::new(client, amount))
                        .and_then::<(), _>(|_| panic!("Not unique tx! tx: {tx}"));
                }
                Tr::Withdrawal { client, tx, amount } => {
                    let account = self
                        .client_to_account
                        .entry(client)
                        .or_insert_with(Account::new);
                    if account.locked {
                        continue;
                    }

                    if account.available >= amount {
                        account.available -= amount;
                        self.tx_to_tr_infos
                            .insert(tx, TrInfo::new(client, -amount))
                            .and_then::<(), _>(|_| panic!("Not unique tx! tx: {tx}"));
                    }
                }
                Tr::Dispute { client, tx } => {
                    let account = self
                        .client_to_account
                        .entry(client)
                        .or_insert_with(Account::new);
                    if account.locked {
                        continue;
                    }

                    match self.tx_to_tr_infos.get_mut(&tx) {
                        Some(TrInfo {
                            client: tr_client,
                            amount,
                            has_disputed,
                        }) if !*has_disputed && *tr_client == client => {
                            *has_disputed = true;
                            // TODO: add method froze amount
                            if *amount >= 0.0 {
                                account.available -= *amount;
                                account.held += *amount;
                            }
                        }
                        _ => (),
                    }
                }
                Tr::Resolve { client, tx } => {
                    let account = self
                        .client_to_account
                        .entry(client)
                        .or_insert_with(Account::new);
                    if account.locked {
                        continue;
                    }

                    match self.tx_to_tr_infos.get(&tx) {
                        Some(&TrInfo {
                            client: tr_client,
                            amount,
                            has_disputed,
                        }) if has_disputed && tr_client == client => {
                            if amount >= 0.0 {
                                account.available += amount;
                                account.held -= amount;
                            }
                            self.tx_to_tr_infos.remove(&tx);
                        }
                        _ => (),
                    }
                }
                Tr::Chargeback { client, tx } => {
                    let account = self
                        .client_to_account
                        .entry(client)
                        .or_insert_with(Account::new);
                    if account.locked {
                        continue;
                    }

                    match self.tx_to_tr_infos.get(&tx) {
                        Some(&TrInfo {
                            client: tr_client,
                            amount,
                            has_disputed,
                        }) if has_disputed && tr_client == client => {
                            if amount >= 0.0 {
                                account.held -= amount;
                            } else {
                                account.available -= amount;
                            }
                            self.tx_to_tr_infos.remove(&tx);
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    pub fn get_client_records(&self) -> impl Iterator<Item = ClientRecord> + '_ {
        self.client_to_account.iter().map(ClientRecord::from)
    }

    fn process_single(&mut self, tr: Tr){
        self.process([tr].into_iter());
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
        let info = get_client_info(&processor, 1);
        assert_eq!(info.available, 250.0)
    }

    #[test]
    fn dispute_resolve() {
        let mut processor = TrProcessor::new();
        let trs = [
            Tr::Deposit {
                client: 1,
                tx: 1,
                amount: 200.0,
            },
            Tr::Withdrawal{
                client: 1,
                tx: 2,
                amount: 100.0
            },
        ];

        processor.process_single(Tr::Dispute { client: 1, tx: 2 });
        processor.process(trs.into_iter());
        let info = get_client_info(&processor, 1);
        assert_eq!(info.available, 0.0);
        assert_eq!(info.held, 100.0);

        processor.process_single(Tr::Resolve { client: 1, tx: 1 });
        let info = get_client_info(&processor, 1);
        assert_eq!(info.available, 100.0);
        assert_eq!(info.held, 0.0);

        processor.process_single(Tr::Dispute { client: 1, tx: 1 });
        let info = get_client_info(&processor, 1);
        assert_eq!(info.available, 0.0);
        assert_eq!(info.held, 200.0);

        processor.process_single(Tr::Chargeback { client: 1, tx: 1 });
        let info = get_client_info(&processor, 1);
        assert_eq!(info.available, 0.0);
        assert_eq!(info.held, 0.0);
    }

    #[test]
    fn dispute_chargeback() {
        let mut processor = TrProcessor::new();
        let trs = [
            Tr::Deposit {
                client: 1,
                tx: 1,
                amount: 200.0,
            },
            Tr::Withdrawal{
                client: 1,
                tx: 2,
                amount: 100.0
            },
        ];

        processor.process(trs.into_iter());
        processor.process_single(Tr::Dispute { client: 1, tx: 2 });
        let info = get_client_info(&processor, 1);
        assert_eq!(info.available, 100.0);
        assert_eq!(info.held, 0.0);

        processor.process_single(Tr::Chargeback { client: 1, tx: 2 });
        let info = get_client_info(&processor, 1);
        assert_eq!(info.available, 200.0);
        assert_eq!(info.held, 0.0);

        processor.process_single(Tr::Dispute { client: 1, tx: 1 });
        let info = get_client_info(&processor, 1);
        assert_eq!(info.available, 0.0);
        assert_eq!(info.held, 200.0);

        processor.process_single(Tr::Chargeback { client: 1, tx: 1 });
        let info = get_client_info(&processor, 1);
        assert_eq!(info.available, 0.0);
        assert_eq!(info.held, 0.0);
    }
    
    fn get_client_info(processor: &TrProcessor, client: u16) -> ClientRecord {
        processor
            .get_client_records()
            .find(|ci| ci.client == client)
            .expect("Client not found!")
    }
}
