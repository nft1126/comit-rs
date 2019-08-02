mod behaviour;
mod handler;
mod protocol;
mod substream;

pub use self::{
    behaviour::{BamBehaviour, BehaviourOutEvent},
    handler::{BamHandler, PendingInboundRequest, PendingOutboundRequest},
    protocol::{BamProtocol, BamStream},
};
use crate::libp2p_bam::handler::ProtocolOutEvent;
use libp2p::core::protocols_handler::ProtocolsHandlerEvent;

pub type BamHandlerEvent =
    ProtocolsHandlerEvent<BamProtocol, PendingOutboundRequest, ProtocolOutEvent>;
