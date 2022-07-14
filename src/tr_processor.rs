use crate::data::*;
use std::collections::HashMap;
use std::slice::Iter;


pub struct TrProcessor {
    tx_to_amount: HashMap<u32, f64>,
    client_to_account: HashMap<u16, Account>
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
                    unimplemented!()
                }
                Tr::Withdrawal { .. } => {
                    unimplemented!()
                }
                Tr::Dispute { .. } => {
                    unimplemented!()
                }
                Tr::Resolve { .. } => {
                    unimplemented!()
                }
                Tr::Chargeback { .. } => {
                    unimplemented!()
                }
            }
        }
    }
    
    pub fn get_client_infos(&self) -> impl Iterator<Item=ClientInfo> + '_{
        self.client_to_account.iter().map(ClientInfo::from)
    }
}
