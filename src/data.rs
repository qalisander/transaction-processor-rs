use anyhow::{anyhow, Error, Result};
use serde::{Deserialize, Serialize};

type Money = f64;

//usage of #[serde(tag = "type")] is not possible because of bug
//https://github.com/BurntSushi/rust-csv/issues/278
#[derive(Deserialize)]
pub(crate) struct TrRecord {
    #[serde(rename = "type")]
    tr_type: String,
    client: u16,
    tx: u32,
    amount: Option<Money>,
}

//Client, available, held, total, locked
#[derive(Serialize)]
pub struct ClientRecord {
    pub(crate) client: u16,
    pub(crate) available: Money,
    pub(crate) held: Money,
    pub(crate) total: Money,
    pub(crate) locked: bool,
}

impl From<(&u16, &Account)> for ClientRecord {
    fn from((client, account): (&u16, &Account)) -> Self {
        Self {
            client: *client,
            available: account.available,
            held: account.held,
            total: account.available + account.held,
            locked: account.locked,
        }
    }
}

pub struct Account {
    pub available: Money,
    pub held: Money,
    pub locked: bool,
}

impl Account {
    pub fn new() -> Self {
        Self {
            available: 0.0,
            held: 0.0,
            locked: false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Tr {
    pub tp: TrType,
    pub client: u16,
    pub tx: u32,
}

impl Tr {
    pub fn new(tp: TrType, client: u16, tx: u32) -> Self {
        Self { tp, client, tx }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TrType {
    Deposit(Money),
    Withdrawal(Money),
    Dispute,
    Resolve,
    Chargeback,
}

impl TryFrom<TrRecord> for Tr {
    type Error = Error;

    fn try_from(csv_row: TrRecord) -> Result<Self> {
        let client = csv_row.client;
        let tx = csv_row.tx;
        let tr_type = match &*csv_row.tr_type {
            "deposit" => match csv_row.amount {
                None => Err(anyhow!("Not valid amount! tx: {tx}")),
                Some(amount) => Ok(TrType::Deposit(amount)),
            },
            "withdrawal" => match csv_row.amount {
                None => Err(anyhow!("Not valid amount! tx: {tx}")),
                Some(amount) => Ok(TrType::Withdrawal(amount)),
            },
            "dispute" => Ok(TrType::Dispute),
            "resolve" => Ok(TrType::Resolve),
            "chargeback" => Ok(TrType::Chargeback),
            _ => Err(anyhow!("Not valid csv row! tx: {tx}")),
        };
        tr_type.map(|tp| Tr { tp, tx, client })
    }
}

pub struct TrInfo {
    pub(crate) client: u16,
    pub(crate) amount: Money,
    pub(crate) has_disputed: bool,
}

impl TrInfo {
    pub fn new(client: u16, amount: Money) -> Self {
        Self {
            client,
            amount,
            has_disputed: false,
        }
    }
}
