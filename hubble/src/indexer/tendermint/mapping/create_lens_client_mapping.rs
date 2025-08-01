use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{create_lens_client_event::CreateLensClientEvent, supported::SupportedBlockEvent},
    tendermint::{fetcher_client::TmFetcherClient, mapping::decoder::Decoder},
};

impl TmFetcherClient {
    pub fn to_create_lens_client(
        &self,
        log: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_create_lens_client - {log}");

        Ok(vec![SupportedBlockEvent::CreateLensClient {
            inner: CreateLensClientEvent {
                header: log.header()?,
                client_id: log.event.client_id()?,
                l1_client_id: log.event.l1_client_id()?,
                l2_client_id: log.event.l2_client_id()?,
                l2_chain_id: log.event.l2_chain_id()?,
            },
        }])
    }
}
