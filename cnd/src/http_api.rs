mod action;
mod dial_addr;
mod info;
mod markets;
mod orders;
mod peers;
mod problem;
mod route_factory;
mod serde_peer_id;
mod swaps;
mod tokens;

pub use self::{problem::*, route_factory::create as create_routes, swaps::SwapResource};

pub const PATH: &str = "swaps";

use crate::{
    asset,
    asset::Erc20Quantity,
    ethereum,
    storage::{BtcDaiOrder, Order},
};
use anyhow::Result;
use comit::{swap::Action, OrderId, Position, Price, Quantity};
use serde::Serialize;
use warp::http::Method;

/// The struct representing the properties within the siren document in our
/// response.
#[derive(Serialize)]
struct OrderProperties {
    id: OrderId,
    position: Position,
    price: Amount,
    quantity: Amount,
    state: State,
}

impl From<(Order, BtcDaiOrder)> for OrderProperties {
    fn from(tuple: (Order, BtcDaiOrder)) -> Self {
        let (order, btc_dai_order) = tuple;

        Self {
            id: order.order_id,
            position: order.position,
            price: Amount::from(btc_dai_order.price),
            quantity: Amount::from(btc_dai_order.quantity),
            state: State {
                open: btc_dai_order.open.to_inner(),
                closed: btc_dai_order.closed.to_inner(),
                settling: btc_dai_order.settling.to_inner(),
                failed: btc_dai_order.failed.to_inner(),
                cancelled: btc_dai_order.cancelled.to_inner(),
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(tag = "currency")]
pub enum Amount {
    #[serde(rename = "BTC")]
    Bitcoin {
        #[serde(with = "asset::bitcoin::sats_as_string")]
        value: asset::Bitcoin,
        decimals: u8,
    },
    #[serde(rename = "DAI")]
    Dai { value: Erc20Quantity, decimals: u8 },
}

impl From<Quantity<asset::Bitcoin>> for Amount {
    fn from(quantity: Quantity<asset::Bitcoin>) -> Self {
        Amount::btc(quantity.to_inner())
    }
}

impl From<Price<asset::Bitcoin, Erc20Quantity>> for Amount {
    fn from(price: Price<asset::Bitcoin, Erc20Quantity>) -> Self {
        Amount::dai(price.wei_per_btc())
    }
}

impl Amount {
    fn btc(value: asset::Bitcoin) -> Self {
        Amount::Bitcoin { value, decimals: 8 }
    }

    fn dai(value: Erc20Quantity) -> Self {
        Amount::Dai {
            value,
            decimals: 18,
        }
    }
}

#[derive(Serialize)]
struct State {
    #[serde(with = "asset::bitcoin::sats_as_string")]
    open: asset::Bitcoin,
    #[serde(with = "asset::bitcoin::sats_as_string")]
    closed: asset::Bitcoin,
    #[serde(with = "asset::bitcoin::sats_as_string")]
    settling: asset::Bitcoin,
    #[serde(with = "asset::bitcoin::sats_as_string")]
    failed: asset::Bitcoin,
    #[serde(with = "asset::bitcoin::sats_as_string")]
    cancelled: asset::Bitcoin,
}

impl State {
    pub fn is_open(&self) -> bool {
        self.open != asset::Bitcoin::ZERO
    }
}

fn make_order_entity(properties: OrderProperties) -> Result<siren::Entity> {
    let mut entity = siren::Entity::default().with_properties(&properties)?;

    if let Some(action) = cancel_action(&properties) {
        entity = entity.with_action(action)
    }

    Ok(entity)
}

fn cancel_action(order: &OrderProperties) -> Option<siren::Action> {
    if order.state.is_open() {
        Some(siren::Action {
            name: "cancel".to_string(),
            class: vec![],
            method: Some(Method::DELETE),
            href: format!("/orders/{}", order.id),
            title: None,
            _type: Some("application/json".to_owned()),
            fields: vec![],
        })
    } else {
        None
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase", tag = "protocol")]
pub enum Protocol {
    Hbit { asset: Amount },
    Herc20 { asset: Amount },
}

impl Protocol {
    pub fn hbit(btc: asset::Bitcoin) -> Self {
        Protocol::Hbit {
            asset: Amount::btc(btc),
        }
    }

    pub fn herc20_dai(dai: Erc20Quantity) -> Self {
        Protocol::Herc20 {
            asset: Amount::dai(dai),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ActionName {
    Deploy,
    Fund,
    Redeem,
}

impl From<Action> for ActionName {
    fn from(action: Action) -> Self {
        match action {
            Action::Herc20Deploy(_) => ActionName::Deploy,
            Action::Herc20Fund(..) => ActionName::Fund,
            Action::Herc20Redeem(..) => ActionName::Redeem,
            Action::HbitFund(_) => ActionName::Fund,
            Action::HbitRedeem(..) => ActionName::Redeem,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(tag = "name", rename_all = "snake_case")]
pub enum SwapEvent {
    HbitFunded { tx: bitcoin::Txid },
    HbitRedeemed { tx: bitcoin::Txid },
    Herc20Deployed { tx: ethereum::Hash },
    Herc20Funded { tx: ethereum::Hash },
    Herc20Redeemed { tx: ethereum::Hash },
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("action not found")]
pub struct ActionNotFound;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        asset,
        asset::{ethereum::FromWei, Bitcoin},
    };
    use uuid::Uuid;

    #[test]
    fn response_serializes_correctly() {
        let properties = OrderProperties {
            id: OrderId::from(Uuid::from_u128(0)),
            position: Position::Sell,
            price: Amount::dai(Erc20Quantity::from_wei_dec_str("9100000000000000000000").unwrap()),
            quantity: Amount::btc(Bitcoin::from_sat(10000000)),
            state: State {
                open: Bitcoin::from_sat(3000000),
                closed: Bitcoin::from_sat(1000000),
                settling: Bitcoin::from_sat(0),
                failed: Bitcoin::from_sat(6000000),
                cancelled: Bitcoin::from_sat(0),
            },
        };

        let result = serde_json::to_string_pretty(&properties).unwrap();

        assert_eq!(
            result,
            r#"{
  "id": "00000000-0000-0000-0000-000000000000",
  "position": "sell",
  "price": {
    "currency": "DAI",
    "value": "9100000000000000000000",
    "decimals": 18
  },
  "quantity": {
    "currency": "BTC",
    "value": "10000000",
    "decimals": 8
  },
  "state": {
    "open": "3000000",
    "closed": "1000000",
    "settling": "0",
    "failed": "6000000",
    "cancelled": "0"
  }
}"#
        );
    }

    #[test]
    fn btc_amount_serializes_properly() {
        let amount = Amount::btc(asset::Bitcoin::from_sat(100000000));

        let string = serde_json::to_string(&amount).unwrap();

        assert_eq!(
            string,
            r#"{"currency":"BTC","value":"100000000","decimals":8}"#
        )
    }

    #[test]
    fn dai_amount_serializes_properly() {
        let amount =
            Amount::dai(Erc20Quantity::from_wei_dec_str("9000000000000000000000").unwrap());

        let string = serde_json::to_string(&amount).unwrap();

        assert_eq!(
            string,
            r#"{"currency":"DAI","value":"9000000000000000000000","decimals":18}"#
        )
    }

    #[test]
    fn hbit_protocol_serializes_correctly() {
        let protocol = Protocol::hbit(asset::Bitcoin::from_sat(10_000));

        let result = serde_json::to_string_pretty(&protocol).unwrap();

        assert_eq!(
            result,
            r#"{
  "protocol": "hbit",
  "asset": {
    "currency": "BTC",
    "value": "10000",
    "decimals": 8
  }
}"#
        )
    }

    #[test]
    fn herc20_protocol_serializes_correctly() {
        let protocol = Protocol::herc20_dai(Erc20Quantity::from_wei(1_000_000_000_000_000u64));

        let result = serde_json::to_string_pretty(&protocol).unwrap();

        assert_eq!(
            result,
            r#"{
  "protocol": "herc20",
  "asset": {
    "currency": "DAI",
    "value": "1000000000000000",
    "decimals": 18
  }
}"#
        )
    }
}
