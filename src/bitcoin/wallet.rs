use crate::bitcoin::{Address, Amount, Client, Network, WalletInfoResponse};
use crate::seed::Seed;
use ::bitcoin::{
    hash_types::PubkeyHash,
    hashes::Hash,
    hashes::{sha512, HashEngine, Hmac, HmacEngine},
    secp256k1,
    secp256k1::SecretKey,
    util::bip32::ExtendedPrivKey,
    PrivateKey, Transaction, Txid,
};
use anyhow::Context;
use bitcoin::util::bip32::ChainCode;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::path::Path;
use url::Url;

#[derive(Debug, Clone)]
pub struct Wallet {
    /// The wallet is named `nectar_x` with `x` being the first 4 byte of the public key hash
    name: String,
    bitcoind_client: Client,
    seed: Seed,
    network: Network,
}

impl Wallet {
    pub async fn new(seed: Seed, url: Url, network: Network) -> anyhow::Result<Wallet> {
        let key = secp256k1::SecretKey::from_slice(&seed.bytes())?;

        let private_key = ::bitcoin::PrivateKey {
            compressed: true,
            network,
            key,
        };

        let bitcoind_client = Client::new(url);

        let name = Wallet::gen_name(private_key);

        let wallet = Wallet {
            name,
            bitcoind_client,
            seed,
            network,
        };

        wallet.init().await?;

        Ok(wallet)
    }

    async fn init(&self) -> anyhow::Result<()> {
        let info = self.info().await;

        // We assume the wallet present with the same name has the
        // same seed, which is fair but could be safer.
        if info.is_err() {
            // TODO: Probably need to protect the wallet with a passphrase
            self.bitcoind_client
                .create_wallet(&self.name, None, Some(true), None, None)
                .await?;

            let wif = self.seed_as_wif();

            self.bitcoind_client
                .set_hd_seed(&self.name, Some(true), Some(wif))
                .await?;
        }

        Ok(())
    }

    pub fn random_transient_sk(&self) -> anyhow::Result<SecretKey> {
        // TODO: Replace random bytes with SwapId or SharedSwapId?
        let mut random_bytes = [0u8; 32];

        rand::thread_rng().fill_bytes(&mut random_bytes);
        // TODO: use bitcoin_hashes instead of adding new dependency sha2
        let mut hash = Sha256::new();
        hash.update(random_bytes);

        let sk = hash.finalize();

        SecretKey::from_slice(&sk).context("failed to generate random transient key")
    }

    pub async fn info(&self) -> anyhow::Result<WalletInfoResponse> {
        self.assert_network(self.network).await?;

        self.bitcoind_client.get_wallet_info(&self.name).await
    }

    pub async fn new_address(&self) -> anyhow::Result<Address> {
        self.assert_network(self.network).await?;

        self.bitcoind_client
            .get_new_address(&self.name, None, Some("bech32".into()))
            .await
    }

    pub async fn balance(&self) -> anyhow::Result<Amount> {
        self.assert_network(self.network).await?;

        self.bitcoind_client
            .get_balance(&self.name, None, None, None)
            .await
    }

    /// Returns the seed in wif format, this allows the user to import the wallet in a
    /// different bitcoind using `sethdseed`.
    /// It seems relevant that access to bitcoind must not be needed to complete the task
    /// in case there is an issue with bitcoind and the user wants to regain control over their wallet
    /// Do note that the `wif` format is only here to allow the communication of `bytes`. The seed
    /// is NOT used as a private key in bitcoin. See `root_extended_private_key` to get the
    /// root private key of the bip32 hd wallet.
    // TODO: check the network against bitcoind in a non-failing manner (just log)
    pub fn seed_as_wif(&self) -> String {
        let key = self.seed.as_secret_key();

        let private_key = PrivateKey {
            compressed: true,
            network: self.network,
            key,
        };

        private_key.to_wif()
    }

    /// This seems to be the standard way to get a root extended private key from a seed
    /// This is the way bitcoind does it when being passed a seed with `sethdseed`
    // TODO: check the network against bitcoind in a non-failing manner (just log)
    pub fn root_extended_private_key(&self) -> ExtendedPrivKey {
        let bytes = self.seed.bytes();
        let hash_key = b"Bitcoin seed";

        let mut engine = HmacEngine::<sha512::Hash>::new(hash_key);
        engine.input(&bytes);
        let hash = Hmac::<sha512::Hash>::from_engine(engine);
        let output = &hash.into_inner()[..];
        let key = &output[..32];
        let chain_code = &output[32..];

        let key = SecretKey::from_slice(key).expect("32 bytes array should be fine");
        let private_key = PrivateKey {
            compressed: true,
            network: self.network,
            key,
        };

        let chain_code = ChainCode::from(chain_code);

        ExtendedPrivKey {
            network: self.network,
            depth: 0,
            parent_fingerprint: Default::default(),
            child_number: 0.into(),
            private_key,
            chain_code,
        }
    }

    pub async fn send_to_address(
        &self,
        address: Address,
        amount: Amount,
        network: Network,
    ) -> anyhow::Result<Txid> {
        self.assert_network(network).await?;

        let txid = self
            .bitcoind_client
            .send_to_address(&self.name, address, amount)
            .await?;
        Ok(txid)
    }

    pub async fn send_raw_transaction(
        &self,
        transaction: Transaction,
        network: Network,
    ) -> anyhow::Result<Txid> {
        self.assert_network(network).await?;

        let txid = self
            .bitcoind_client
            .send_raw_transaction(&self.name, transaction)
            .await?;
        Ok(txid)
    }

    pub async fn get_raw_transaction(&self, txid: Txid) -> anyhow::Result<Transaction> {
        self.assert_network(self.network).await?;

        let transaction = self
            .bitcoind_client
            .get_raw_transaction(&self.name, txid)
            .await?;

        Ok(transaction)
    }

    pub async fn dump(&self, filename: &Path) -> anyhow::Result<()> {
        self.bitcoind_client.dump_wallet(&self.name, filename).await
    }

    async fn assert_network(&self, expected: Network) -> anyhow::Result<()> {
        let actual = self.bitcoind_client.network().await?;

        if expected != actual {
            anyhow::bail!("Wrong network: expected {}, got {}", expected, actual);
        }

        Ok(())
    }

    // TODO: Just hash the seed instead of the public key of the seed (as a private key)
    fn gen_name(private_key: PrivateKey) -> String {
        let mut hash_engine = PubkeyHash::engine();
        private_key
            .public_key(&crate::SECP)
            .write_into(&mut hash_engine);
        let public_key_hash = PubkeyHash::from_engine(hash_engine);

        format!(
            "nectar_{:x}{:x}{:x}{:x}",
            public_key_hash[0], public_key_hash[1], public_key_hash[2], public_key_hash[3]
        )
    }
}

#[cfg(all(test, feature = "test-docker"))]
mod docker_tests {
    use super::*;
    use crate::test_harness::bitcoin;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    use tempdir::TempDir;
    use testcontainers::clients;

    #[tokio::test]
    async fn create_bitcoin_wallet_from_seed_and_get_address() {
        let tc_client = clients::Cli::default();
        let blockchain = bitcoin::Blockchain::new(&tc_client).unwrap();

        blockchain.init().await.unwrap();

        let seed = Seed::random().unwrap();
        let wallet = Wallet::new(seed, blockchain.node_url.clone(), Network::Regtest)
            .await
            .unwrap();

        let _address = wallet.new_address().await.unwrap();
    }

    #[tokio::test]
    async fn root_key_calculated_from_seed_is_the_same_than_bitcoind_s() {
        let tc_client = clients::Cli::default();
        let blockchain = bitcoin::Blockchain::new(&tc_client).unwrap();

        blockchain.init().await.unwrap();

        let seed = Seed::random().unwrap();
        let wallet = Wallet::new(seed, blockchain.node_url.clone(), Network::Regtest)
            .await
            .unwrap();

        let wif_path_docker = Path::new("/wallet.wif");

        let _ = wallet.dump(wif_path_docker).await.unwrap();

        // Wait for bitcoind to write the wif file
        std::thread::sleep(std::time::Duration::from_secs(3600));

        let tmp_dir = TempDir::new("nectar_test").unwrap();
        let path = tmp_dir.path().join("wallet.wif");

        Command::new("docker")
            .arg("cp")
            .arg(format!(
                "{}:{}",
                blockchain.container_id(),
                wif_path_docker.display()
            ))
            .arg(path.clone())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to execute docker cp")
            .wait()
            .expect("Failed to run docker cp");

        let wif = File::open(path).unwrap();
        let wif = BufReader::new(wif);

        // The line we are looking for looks like that:
        // # extended private masterkey: tprv...

        let line = wif
            .lines()
            .find(|line| {
                line.as_ref()
                    .map(|line| line.starts_with("# extended private masterkey: "))
                    .unwrap_or(false)
            })
            .unwrap()
            .unwrap();

        let key = line.split_ascii_whitespace().last().unwrap();
        assert_eq!(key, &wallet.root_extended_private_key().to_string());
    }

    #[tokio::test]
    async fn create_bitcoin_wallet_from_seed_and_get_balance() {
        let tc_client = clients::Cli::default();
        let blockchain = bitcoin::Blockchain::new(&tc_client).unwrap();

        blockchain.init().await.unwrap();

        let seed = Seed::random().unwrap();
        let wallet = Wallet::new(seed, blockchain.node_url.clone(), Network::Regtest)
            .await
            .unwrap();

        let _balance = wallet.balance().await.unwrap();
    }

    #[tokio::test]
    async fn create_bitcoin_wallet_when_already_existing_and_get_address() {
        let tc_client = clients::Cli::default();
        let blockchain = bitcoin::Blockchain::new(&tc_client).unwrap();

        blockchain.init().await.unwrap();

        let seed = Seed::random().unwrap();
        {
            let wallet = Wallet::new(seed, blockchain.node_url.clone(), Network::Regtest)
                .await
                .unwrap();

            let _address = wallet.new_address().await.unwrap();
        }

        {
            let wallet = Wallet::new(seed, blockchain.node_url.clone(), Network::Regtest)
                .await
                .unwrap();

            let _address = wallet.new_address().await.unwrap();
        }
    }
}
