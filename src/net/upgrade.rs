use futures::future::Future;

pub const PROTOCOL_UNSUPPORTED: &str = "na";

pub trait ProtocolId {
    fn protocol_id() -> &'static str;
}

pub trait UpgradeOutbound<'a, T> {
    type Output;
    type Error;
    type Future: Future<Output = Result<Self::Output, Self::Error>> + 'a;

    fn upgrade_outbound(self, stream: T) -> Self::Future;
}
