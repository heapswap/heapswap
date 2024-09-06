use std::future::{ready, Ready};

use libp2p::core::{InboundUpgrade, OutboundUpgrade, UpgradeInfo};
use libp2p::swarm::{Stream, StreamProtocol};

pub struct SubfieldUpgrade {
    pub(crate) supported_protocols: Vec<StreamProtocol>,
}

impl UpgradeInfo for SubfieldUpgrade {
    type Info = StreamProtocol;

    type InfoIter = std::vec::IntoIter<StreamProtocol>;

    fn protocol_info(&self) -> Self::InfoIter {
        self.supported_protocols.clone().into_iter()
    }
}

impl InboundUpgrade<Stream> for SubfieldUpgrade {
    type Output = (Stream, StreamProtocol);

    type Error = void::Void;

    type Future = Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, socket: Stream, info: Self::Info) -> Self::Future {
        ready(Ok((socket, info)))
    }
}

impl OutboundUpgrade<Stream> for SubfieldUpgrade {
    type Output = (Stream, StreamProtocol);

    type Error = void::Void;

    type Future = Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_outbound(self, socket: Stream, info: Self::Info) -> Self::Future {
        ready(Ok((socket, info)))
    }
}
