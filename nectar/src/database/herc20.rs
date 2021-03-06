use crate::{
    database::{Database, Load, Save},
    swap::herc20,
    SwapId,
};
use comit::{asset::Erc20, ethereum, identity, Secret, SecretHash, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Herc20Deployed {
    pub transaction: ethereum::Hash,
    pub location: comit::htlc_location::Ethereum,
}

impl From<Herc20Deployed> for herc20::Deployed {
    fn from(event: Herc20Deployed) -> Self {
        herc20::Deployed {
            transaction: event.transaction,
            location: event.location,
        }
    }
}

impl From<herc20::Deployed> for Herc20Deployed {
    fn from(event: herc20::Deployed) -> Self {
        Herc20Deployed {
            transaction: event.transaction,
            location: event.location,
        }
    }
}

#[async_trait::async_trait]
impl Save<herc20::Deployed> for Database {
    async fn save(&self, event: herc20::Deployed, swap_id: SwapId) -> anyhow::Result<()> {
        self.update_swap(&swap_id, |mut old_swap| match &old_swap.herc20_deployed {
            Some(_) => anyhow::bail!("Herc20 Deployed event is already stored"),
            None => {
                old_swap.herc20_deployed = Some(event.into());
                Ok(old_swap)
            }
        })
        .await
    }
}

impl Load<herc20::Deployed> for Database {
    fn load(&self, swap_id: SwapId) -> anyhow::Result<Option<herc20::Deployed>> {
        let swap = self.get_swap_or_bail(&swap_id)?;

        Ok(swap.herc20_deployed.map(Into::into))
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Herc20Funded {
    pub transaction: ethereum::Hash,
}

impl From<Herc20Funded> for herc20::Funded {
    fn from(event: Herc20Funded) -> Self {
        herc20::Funded {
            transaction: event.transaction,
        }
    }
}

impl From<herc20::Funded> for Herc20Funded {
    fn from(event: herc20::Funded) -> Self {
        Herc20Funded {
            transaction: event.transaction,
        }
    }
}

#[async_trait::async_trait]
impl Save<herc20::Funded> for Database {
    async fn save(&self, event: herc20::Funded, swap_id: SwapId) -> anyhow::Result<()> {
        self.update_swap(&swap_id, |mut old_swap| match &old_swap.herc20_funded {
            Some(_) => anyhow::bail!("Herc20 Funded event is already stored"),
            None => {
                old_swap.herc20_funded = Some(event.into());
                Ok(old_swap)
            }
        })
        .await
    }
}

impl Load<herc20::Funded> for Database {
    fn load(&self, swap_id: SwapId) -> anyhow::Result<Option<herc20::Funded>> {
        let swap = self.get_swap_or_bail(&swap_id)?;

        Ok(swap.herc20_funded.map(Into::into))
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Herc20Redeemed {
    pub transaction: ethereum::Hash,
    pub secret: Secret,
}

impl From<Herc20Redeemed> for herc20::Redeemed {
    fn from(event: Herc20Redeemed) -> Self {
        herc20::Redeemed {
            transaction: event.transaction,
            secret: event.secret,
        }
    }
}

impl From<herc20::Redeemed> for Herc20Redeemed {
    fn from(event: herc20::Redeemed) -> Self {
        Herc20Redeemed {
            transaction: event.transaction,
            secret: event.secret,
        }
    }
}

#[async_trait::async_trait]
impl Save<herc20::Redeemed> for Database {
    async fn save(&self, event: herc20::Redeemed, swap_id: SwapId) -> anyhow::Result<()> {
        self.update_swap(&swap_id, |mut old_swap| match &old_swap.herc20_redeemed {
            Some(_) => anyhow::bail!("Herc20 Redeem event is already stored"),
            None => {
                old_swap.herc20_redeemed = Some(event.into());
                Ok(old_swap)
            }
        })
        .await
    }
}

impl Load<herc20::Redeemed> for Database {
    fn load(&self, swap_id: SwapId) -> anyhow::Result<Option<herc20::Redeemed>> {
        let swap = self.get_swap_or_bail(&swap_id)?;

        Ok(swap.herc20_redeemed.map(Into::into))
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Herc20Refunded {
    pub transaction: ethereum::Hash,
}

impl From<Herc20Refunded> for herc20::Refunded {
    fn from(event: Herc20Refunded) -> Self {
        herc20::Refunded {
            transaction: event.transaction,
        }
    }
}

impl From<herc20::Refunded> for Herc20Refunded {
    fn from(event: herc20::Refunded) -> Self {
        Herc20Refunded {
            transaction: event.transaction,
        }
    }
}

#[async_trait::async_trait]
impl Save<herc20::Refunded> for Database {
    async fn save(&self, event: herc20::Refunded, swap_id: SwapId) -> anyhow::Result<()> {
        self.update_swap(&swap_id, |mut old_swap| match &old_swap.herc20_refunded {
            Some(_) => anyhow::bail!("Herc20 Refunded event is already stored"),
            None => {
                old_swap.herc20_refunded = Some(event.into());
                Ok(old_swap)
            }
        })
        .await
    }
}

impl Load<herc20::Refunded> for Database {
    fn load(&self, swap_id: SwapId) -> anyhow::Result<Option<herc20::Refunded>> {
        let swap = self.get_swap_or_bail(&swap_id)?;

        Ok(swap.herc20_refunded.map(Into::into))
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Erc20Asset {
    pub token_contract: ethereum::Address,
    pub quantity: comit::asset::Erc20Quantity,
}

impl From<Erc20Asset> for comit::asset::Erc20 {
    fn from(asset: Erc20Asset) -> Self {
        comit::asset::Erc20 {
            token_contract: asset.token_contract,
            quantity: asset.quantity,
        }
    }
}

impl From<comit::asset::Erc20> for Erc20Asset {
    fn from(asset: Erc20) -> Self {
        Erc20Asset {
            token_contract: asset.token_contract,
            quantity: asset.quantity,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Params {
    pub asset: Erc20Asset,
    pub redeem_identity: identity::Ethereum,
    pub refund_identity: identity::Ethereum,
    pub expiry: Timestamp,
    pub secret_hash: SecretHash,
    pub chain_id: ethereum::ChainId,
}

impl From<Params> for comit::herc20::Params {
    fn from(params: Params) -> Self {
        let Params {
            asset,
            redeem_identity,
            refund_identity,
            expiry,
            secret_hash,
            chain_id,
        } = params;

        comit::herc20::Params {
            asset: asset.into(),
            redeem_identity,
            refund_identity,
            expiry,
            secret_hash,
            chain_id,
        }
    }
}

impl From<comit::herc20::Params> for Params {
    fn from(params: comit::herc20::Params) -> Self {
        Params {
            asset: params.asset.into(),
            redeem_identity: params.redeem_identity,
            refund_identity: params.refund_identity,
            expiry: params.expiry,
            secret_hash: params.secret_hash,
            chain_id: params.chain_id,
        }
    }
}

#[cfg(test)]
impl crate::StaticStub for Params {
    fn static_stub() -> Self {
        Params {
            asset: Erc20Asset {
                token_contract: Default::default(),
                quantity: comit::asset::Erc20Quantity::from_wei_dec_str(
                    "34_000_000_000_000_000_000",
                )
                .unwrap(),
            },
            redeem_identity: Default::default(),
            refund_identity: Default::default(),
            expiry: 12345689.into(),
            secret_hash: SecretHash::new(Secret::from(*b"hello world, you are beautiful!!")),
            chain_id: comit::ethereum::ChainId::GETH_DEV,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{database::Swap, swap::SwapKind, StaticStub};

    #[tokio::test]
    async fn save_and_load_herc20_deployed() {
        let db = Database::new_test().unwrap();
        let swap = Swap::static_stub();
        let swap_id = SwapId::default();
        let transaction = comit::transaction::Ethereum::default().hash;
        let location = comit::htlc_location::Ethereum::random();

        let swap_kind = SwapKind::from((swap, swap_id));

        db.insert_swap(swap_kind).await.unwrap();

        let event = herc20::Deployed {
            transaction,
            location,
        };
        db.save(event, swap_id).await.unwrap();

        let stored_event: herc20::Deployed = db
            .load(swap_id)
            .expect("No error loading")
            .expect("found the event");

        assert_eq!(stored_event.transaction, transaction);
        assert_eq!(stored_event.location, location);
    }

    #[tokio::test]
    async fn save_and_load_herc20_funded() {
        let db = Database::new_test().unwrap();
        let swap = Swap::static_stub();
        let swap_id = SwapId::default();
        let transaction = comit::transaction::Ethereum::default().hash;

        let swap_kind = SwapKind::from((swap, swap_id));

        db.insert_swap(swap_kind).await.unwrap();

        let event = herc20::Funded { transaction };
        db.save(event, swap_id).await.unwrap();

        let stored_event: herc20::Funded = db
            .load(swap_id)
            .expect("No error loading")
            .expect("found the event");

        assert_eq!(stored_event.transaction, transaction);
    }

    #[tokio::test]
    async fn save_and_load_herc20_redeemed() {
        let db = Database::new_test().unwrap();
        let swap = Swap::static_stub();
        let swap_id = SwapId::default();
        let transaction = comit::transaction::Ethereum::default().hash;
        let secret = Secret::from_vec(b"are those thirty-two bytes? Hum.").unwrap();

        let swap_kind = SwapKind::from((swap, swap_id));

        db.insert_swap(swap_kind).await.unwrap();

        let event = herc20::Redeemed {
            transaction,
            secret,
        };
        db.save(event, swap_id).await.unwrap();

        let stored_event: herc20::Redeemed = db
            .load(swap_id)
            .expect("No error loading")
            .expect("found the event");

        assert_eq!(stored_event.transaction, transaction);
        assert_eq!(stored_event.secret, secret);
    }

    #[tokio::test]
    async fn save_and_load_herc20_refunded() {
        let db = Database::new_test().unwrap();
        let swap = Swap::static_stub();
        let swap_id = SwapId::default();
        let transaction = comit::transaction::Ethereum::default().hash;

        let swap_kind = SwapKind::from((swap, swap_id));

        db.insert_swap(swap_kind).await.unwrap();

        let event = herc20::Refunded { transaction };
        db.save(event, swap_id).await.unwrap();

        let stored_event: herc20::Refunded = db
            .load(swap_id)
            .expect("No error loading")
            .expect("found the event");

        assert_eq!(stored_event.transaction, transaction);
    }
}
