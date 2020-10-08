//! Code that could be upstreamed to COMIT lib.

pub mod hbit;
pub mod hbit_herc20;
pub mod herc20;
pub mod herc20_hbit;

pub use comit::{ethereum, *};
pub use hbit_herc20::{hbit_herc20_alice, hbit_herc20_bob};
pub use herc20_hbit::{herc20_hbit_alice, herc20_hbit_bob};

use std::error::Error as StdError;
use thiserror::Error;

#[derive(Debug)]
pub enum Action {
    Herc20Deploy(herc20::Params),
    Herc20Fund(herc20::Params, herc20::Deployed),
    Herc20Redeem(herc20::Params, herc20::Deployed, Secret),
    HbitFund(hbit::Params),
    HbitRedeem(hbit::Params, hbit::Funded, Secret),
}

#[derive(Debug, Clone, Copy, Error)]
pub enum Error<A, B>
where
    A: StdError + 'static,
    B: StdError + 'static,
{
    #[error("alpha ledger was incorrectly funded")]
    AlphaIncorrectlyFunded(#[source] A),
    #[error("beta ledger was incorrectly funded")]
    BetaIncorrectlyFunded(#[source] B),
}
