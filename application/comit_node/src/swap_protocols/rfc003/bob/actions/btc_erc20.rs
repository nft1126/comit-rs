use crate::swap_protocols::{
    ledger::{Bitcoin, Ethereum},
    rfc003::{
        bitcoin,
        bob::{
            self,
            actions::{Accept, Decline},
            SwapCommunication,
        },
        ethereum::{self, Erc20Htlc},
        secret::Secret,
        secret_source::SecretSource,
        swap_accepted, Actions, LedgerState,
    },
};
use bitcoin_support::{BitcoinQuantity, OutPoint};
use bitcoin_witness::PrimedInput;
use ethereum_support::{Bytes, Erc20Token, EtherQuantity};
use std::sync::Arc;

type SwapAccepted = swap_accepted::SwapAccepted<Bitcoin, Ethereum, BitcoinQuantity, Erc20Token>;

fn deploy_action(swap_accepted: &SwapAccepted) -> ethereum::ContractDeploy {
    swap_accepted.beta_htlc_params().into()
}

pub fn fund_action(
    swap_accepted: &SwapAccepted,
    beta_htlc_location: ethereum_support::Address,
) -> ethereum::SendTransaction {
    let to = swap_accepted.request.beta_asset.token_contract();
    let htlc = Erc20Htlc::from(swap_accepted.beta_htlc_params());
    let gas_limit = Erc20Htlc::fund_tx_gas_limit();
    let network = swap_accepted.request.beta_ledger.network;

    ethereum::SendTransaction {
        to,
        data: htlc.funding_tx_payload(beta_htlc_location),
        gas_limit,
        amount: EtherQuantity::zero(),
        network,
    }
}

pub fn _refund_action(
    swap_accepted: &SwapAccepted,
    beta_htlc_location: ethereum_support::Address,
) -> ethereum::SendTransaction {
    let data = Bytes::default();
    let gas_limit = Erc20Htlc::tx_gas_limit();
    let network = swap_accepted.request.beta_ledger.network;

    ethereum::SendTransaction {
        to: beta_htlc_location,
        data,
        gas_limit,
        amount: EtherQuantity::zero(),
        network,
    }
}

pub fn redeem_action(
    swap_accepted: &SwapAccepted,
    alpha_htlc_location: OutPoint,
    secret_source: &dyn SecretSource,
    secret: Secret,
) -> bitcoin::SpendOutput {
    let alpha_asset = swap_accepted.request.alpha_asset;
    let htlc = bitcoin::Htlc::from(swap_accepted.alpha_htlc_params());
    let network = swap_accepted.request.alpha_ledger.network;

    bitcoin::SpendOutput {
        output: PrimedInput::new(
            alpha_htlc_location,
            alpha_asset,
            htlc.unlock_with_secret(secret_source.secp256k1_redeem(), &secret),
        ),
        network,
    }
}

impl Actions for bob::State<Bitcoin, Ethereum, BitcoinQuantity, Erc20Token> {
    type ActionKind = bob::ActionKind<
        Accept<Bitcoin, Ethereum>,
        Decline<Bitcoin, Ethereum>,
        ethereum::ContractDeploy,
        ethereum::SendTransaction,
        bitcoin::SpendOutput,
        ethereum::SendTransaction,
    >;

    fn actions(&self) -> Vec<Self::ActionKind> {
        let swap_accepted = match &self.swap_communication {
            SwapCommunication::Proposed {
                pending_response, ..
            } => {
                return vec![
                    bob::ActionKind::Accept(Accept::new(
                        pending_response.sender.clone(),
                        Arc::clone(&self.secret_source),
                    )),
                    bob::ActionKind::Decline(Decline::new(pending_response.sender.clone())),
                ];
            }
            SwapCommunication::Accepted { ref swap_accepted } => swap_accepted,
            SwapCommunication::Rejected { .. } => return vec![],
        };

        let alpha_state = &self.alpha_ledger_state;
        let beta_state = &self.beta_ledger_state;

        use self::LedgerState::*;
        match (alpha_state, beta_state, self.secret) {
            (Funded { htlc_location, .. }, _, Some(secret)) => {
                vec![bob::ActionKind::Redeem(redeem_action(
                    &swap_accepted,
                    *htlc_location,
                    self.secret_source.as_ref(),
                    secret,
                ))]
            }
            (Funded { .. }, NotDeployed, _) => {
                vec![bob::ActionKind::Deploy(deploy_action(&swap_accepted))]
            }
            (Funded { .. }, Deployed { htlc_location, .. }, _) => vec![bob::ActionKind::Fund(
                fund_action(&swap_accepted, *htlc_location),
            )],
            _ => vec![],
        }
    }
}
