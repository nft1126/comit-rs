pub mod alice;
pub mod bob;

use crate::{
    halbit, herc20,
    http_api::{problem, Halbit, Herc20, PostBody},
    network::{swap_digest, Identities},
    storage::Save,
    Facade, LocalSwapId,
};
use serde::Deserialize;
use warp::{http::StatusCode, Rejection, Reply};

pub async fn post_swap(body: serde_json::Value, facade: Facade) -> Result<impl Reply, Rejection> {
    let body = PostBody::<Herc20, Halbit>::deserialize(&body)
        .map_err(anyhow::Error::new)
        .map_err(problem::from_anyhow)
        .map_err(warp::reject::custom)?;

    let swap_id = LocalSwapId::default();
    let reply = warp::reply::reply();

    let swap = body.to_created_swap::<herc20::CreatedSwap, halbit::CreatedSwap>(swap_id);
    facade
        .save(swap)
        .await
        .map_err(problem::from_anyhow)
        .map_err(warp::reject::custom)?;

    let identities = Identities {
        ethereum_identity: Some(body.alpha.identity),
        lightning_identity: Some(body.beta.identity),
        bitcoin_identity: None,
    };
    let digest = swap_digest::herc20_halbit(body.clone());
    let peer = body.peer.into();
    let role = body.role.0;

    facade
        .initiate_communication(swap_id, peer, role, digest, identities)
        .await
        .map(|_| {
            warp::reply::with_status(
                warp::reply::with_header(reply, "Location", format!("/swaps/{}", swap_id)),
                StatusCode::CREATED,
            )
        })
        .map_err(problem::from_anyhow)
        .map_err(warp::reject::custom)
}

impl From<PostBody<Herc20, Halbit>> for swap_digest::Herc20Halbit {
    fn from(body: PostBody<Herc20, Halbit>) -> Self {
        Self {
            ethereum_absolute_expiry: body.alpha.absolute_expiry.into(),
            erc20_amount: body.alpha.amount,
            token_contract: body.alpha.token_contract,
            lightning_cltv_expiry: body.beta.cltv_expiry.into(),
            lightning_amount: body.beta.amount.0,
        }
    }
}
