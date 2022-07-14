use serde::{Deserialize, Serialize};

//usage of #[serde(tag = "type")] is not possible because of bug
//https://github.com/BurntSushi/rust-csv/issues/278
#[derive(Deserialize)]
pub(crate) struct TrCsvRow {
    #[serde(alias = "type")]
    tr_type: String,
    client: u16,
    tx: u32,
    amount: Option<f64>,
}

//client, available, held, total, locked
#[derive(Serialize)]
pub struct ClientInfo {
    pub(crate) client: u16,
    pub(crate) available: f64,
    pub(crate) held: f64,
    pub(crate) total: f64,
    pub(crate) locked: bool,
}

impl From<(&u16, &Account)> for ClientInfo {
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
    pub available: f64,
    pub held: f64,
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
pub enum Tr {
    Deposit { client: u16, tx: u32, amount: f64 },
    Withdrawal { client: u16, tx: u32, amount: f64 },
    Dispute { client: u16, tx: u32 },
    Resolve { client: u16, tx: u32 },
    Chargeback { client: u16, tx: u32 },
}

impl TryFrom<TrCsvRow> for Tr {
    type Error = &'static str;

    fn try_from(csv_row: TrCsvRow) -> std::result::Result<Self, Self::Error> {
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

//TODO: create transaction info struct. Store there client, tx, amount
