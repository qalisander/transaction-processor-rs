use serde::{Deserialize, Serialize};

type Money = f64;

//usage of #[serde(tag = "type")] is not possible because of bug
//https://github.com/BurntSushi/rust-csv/issues/278
#[derive(Deserialize)]
pub(crate) struct TrRecord {
    #[serde(alias = "type")]
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

// TODO: use struct Tr and TrType
#[derive(Debug, Copy, Clone)]
pub enum Tr {
    Deposit { client: u16, tx: u32, amount: Money },
    Withdrawal { client: u16, tx: u32, amount: Money },
    Dispute { client: u16, tx: u32 },
    Resolve { client: u16, tx: u32 },
    Chargeback { client: u16, tx: u32 },
}

impl TryFrom<TrRecord> for Tr {
    type Error = &'static str;

    fn try_from(csv_row: TrRecord) -> std::result::Result<Self, Self::Error> {
        let client = csv_row.client;
        let tx = csv_row.tx;

        match &*csv_row.tr_type {
            "deposit" => match csv_row.amount {
                None => Err("Not valid amount!"),
                Some(amount) => Ok(Tr::Deposit { client, tx, amount }),
            },
            "withdrawal" => match csv_row.amount {
                None => Err("Not valid amount!"),
                Some(amount) => Ok(Tr::Withdrawal { client, tx, amount }),
            },
            "dispute" => Ok(Tr::Dispute { tx, client }),
            "resolve" => Ok(Tr::Resolve { tx, client }),
            "chargeback" => Ok(Tr::Chargeback { tx, client }),
            _ => Err("Not valid csv row!"),
        }
    }
}

pub struct TrInfo {
    pub(crate) client: u16,
    pub(crate) amount: Money,
    pub(crate) has_disputed: bool,
}

impl TrInfo {
    pub fn new(client: u16, amount: Money) -> Self {
        Self { client, amount, has_disputed: false }
    }
}
