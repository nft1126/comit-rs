use bitcoin_support::{
    Address, BitcoinQuantity, IntoP2wpkhAddress, Network, PubkeyHash, Transaction, TransactionId,
};
use secp256k1_support::PublicKey;
use swap_protocols::ledger::Ledger;

#[derive(Clone, Debug, PartialEq)]
pub struct Bitcoin {
    network: Network,
}

impl Bitcoin {
    pub fn new(network: Network) -> Self {
        Bitcoin { network }
    }

    pub fn regtest() -> Self {
        Bitcoin {
            network: Network::Regtest,
        }
    }
}

//TODO: fix with #376
impl Default for Bitcoin {
    fn default() -> Self {
        Bitcoin {
            network: Network::Regtest,
        }
    }
}

impl Ledger for Bitcoin {
    type Quantity = BitcoinQuantity;
    type TxId = TransactionId;
    type Pubkey = PublicKey;
    type Address = Address;
    type Identity = PubkeyHash;
    type Transaction = Transaction;

    fn symbol() -> String {
        String::from("BTC")
    }

    fn address_for_identity(&self, pubkeyhash: PubkeyHash) -> Address {
        pubkeyhash.into_p2wpkh_address(self.network)
    }
}

impl Bitcoin {
    pub fn network(&self) -> Network {
        self.network
    }
}