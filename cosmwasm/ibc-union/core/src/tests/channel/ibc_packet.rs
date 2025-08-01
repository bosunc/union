use cosmwasm_std::{testing::mock_dependencies, to_json_binary};
use ibc_union_msg::{
    lightclient::VerifyCreationResponse,
    msg::{
        InitMsg, MsgBatchAcks, MsgBatchSend, MsgChannelOpenAck, MsgIntentPacketRecv,
        MsgPacketAcknowledgement, MsgPacketRecv, MsgPacketTimeout, MsgSendPacket,
        MsgWriteAcknowledgement,
    },
};
use ibc_union_spec::{MustBeZero, Packet};

use super::*;
use crate::contract::init;

#[test]
fn send_packet_ok() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");

    let msg = MsgSendPacket {
        source_channel_id: ChannelId!(1),
        timeout_timestamp: Timestamp::from_nanos(1000000),
        data: vec![0, 1, 2].into(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg)
    )
    .is_ok())
}

#[test]
fn send_packet_missing_timeout() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");

    let msg = MsgSendPacket {
        source_channel_id: ChannelId!(1),
        timeout_timestamp: Timestamp::ZERO,
        data: vec![0, 1, 2].into(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .is_err_and(|err| { matches!(err, ContractError::TimeoutMustBeSet) }))
}

#[test]
fn send_packet_channel_does_not_exist() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");

    let msg = MsgSendPacket {
        source_channel_id: ChannelId!(3),
        timeout_timestamp: Timestamp::from_nanos(1000000),
        data: vec![0, 1, 2].into(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .is_err_and(|err| {
        match err {
            ContractError::Std(err) => matches!(err, StdError::GenericErr { .. }),
            _ => false,
        }
    }))
}

#[test]
fn send_packet_module_is_not_channel_owner() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");

    let msg = MsgSendPacket {
        source_channel_id: ChannelId!(1),
        timeout_timestamp: Timestamp::from_nanos(1000000),
        data: vec![0, 1, 2].into(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr("not module"), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .is_err_and(|err| { matches!(err, ContractError::Unauthorized { .. }) }))
}

#[test]
fn recv_packet_ok() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");

    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel_id: ChannelId!(2),
            destination_channel_id: ChannelId!(1),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp: Timestamp::from_nanos(2000000000000000000),
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .is_ok())
}

#[test]
fn recv_packet_invalid_channel_state() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");

    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel_id: ChannelId!(2),
            destination_channel_id: ChannelId!(5),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp: Timestamp::from_nanos(2000000000000000000),
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .is_err_and(|err| {
        match err {
            ContractError::Std(err) => {
                matches!(err, StdError::GenericErr { .. })
            }
            _ => false,
        }
    }))
}

#[test]
fn recv_packet_timeout_timestamp() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    let mut env = mock_env();
    env.block.time = cosmwasm_std::Timestamp::from_nanos(128);
    let timeout_timestamp = Timestamp::from_nanos(64);

    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");

    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel_id: ChannelId!(2),
            destination_channel_id: ChannelId!(1),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };

    assert!(execute(
        deps.as_mut(),
        env,
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .is_err_and(|err| { matches!(err, ContractError::ReceivedTimedOutPacketTimestamp { .. }) }))
}

#[test]
fn recv_intent_packet_ok() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");

    let msg = MsgIntentPacketRecv {
        packets: vec![Packet {
            source_channel_id: ChannelId!(2),
            destination_channel_id: ChannelId!(1),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp: Timestamp::from_nanos(2000000000000000000),
        }],
        market_maker_msgs: vec![vec![1, 2, 3].into()],
        market_maker: mock_addr("marketmaker").into_string(),
        empty_proof: vec![].into(),
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::IntentPacketRecv(msg)
    )
    .is_ok())
}

#[test]
fn recv_intent_packet_timeout_timestamp() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");

    let msg = MsgIntentPacketRecv {
        packets: vec![Packet {
            source_channel_id: ChannelId!(2),
            destination_channel_id: ChannelId!(1),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp: Timestamp::from_nanos(100),
        }],
        market_maker_msgs: vec![vec![1, 2, 3].into()],
        market_maker: mock_addr("marketmaker").into_string(),
        empty_proof: vec![].into(),
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::IntentPacketRecv(msg),
    )
    .is_err_and(|err| { matches!(err, ContractError::ReceivedTimedOutPacketTimestamp { .. }) }))
}

#[test]
fn acknowledge_packet_ok() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: ConnectionId!(1),
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: ChannelId!(2),
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: ChannelId!(1),
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgSendPacket {
        source_channel_id: ChannelId!(1),
        timeout_timestamp: Timestamp::from_nanos(2000000000000000000),
        data: vec![1, 2, 3].into(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .expect("send packet ok");

    let msg = MsgPacketAcknowledgement {
        packets: vec![Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(1),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp: Timestamp::from_nanos(2000000000000000000),
        }],
        acknowledgements: vec![vec![1, 2, 3].into()],
        proof: vec![1].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketAck(msg)
    )
    .is_ok())
}

#[test]
fn acknowledge_packet_tampered() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: ConnectionId!(1),
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: ChannelId!(2),
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: ChannelId!(1),
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgSendPacket {
        source_channel_id: ChannelId!(1),
        timeout_timestamp: Timestamp::from_nanos(2000000000000000000),
        data: vec![1, 2, 3].into(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .expect("send packet ok");

    let msg = MsgPacketAcknowledgement {
        packets: vec![Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(1),
            data: vec![4, 1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp: Timestamp::from_nanos(2000000000000000000),
        }],
        acknowledgements: vec![vec![1, 2, 3].into()],
        proof: vec![1].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketAck(msg)
    )
    .is_err_and(|err| matches!(err, ContractError::PacketCommitmentNotFound)))
}

#[test]
fn acknowledge_packet_not_sent() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: ConnectionId!(1),
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: ChannelId!(2),
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: ChannelId!(1),
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgPacketAcknowledgement {
        packets: vec![Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(1),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp: Timestamp::from_nanos(2000000000000000000),
        }],
        acknowledgements: vec![vec![1, 2, 3].into()],
        proof: vec![1].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketAck(msg)
    )
    .is_err_and(|err| matches!(err, ContractError::PacketCommitmentNotFound)))
}

#[test]
fn timeout_packet_timestamp_ok() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    let mut env = mock_env();
    env.block.time = cosmwasm_std::Timestamp::from_nanos(128);
    let timeout_timestamp = Timestamp::from_nanos(64);

    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");

    let msg = MsgSendPacket {
        source_channel_id: ChannelId!(1),
        timeout_timestamp,
        data: vec![1, 2, 3].into(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .expect("send packet ok");

    let msg = MsgPacketTimeout {
        packet: Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(1),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        },
        proof: vec![1].into(),
        proof_height: 11,
        relayer: mock_addr(RELAYER).into_string(),
    };
    assert!(execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketTimeout(msg)
    )
    .is_ok())
}

#[test]
fn timeout_packet_timestamp_timestamp_not_reached() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    let mut env = mock_env();
    env.block.time = cosmwasm_std::Timestamp::from_nanos(128);
    let timeout_timestamp = Timestamp::from_nanos(248);

    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&128),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");

    let msg = MsgSendPacket {
        source_channel_id: ChannelId!(1),
        timeout_timestamp,
        data: vec![1, 2, 3].into(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .expect("send packet ok");

    let msg = MsgPacketTimeout {
        packet: Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(1),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        },
        proof: vec![1].into(),
        proof_height: 11,
        relayer: mock_addr(RELAYER).into_string(),
    };
    assert!(execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketTimeout(msg)
    )
    .is_err_and(|err| { matches!(err, ContractError::TimeoutTimestampNotReached) }))
}

#[test]
fn write_acknowledgement_ok() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    let mut env = mock_env();
    env.block.time = cosmwasm_std::Timestamp::from_nanos(128);
    let timeout_timestamp = Timestamp::from_nanos(248);

    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: ConnectionId!(1),
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: ChannelId!(2),
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: ChannelId!(1),
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .expect("recv packet ok");

    let msg = MsgWriteAcknowledgement {
        packet: Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        },
        acknowledgement: vec![1].into(),
    };
    assert!(execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .is_ok())
}

#[test]
fn write_acknowledgement_module_is_not_channel_owner() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    let mut env = mock_env();
    env.block.time = cosmwasm_std::Timestamp::from_nanos(128);
    let timeout_timestamp = Timestamp::from_nanos(248);

    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    let msg = MsgChannelOpenInit {
        port_id: mock_addr("malicious").to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: ConnectionId!(1),
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr("malicious"), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: ChannelId!(2),
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: ChannelId!(1),
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr("malicious"), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .expect("recv packet ok");

    let msg = MsgWriteAcknowledgement {
        packet: Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        },
        acknowledgement: vec![1].into(),
    };
    assert!(execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .is_err_and(|err| { matches!(err, ContractError::Unauthorized { .. }) }))
}

#[test]
fn write_acknowledgement_packet_not_received() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: ConnectionId!(1),
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: ChannelId!(2),
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: ChannelId!(1),
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgWriteAcknowledgement {
        packet: Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp: Timestamp::from_nanos(2000000000000000000),
        },
        acknowledgement: vec![1].into(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .is_err_and(|err| { matches!(err, ContractError::PacketNotReceived) }))
}

#[test]
fn write_acknowledgement_already_exists() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    let mut env = mock_env();
    env.block.time = cosmwasm_std::Timestamp::from_nanos(128);
    let timeout_timestamp = Timestamp::from_nanos(248);

    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: ConnectionId!(1),
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: ChannelId!(2),
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: ChannelId!(1),
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .expect("recv packet ok");

    let msg = MsgWriteAcknowledgement {
        packet: Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        },
        acknowledgement: vec![1].into(),
    };
    assert!(execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .is_ok());
    let msg = MsgWriteAcknowledgement {
        packet: Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        },
        acknowledgement: vec![1].into(),
    };
    assert!(execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .is_err_and(|err| { matches!(err, ContractError::AlreadyAcknowledged) }))
}

#[test]
fn batch_send_ok() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    let mut env = mock_env();
    env.block.time = cosmwasm_std::Timestamp::from_nanos(128);
    let timeout_timestamp = Timestamp::from_nanos(248);

    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: ConnectionId!(1),
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: ChannelId!(2),
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: ChannelId!(1),
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgSendPacket {
        source_channel_id: ChannelId!(2),
        timeout_timestamp,
        data: vec![1, 2, 3].into(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .expect("send packet is ok");
    let msg = MsgSendPacket {
        source_channel_id: ChannelId!(2),
        timeout_timestamp,
        data: vec![4, 5, 6].into(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .expect("send packet is ok");

    let msg = MsgBatchSend {
        packets: vec![
            Packet {
                source_channel_id: ChannelId!(2),
                destination_channel_id: ChannelId!(1),
                data: vec![4, 5, 6].into(),
                timeout_height: MustBeZero,
                timeout_timestamp,
            },
            Packet {
                source_channel_id: ChannelId!(2),
                destination_channel_id: ChannelId!(1),
                data: vec![1, 2, 3].into(),
                timeout_height: MustBeZero,
                timeout_timestamp,
            },
        ],
    };
    assert!(execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::BatchSend(msg)
    )
    .is_ok())
}

#[test]
fn batch_send_packet_not_sent() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: ConnectionId!(1),
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: ChannelId!(2),
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: ChannelId!(1),
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgBatchSend {
        packets: vec![
            Packet {
                source_channel_id: ChannelId!(2),
                destination_channel_id: ChannelId!(1),
                data: vec![4, 5, 6].into(),
                timeout_height: MustBeZero,
                timeout_timestamp: Timestamp::ZERO,
            },
            Packet {
                source_channel_id: ChannelId!(2),
                destination_channel_id: ChannelId!(1),
                data: vec![1, 2, 3].into(),
                timeout_height: MustBeZero,
                timeout_timestamp: Timestamp::ZERO,
            },
        ],
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::BatchSend(msg)
    )
    .is_err_and(|err| { matches!(err, ContractError::PacketCommitmentNotFound) }))
}

#[test]
fn batch_acks_ok() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    let mut env = mock_env();
    env.block.time = cosmwasm_std::Timestamp::from_nanos(128);
    let timeout_timestamp = Timestamp::from_nanos(248);
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: ConnectionId!(1),
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: ChannelId!(2),
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: ChannelId!(1),
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .expect("recv packet ok");
    let msg = MsgWriteAcknowledgement {
        packet: Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        },
        acknowledgement: vec![1].into(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .expect("write ack is ok");
    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            data: vec![3, 4, 5].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .expect("recv packet ok");
    let msg = MsgWriteAcknowledgement {
        packet: Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            data: vec![3, 4, 5].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        },
        acknowledgement: vec![1].into(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .expect("write ack is ok");

    let msg = MsgBatchAcks {
        packets: vec![
            Packet {
                source_channel_id: ChannelId!(1),
                destination_channel_id: ChannelId!(2),
                data: vec![1, 2, 3].into(),
                timeout_height: MustBeZero,
                timeout_timestamp,
            },
            Packet {
                source_channel_id: ChannelId!(1),
                destination_channel_id: ChannelId!(2),
                data: vec![3, 4, 5].into(),
                timeout_height: MustBeZero,
                timeout_timestamp,
            },
        ],
        acks: vec![vec![1].into(), vec![1].into()],
    };
    assert!(execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::BatchAcks(msg)
    )
    .is_ok())
}

#[test]
fn batch_acks_packet_not_received() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: ConnectionId!(1),
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: ChannelId!(2),
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: ChannelId!(1),
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgBatchAcks {
        packets: vec![
            Packet {
                source_channel_id: ChannelId!(1),
                destination_channel_id: ChannelId!(2),
                data: vec![1, 2, 3].into(),
                timeout_height: MustBeZero,
                timeout_timestamp: Timestamp::ZERO,
            },
            Packet {
                source_channel_id: ChannelId!(1),
                destination_channel_id: ChannelId!(2),
                data: vec![3, 4, 5].into(),
                timeout_height: MustBeZero,
                timeout_timestamp: Timestamp::from_nanos(0),
            },
        ],
        acks: vec![vec![1].into(), vec![1].into()],
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::BatchAcks(msg)
    )
    .is_err_and(|err| { matches!(err, ContractError::PacketCommitmentNotFound) }))
}

#[test]
fn batch_acks_tampered_packet() {
    let mut deps = mock_dependencies();
    init(
        deps.as_mut(),
        InitMsg {
            relayers_admin: None,
            relayers: vec![mock_addr(SENDER).to_string()],
        },
    )
    .expect("init is ok");
    let mut env = mock_env();
    env.block.time = cosmwasm_std::Timestamp::from_nanos(128);
    let timeout_timestamp = Timestamp::from_nanos(248);
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: ConnectionId!(1),
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: ChannelId!(2),
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: ChannelId!(1),
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .expect("recv packet ok");
    let msg = MsgWriteAcknowledgement {
        packet: Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            data: vec![1, 2, 3].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        },
        acknowledgement: vec![1].into(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .expect("write ack is ok");
    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            data: vec![3, 4, 5].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .expect("recv packet ok");
    let msg = MsgWriteAcknowledgement {
        packet: Packet {
            source_channel_id: ChannelId!(1),
            destination_channel_id: ChannelId!(2),
            data: vec![3, 4, 5].into(),
            timeout_height: MustBeZero,
            timeout_timestamp,
        },
        acknowledgement: vec![1].into(),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .expect("write ack is ok");

    let msg = MsgBatchAcks {
        packets: vec![
            Packet {
                source_channel_id: ChannelId!(1),
                destination_channel_id: ChannelId!(2),
                data: vec![10, 20, 30].into(),
                timeout_height: MustBeZero,
                timeout_timestamp,
            },
            Packet {
                source_channel_id: ChannelId!(1),
                destination_channel_id: ChannelId!(2),
                data: vec![30, 40, 50].into(),
                timeout_height: MustBeZero,
                timeout_timestamp: Timestamp::ZERO,
            },
        ],
        acks: vec![vec![1].into(), vec![1].into()],
    };
    assert!(execute(
        deps.as_mut(),
        env.clone(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::BatchAcks(msg)
    )
    .is_err_and(|err| { matches!(err, ContractError::PacketCommitmentNotFound) }))
}
