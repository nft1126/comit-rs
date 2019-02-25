pub mod block;
pub mod event;
pub mod transaction;

pub use self::{block::BlockQuery, event::EventQuery, transaction::TransactionQuery};
use ethereum_support::{Transaction, TransactionReceipt};
use ethereum_types::{clean_0x, H256};

fn to_h256<S: AsRef<str>>(tx_id: S) -> Option<H256> {
    let tx_id = tx_id.as_ref();

    match hex::decode(clean_0x(tx_id)) {
        Ok(bytes) => Some(H256::from_slice(&bytes)),
        Err(e) => {
            warn!("skipping {} because it is not valid hex: {:?}", tx_id, e);
            None
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum PayloadKind {
    Id {
        id: H256,
    },
    Transaction {
        transaction: Box<Transaction>,
    },
    Receipt {
        receipt: Box<TransactionReceipt>,
    },
    TransactionAndReceipt {
        transaction: Box<Transaction>,
        receipt: Box<TransactionReceipt>,
    },
}
