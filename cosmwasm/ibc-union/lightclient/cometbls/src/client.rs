use std::marker::PhantomData;

use cometbls_light_client_types::{
    client_state::ClientState, consensus_state::ConsensusState, header::Header,
    misbehaviour::Misbehaviour,
};
use cosmwasm_std::{Addr, Empty};
use ibc_union_light_client::{
    ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate,
};
use ibc_union_spec::{path::IBC_UNION_COSMWASM_COMMITMENT_PREFIX, Duration, Status, Timestamp};
use ics23::ibc_api::SDK_SPECS;
use unionlabs::{
    encoding::Bincode,
    ibc::core::{
        client::height::Height,
        commitment::{merkle_proof::MerkleProof, merkle_root::MerkleRoot},
    },
};

use crate::{
    errors::{Error, InvalidHeaderError},
    zkp_verifier::ZkpVerifier,
};

pub const WASMD_MODULE_STORE_KEY: &[u8] = b"wasm";
pub const WASMD_CONTRACT_STORE_PREFIX: u8 = 0x03;

pub struct CometblsLightClient<T: ZkpVerifier = ()>(PhantomData<T>);

impl<T: ZkpVerifier> IbcClient for CometblsLightClient<T> {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    type Misbehaviour = Misbehaviour;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type StorageProof = MerkleProof;

    type Encoding = Bincode;

    fn verify_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(height)?;
        Ok(ics23::ibc_api::verify_membership(
            &storage_proof,
            &SDK_SPECS,
            &consensus_state.app_hash,
            &[
                WASMD_MODULE_STORE_KEY.into(),
                [WASMD_CONTRACT_STORE_PREFIX]
                    .into_iter()
                    .chain(client_state.contract_address)
                    .chain(IBC_UNION_COSMWASM_COMMITMENT_PREFIX)
                    .chain(key)
                    .collect::<Vec<_>>(),
            ],
            value,
        )
        .map_err(Into::<Error>::into)?)
    }

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(height)?;
        Ok(ics23::ibc_api::verify_non_membership(
            &storage_proof,
            &SDK_SPECS,
            &consensus_state.app_hash,
            &[
                WASMD_MODULE_STORE_KEY.into(),
                [WASMD_CONTRACT_STORE_PREFIX]
                    .into_iter()
                    .chain(client_state.contract_address)
                    .chain(IBC_UNION_COSMWASM_COMMITMENT_PREFIX)
                    .chain(key)
                    .collect::<Vec<_>>(),
            ],
        )
        .map_err(Into::<Error>::into)?)
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> Timestamp {
        consensus_state.timestamp
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        client_state.latest_height.height()
    }

    fn get_counterparty_chain_id(client_state: &Self::ClientState) -> String {
        client_state.chain_id.clone().into_string()
    }

    fn status(ctx: IbcClientCtx<Self>, client_state: &Self::ClientState) -> Status {
        if client_state.frozen_height.height() != 0 {
            Status::Frozen
        } else {
            let Ok(consensus_state) =
                ctx.read_self_consensus_state(client_state.latest_height.height())
            else {
                return Status::Expired;
            };

            if is_client_expired(
                consensus_state.timestamp,
                client_state.trusting_period,
                Timestamp::from_nanos(ctx.env.block.time.nanos()),
            ) {
                return Status::Expired;
            }

            Status::Active
        }
    }

    fn verify_creation(
        _caller: Addr,
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
        _relayer: Addr,
    ) -> Result<ClientCreationResult<Self>, ibc_union_light_client::IbcClientError<Self>> {
        Ok(ClientCreationResult::new())
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        _caller: Addr,
        header: Self::Header,
        _relayer: Addr,
    ) -> Result<StateUpdate<Self>, ibc_union_light_client::IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(header.trusted_height.height())?;

        // If the update is already happened, we just do noop
        let header_height = header.signed_header.height.inner() as u64;
        if let Ok(consensus_state) = ctx.read_self_consensus_state(header_height) {
            return Ok(StateUpdate::new(header_height, consensus_state));
        }

        verify_header::<T>(&ctx, &client_state, &consensus_state, &header)?;

        update_state(client_state, consensus_state, header).map_err(Into::into)
    }

    fn misbehaviour(
        ctx: IbcClientCtx<Self>,
        _caller: Addr,
        misbehaviour: Self::Misbehaviour,
        _relayer: Addr,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        let mut client_state = ctx.read_self_client_state()?;

        verify_misbehaviour(
            &ctx,
            &client_state,
            &ctx.read_self_consensus_state(misbehaviour.header_a.trusted_height.height())?,
            &ctx.read_self_consensus_state(misbehaviour.header_b.trusted_height.height())?,
            misbehaviour,
        )?;

        client_state.frozen_height = Height::new(1);

        Ok(client_state)
    }
}

fn verify_header<T: ZkpVerifier>(
    ctx: &IbcClientCtx<CometblsLightClient<T>>,
    client_state: &ClientState,
    consensus_state: &ConsensusState,
    header: &Header,
) -> Result<(), Error> {
    // SAFETY: height is bound to be 0..i64::MAX which makes it within the bounds of u64
    let untrusted_height_number = header.signed_header.height.inner() as u64;
    let trusted_height_number = header.trusted_height.height();

    if untrusted_height_number <= trusted_height_number {
        return Err(InvalidHeaderError::SignedHeaderHeightMustBeMoreRecent {
            signed_height: untrusted_height_number,
            trusted_height: trusted_height_number,
        }
        .into());
    }

    let trusted_timestamp = consensus_state.timestamp;
    // Normalized to nanoseconds to follow tendermint convention
    let untrusted_timestamp = Timestamp::from_nanos(header.signed_header.time.as_unix_nanos());

    if untrusted_timestamp <= trusted_timestamp {
        return Err(InvalidHeaderError::SignedHeaderTimestampMustBeMoreRecent {
            signed_timestamp: untrusted_timestamp,
            trusted_timestamp,
        }
        .into());
    }

    if is_client_expired(
        trusted_timestamp,
        client_state.trusting_period,
        Timestamp::from_nanos(ctx.env.block.time.nanos()),
    ) {
        return Err(InvalidHeaderError::HeaderExpired(consensus_state.timestamp).into());
    }

    let max_clock_drift_timestamp = Timestamp::from_nanos(ctx.env.block.time.nanos())
        .plus_duration(client_state.max_clock_drift)
        .ok_or(Error::MathOverflow)?;

    if untrusted_timestamp >= max_clock_drift_timestamp {
        return Err(InvalidHeaderError::SignedHeaderCannotExceedMaxClockDrift {
            signed_timestamp: untrusted_timestamp,
            max_clock_drift_timestamp,
        }
        .into());
    }

    let trusted_validators_hash = consensus_state.next_validators_hash;

    if untrusted_height_number == trusted_height_number + 1
        && header.signed_header.validators_hash != trusted_validators_hash
    {
        return Err(InvalidHeaderError::InvalidValidatorsHash {
            expected: trusted_validators_hash,
            actual: header.signed_header.validators_hash,
        }
        .into());
    }

    T::verify_zkp(
        &client_state.chain_id,
        trusted_validators_hash.into_encoding(),
        &header.signed_header,
        &header.zero_knowledge_proof,
    )
    .map_err(Error::InvalidZKP)
}

fn verify_misbehaviour<T: ZkpVerifier>(
    ctx: &IbcClientCtx<CometblsLightClient<T>>,
    client_state: &ClientState,
    consensus_state_a: &ConsensusState,
    consensus_state_b: &ConsensusState,
    misbehaviour: Misbehaviour,
) -> Result<(), Error> {
    if misbehaviour.header_a.signed_header.height < misbehaviour.header_b.signed_header.height {
        return Err(Error::InvalidMisbehaviourHeaderSequence);
    }

    verify_header(ctx, client_state, consensus_state_a, &misbehaviour.header_a)?;
    verify_header(ctx, client_state, consensus_state_b, &misbehaviour.header_b)?;

    if misbehaviour.header_a.signed_header.height == misbehaviour.header_b.signed_header.height {
        if misbehaviour.header_a.signed_header == misbehaviour.header_b.signed_header {
            return Ok(());
        }
    } else if misbehaviour.header_a.signed_header.time.as_unix_nanos()
        <= misbehaviour.header_b.signed_header.time.as_unix_nanos()
    {
        return Ok(());
    }

    Err(Error::MisbehaviourNotFound)
}

fn update_state<T: ZkpVerifier>(
    mut client_state: ClientState,
    mut consensus_state: ConsensusState,
    header: Header,
) -> Result<StateUpdate<CometblsLightClient<T>>, Error> {
    let untrusted_height = Height::new_with_revision(
        header.trusted_height.revision(),
        header.signed_header.height.inner() as u64,
    );

    consensus_state.app_hash = MerkleRoot {
        hash: header.signed_header.app_hash.into_encoding(),
    };

    consensus_state.next_validators_hash = header.signed_header.next_validators_hash;
    // Normalized to nanoseconds to follow tendermint convention
    consensus_state.timestamp = Timestamp::from_nanos(header.signed_header.time.as_unix_nanos());

    let state_update = StateUpdate::new(untrusted_height.height(), consensus_state);

    if untrusted_height > client_state.latest_height {
        client_state.latest_height = untrusted_height;
        Ok(state_update.overwrite_client_state(client_state))
    } else {
        Ok(state_update)
    }
}

fn is_client_expired(
    consensus_state_timestamp: Timestamp,
    trusting_period: Duration,
    current_block_time: Timestamp,
) -> bool {
    if let Some(sum) = consensus_state_timestamp.plus_duration(trusting_period) {
        sum < current_block_time
    } else {
        true
    }
}

// #[cfg(test)]
// mod tests {
//     use std::fs;

//     use cosmwasm_std::{
//         testing::{mock_dependencies, MockApi, MockQuerier, MockStorage},
//         OwnedDeps,
//     };
//     use ics008_wasm_client::{
//         storage_utils::{
//             consensus_db_key, read_subject_consensus_state, HOST_CLIENT_STATE_KEY,
//             SUBJECT_CLIENT_STORE_PREFIX, SUBSTITUTE_CLIENT_STORE_PREFIX,
//         },
//         FROZEN_HEIGHT,
//     };
//     use unionlabs::{encoding::EncodeAs, google::protobuf::any::Any};

//     use super::*;

//     const INITIAL_CONSENSUS_STATE_HEIGHT: Height = Height::new_with_revision(1, 1124);

//     const INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT: Height = Height::new_with_revision(1, 1200);

//     fn save_states_to_migrate_store(
//         deps: DepsMut,
//         subject_client_state: &WasmClientState,
//         substitute_client_state: &WasmClientState,
//         subject_consensus_state: &WasmConsensusState,
//         substitute_consensus_state: &WasmConsensusState,
//     ) {
//         deps.storage.set(
//             format!("{SUBJECT_CLIENT_STORE_PREFIX}{HOST_CLIENT_STATE_KEY}").as_bytes(),
//             &Any(subject_client_state.clone()).encode_as::<Proto>(),
//         );
//         deps.storage.set(
//             format!(
//                 "{SUBJECT_CLIENT_STORE_PREFIX}{}",
//                 consensus_db_key(&INITIAL_CONSENSUS_STATE_HEIGHT)
//             )
//             .as_bytes(),
//             &Any(subject_consensus_state.clone()).encode_as::<Proto>(),
//         );
//         deps.storage.set(
//             format!("{SUBSTITUTE_CLIENT_STORE_PREFIX}{HOST_CLIENT_STATE_KEY}").as_bytes(),
//             &Any(substitute_client_state.clone()).encode_as::<Proto>(),
//         );
//         deps.storage.set(
//             format!(
//                 "{SUBSTITUTE_CLIENT_STORE_PREFIX}{}",
//                 consensus_db_key(&INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT)
//             )
//             .as_bytes(),
//             &Any(substitute_consensus_state.clone()).encode_as::<Proto>(),
//         );
//     }

//     fn prepare_migrate_tests() -> (
//         OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
//         WasmClientState,
//         WasmConsensusState,
//         WasmClientState,
//         WasmConsensusState,
//     ) {
//         (
//             mock_dependencies(),
//             serde_json::from_str(&fs::read_to_string("src/test/client_state.json").unwrap())
//                 .unwrap(),
//             serde_json::from_str(&fs::read_to_string("src/test/consensus_state.json").unwrap())
//                 .unwrap(),
//             serde_json::from_str(
//                 &fs::read_to_string("src/test/substitute_client_state.json").unwrap(),
//             )
//             .unwrap(),
//             serde_json::from_str(
//                 &fs::read_to_string("src/test/substitute_consensus_state.json").unwrap(),
//             )
//             .unwrap(),
//         )
//     }

//     #[test]
//     fn migrate_client_store_works() {
//         let (
//             mut deps,
//             mut wasm_client_state,
//             wasm_consensus_state,
//             substitute_wasm_client_state,
//             substitute_wasm_consensus_state,
//         ) = prepare_migrate_tests();

//         wasm_client_state.data.frozen_height = FROZEN_HEIGHT;

//         save_states_to_migrate_store(
//             deps.as_mut(),
//             &wasm_client_state,
//             &substitute_wasm_client_state,
//             &wasm_consensus_state,
//             &substitute_wasm_consensus_state,
//         );

//         CometblsLightClient::<()>::migrate_client_store(deps.as_mut()).unwrap();

//         let wasm_client_state: WasmClientState =
//             read_subject_client_state::<CometblsLightClient<()>>(deps.as_ref()).unwrap();
//         // we didn't miss updating any fields
//         assert_eq!(wasm_client_state, substitute_wasm_client_state);
//         // client is unfrozen
//         assert_eq!(wasm_client_state.data.frozen_height, ZERO_HEIGHT);

//         // the new consensus state is saved under the correct height
//         assert_eq!(
//             read_subject_consensus_state::<CometblsLightClient<()>>(
//                 deps.as_ref(),
//                 &INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT
//             )
//             .unwrap()
//             .unwrap(),
//             substitute_wasm_consensus_state
//         );

//         // the new consensus state metadata is saved under substitute's latest height
//         assert_eq!(
//             get_current_or_next_consensus_state_meta(
//                 deps.as_ref(),
//                 INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT
//             )
//             .unwrap()
//             .unwrap()
//             .0,
//             substitute_wasm_client_state.latest_height
//         );
//     }

//     #[test]
//     fn migrate_client_store_fails_when_invalid_change() {
//         let (
//             mut deps,
//             wasm_client_state,
//             wasm_consensus_state,
//             substitute_wasm_client_state,
//             substitute_wasm_consensus_state,
//         ) = prepare_migrate_tests();

//         macro_rules! modify_fns {
//             ($param:ident, $($m:expr), + $(,)?) => ([$(|$param: &mut ClientState| $m),+])
//         }

//         let modifications = modify_fns! { s,
//             s.max_clock_drift ^= u64::MAX,
//         };

//         for m in modifications {
//             let mut state = substitute_wasm_client_state.clone();
//             m(&mut state.data);

//             save_states_to_migrate_store(
//                 deps.as_mut(),
//                 &wasm_client_state,
//                 &state,
//                 &wasm_consensus_state,
//                 &substitute_wasm_consensus_state,
//             );
//             assert_eq!(
//                 CometblsLightClient::<()>::migrate_client_store(deps.as_mut()),
//                 Err(Error::MigrateFieldsChanged.into())
//             );
//         }
//     }

//     #[test]
//     fn migrate_client_store_fails_when_substitute_client_frozen() {
//         let (
//             mut deps,
//             wasm_client_state,
//             wasm_consensus_state,
//             mut substitute_wasm_client_state,
//             substitute_wasm_consensus_state,
//         ) = prepare_migrate_tests();

//         substitute_wasm_client_state.data.frozen_height = FROZEN_HEIGHT;

//         save_states_to_migrate_store(
//             deps.as_mut(),
//             &wasm_client_state,
//             &substitute_wasm_client_state,
//             &wasm_consensus_state,
//             &substitute_wasm_consensus_state,
//         );

//         assert_eq!(
//             CometblsLightClient::<()>::migrate_client_store(deps.as_mut()),
//             Err(Error::SubstituteClientFrozen.into())
//         );
//     }
// }
