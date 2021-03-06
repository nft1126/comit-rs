#![allow(dead_code)]

//! This module defines custom proptest strategies that make it easy to write
//! property tests for our domain.
//!
//! The functions and modules in this file are organized in the same hierarchy
//! as the types they produce. For example, the strategy for
//! `crate::identity::Bitcoin` is defined at
//! `crate::proptest::identity::bitcoin()`.

use crate::{ethereum::ChainId, LocalSwapId, Role, Side};
pub use proptest::prelude::*;
use uuid::Uuid;

pub fn role() -> impl Strategy<Value = Role> {
    prop_oneof![Just(Role::Alice), Just(Role::Bob)]
}

pub fn side() -> impl Strategy<Value = Side> {
    prop_oneof![Just(Side::Alpha), Just(Side::Beta)]
}

prop_compose! {
    pub fn timestamp()(
        secs in any::<u32>(),
    ) -> time::OffsetDateTime {
        time::OffsetDateTime::from_unix_timestamp(secs as i64)
    }
}

pub fn local_swap_id() -> impl Strategy<Value = LocalSwapId> {
    prop::num::u128::ANY.prop_map(|v| LocalSwapId::from(Uuid::from_u128(v)))
}

pub fn chain_id() -> impl Strategy<Value = ChainId> {
    prop::num::u32::ANY.prop_map(ChainId::from)
}

pub mod libp2p {
    use super::*;
    use ::libp2p::{
        core::PublicKey,
        identity::secp256k1::{Keypair, SecretKey},
        Multiaddr, PeerId,
    };
    use std::net::Ipv4Addr;

    pub fn peer_id() -> impl Strategy<Value = PeerId> {
        prop::array::uniform32(1u8..)
            .prop_map(|bytes| {
                SecretKey::from_bytes(bytes).expect("any 32 bytes are a valid secret key")
            })
            .prop_map(|sk| PublicKey::Secp256k1(Keypair::from(sk).public().clone()))
            .prop_map(PeerId::from_public_key)
    }

    prop_compose! {
        // we just generate a random ipv4 multiaddress, there are a lot more combinations but for our purposes, this is fine
        pub fn multiaddr()(
            a in any::<u8>(),
            b in any::<u8>(),
            c in any::<u8>(),
            d in any::<u8>(),
        ) -> Multiaddr {
            Ipv4Addr::new(a, b, c, d).into()
        }
    }
}

pub mod identity {
    use super::*;
    use comit::identity;

    pub fn ethereum() -> impl Strategy<Value = identity::Ethereum> {
        prop::array::uniform20(1u8..).prop_map(identity::Ethereum::from)
    }

    pub fn bitcoin() -> impl Strategy<Value = identity::Bitcoin> {
        prop::array::uniform32(1u8..)
            .prop_map(|bytes| {
                ::bitcoin::secp256k1::SecretKey::from_slice(&bytes)
                    .expect("any 32 bytes are a valid secret key")
            })
            .prop_map(|sk| identity::Bitcoin::from_secret_key(&&crate::SECP, &sk))
    }
}

pub mod ethereum {
    use super::*;
    use comit::asset::{ethereum::FromWei, Erc20Quantity};

    pub fn erc20_quantity() -> impl Strategy<Value = Erc20Quantity> {
        prop::num::u128::ANY.prop_map(Erc20Quantity::from_wei)
    }
}

pub mod bitcoin {
    use super::*;

    prop_compose! {
        pub fn address()(
            public_key in identity::bitcoin(),
            network in ledger::bitcoin(),
        ) -> ::bitcoin::Address {
            ::bitcoin::Address::p2wpkh(&public_key.into(), network.into()).expect("our public keys are always compressed")
        }
    }
}

pub mod ledger {
    use super::*;
    use comit::ledger;

    pub fn bitcoin() -> impl Strategy<Value = ledger::Bitcoin> {
        prop_oneof![
            Just(ledger::Bitcoin::Mainnet),
            Just(ledger::Bitcoin::Testnet),
            Just(ledger::Bitcoin::Regtest)
        ]
    }

    prop_compose! {
        pub fn ethereum()(
            chain_id in any::<u32>()
        ) -> ledger::Ethereum {
            chain_id.into()
        }
    }
}

pub mod asset {
    use super::*;
    use comit::asset;

    pub fn bitcoin() -> impl Strategy<Value = asset::Bitcoin> {
        prop::num::u64::ANY.prop_map(asset::Bitcoin::from_sat)
    }

    prop_compose! {
        pub fn erc20()(
            quantity in ethereum::erc20_quantity(),
            token_contract in identity::ethereum()
        ) -> asset::Erc20 {
            asset::Erc20::new(token_contract, quantity)
        }
    }
}
