use crate::kvstore::kv::KeyValue;
use crate::kvstore::server::OmniPaxosServer;

use super::client::OmnipaxosTransport;
use super::connection::proto;
use super::parse_utils;

use omnipaxos::messages::ballot_leader_election::{
    BLEMessage, HeartbeatMsg, HeartbeatReply, HeartbeatRequest,
};
use omnipaxos::messages::sequence_paxos::{
    AcceptDecide, AcceptStopSign, AcceptSync, Accepted, AcceptedStopSign, Decide, DecideStopSign,
    PaxosMessage, PaxosMsg, Prepare, Promise,
};
use omnipaxos::messages::Message;

use proto::omni_paxos_protocol_server::OmniPaxosProtocol;
use tonic::async_trait;
use tonic::{Request, Response, Status};

struct OmniPaxosProtocolService<T: OmnipaxosTransport + Send + Sync> {
    omnipaxos_server: OmniPaxosServer<T>,
}

#[async_trait()]
impl<T: OmnipaxosTransport + Send + Sync + 'static> OmniPaxosProtocol
    for OmniPaxosProtocolService<T>
{
    async fn set_request(&self, req: Request<proto::Set>) -> Result<Response<proto::Void>, Status> {
        let req = req.into_inner();
        let key = req.key;
        let value = req.value;

        let keyval = KeyValue { key, value };

        self.omnipaxos_server.handle_set(keyval);

        Ok(Response::new(proto::Void {}))
    }

    async fn get_request(
        &self,
        req: Request<proto::Get>,
    ) -> Result<Response<proto::Result>, Status> {
        let req = req.into_inner();
        let key = req.key;

        let value = self.omnipaxos_server.handle_get(key);
        let result = proto::Result { value };

        Ok(Response::new(result))
    }

    async fn prepare_request(
        &self,
        req: Request<proto::PrepareReq>,
    ) -> Result<Response<proto::Void>, Status> {
        let req = req.into_inner();
        let from = req.from as u64;
        let to = req.to as u64;

        let msg: PaxosMsg<KeyValue> = PaxosMsg::PrepareReq;
        let paxos_msg = PaxosMessage { from, to, msg };
        let message = Message::SequencePaxos(paxos_msg);
        self.omnipaxos_server.receive_message(message);

        Ok(Response::new(proto::Void {}))
    }

    async fn prepare_message(
        &self,
        req: Request<proto::Prepare>,
    ) -> Result<Response<proto::Void>, Status> {
        let req = req.into_inner();
        let from = req.from as u64;
        let to = req.to as u64;

        let n = parse_utils::get_ballot_struct(req.n.unwrap());
        let decided_idx = req.decided_idx;
        let n_accepted = parse_utils::get_ballot_struct(req.n_accepted.unwrap());
        let accepted_idx = req.accepted_idx;

        let prepare = Prepare {
            n,
            decided_idx,
            n_accepted,
            accepted_idx,
        };
        let msg: PaxosMsg<KeyValue> = PaxosMsg::Prepare(prepare);
        let paxos_msg = PaxosMessage { from, to, msg };
        let message = Message::SequencePaxos(paxos_msg);
        self.omnipaxos_server.receive_message(message);

        Ok(Response::new(proto::Void {}))
    }

    async fn promise_message(
        &self,
        req: Request<proto::Promise>,
    ) -> Result<Response<proto::Void>, Status> {
        let req = req.into_inner();
        let from = req.from as u64;
        let to = req.to as u64;

        let n = parse_utils::get_ballot_struct(req.n.unwrap());
        let n_accepted = parse_utils::get_ballot_struct(req.n_accepted.unwrap());
        let decided_snapshot = match req.decided_snapshot {
            Some(decided_snapshot) => Some(parse_utils::get_snapshot_type_enum(decided_snapshot)),
            _ => None,
        };
        let suffix: Vec<KeyValue> = req
            .suffix
            .into_iter()
            .map(|kv| parse_utils::get_keyval_struct(kv))
            .collect();
        let decided_idx = req.decided_idx;
        let accepted_idx = req.accepted_idx;
        let stopsign = match req.stopsign {
            Some(stopsign) => Some(parse_utils::get_stopsign_struct(stopsign)),
            _ => None,
        };

        let promise = Promise {
            n,
            n_accepted,
            decided_snapshot,
            suffix,
            decided_idx,
            accepted_idx,
            stopsign,
        };

        let msg: PaxosMsg<KeyValue> = PaxosMsg::Promise(promise);
        let paxos_msg = PaxosMessage { from, to, msg };
        let message = Message::SequencePaxos(paxos_msg);
        self.omnipaxos_server.receive_message(message);

        Ok(Response::new(proto::Void {}))
    }

    async fn accept_sync_message(
        &self,
        req: Request<proto::AcceptSync>,
    ) -> Result<Response<proto::Void>, Status> {
        let req = req.into_inner();
        let from = req.from as u64;
        let to = req.to as u64;

        let n = parse_utils::get_ballot_struct(req.n.unwrap());
        let seq_num = parse_utils::get_seq_num_struct(req.seq_num.unwrap());
        let decided_snapshot = match req.decided_snapshot {
            Some(decided_snapshot) => Some(parse_utils::get_snapshot_type_enum(decided_snapshot)),
            _ => None,
        };
        let suffix: Vec<KeyValue> = req
            .suffix
            .into_iter()
            .map(|kv| parse_utils::get_keyval_struct(kv))
            .collect();
        let sync_idx = req.sync_idx;
        let decided_idx = req.decided_idx;
        let stopsign = match req.stopsign {
            Some(stopsign) => Some(parse_utils::get_stopsign_struct(stopsign)),
            _ => None,
        };

        let accept_sync = AcceptSync {
            n,
            seq_num,
            decided_snapshot,
            suffix,
            sync_idx,
            decided_idx,
            stopsign,
        };

        let msg: PaxosMsg<KeyValue> = PaxosMsg::AcceptSync(accept_sync);
        let paxos_msg = PaxosMessage { from, to, msg };
        let message = Message::SequencePaxos(paxos_msg);
        self.omnipaxos_server.receive_message(message);

        Ok(Response::new(proto::Void {}))
    }

    async fn accept_decide_message(
        &self,
        req: Request<proto::AcceptDecide>,
    ) -> Result<Response<proto::Void>, Status> {
        let req = req.into_inner();
        let from = req.from as u64;
        let to = req.to as u64;

        let n = parse_utils::get_ballot_struct(req.n.unwrap());
        let seq_num = parse_utils::get_seq_num_struct(req.seq_num.unwrap());
        let decided_idx = req.decided_idx;
        let entries: Vec<KeyValue> = req
            .entries
            .into_iter()
            .map(|kv| parse_utils::get_keyval_struct(kv))
            .collect();

        let accept_decide = AcceptDecide {
            n,
            seq_num,
            decided_idx,
            entries,
        };

        let msg: PaxosMsg<KeyValue> = PaxosMsg::AcceptDecide(accept_decide);
        let paxos_msg = PaxosMessage { from, to, msg };
        let message = Message::SequencePaxos(paxos_msg);
        self.omnipaxos_server.receive_message(message);

        Ok(Response::new(proto::Void {}))
    }

    async fn accepted_message(
        &self,
        req: Request<proto::Accepted>,
    ) -> Result<Response<proto::Void>, Status> {
        let req = req.into_inner();
        let from = req.from as u64;
        let to = req.to as u64;

        let n = parse_utils::get_ballot_struct(req.n.unwrap());
        let accepted_idx = req.accepted_idx;

        let accepted = Accepted { n, accepted_idx };
        let msg: PaxosMsg<KeyValue> = PaxosMsg::Accepted(accepted);
        let paxos_msg = PaxosMessage { from, to, msg };
        let message = Message::SequencePaxos(paxos_msg);
        self.omnipaxos_server.receive_message(message);

        Ok(Response::new(proto::Void {}))
    }

    async fn decide_message(
        &self,
        req: Request<proto::Decide>,
    ) -> Result<Response<proto::Void>, Status> {
        let req = req.into_inner();
        let from = req.from as u64;
        let to = req.to as u64;

        let n = parse_utils::get_ballot_struct(req.n.unwrap());
        let seq_num = parse_utils::get_seq_num_struct(req.seq_num.unwrap());
        let decided_idx = req.decided_idx;

        let decide = Decide {
            n,
            seq_num,
            decided_idx,
        };
        let msg: PaxosMsg<KeyValue> = PaxosMsg::Decide(decide);
        let paxos_msg = PaxosMessage { from, to, msg };
        let message = Message::SequencePaxos(paxos_msg);
        self.omnipaxos_server.receive_message(message);

        Ok(Response::new(proto::Void {}))
    }

    async fn proposal_forward_message(
        &self,
        req: Request<proto::ProposalForward>,
    ) -> Result<Response<proto::Void>, Status> {
        let req = req.into_inner();
        let from = req.from as u64;
        let to = req.to as u64;

        let proposals: Vec<KeyValue> = req
            .proposals
            .into_iter()
            .map(|kv| parse_utils::get_keyval_struct(kv))
            .collect();
        let msg: PaxosMsg<KeyValue> = PaxosMsg::ProposalForward(proposals);
        let paxos_msg = PaxosMessage { from, to, msg };
        let message = Message::SequencePaxos(paxos_msg);
        self.omnipaxos_server.receive_message(message);

        Ok(Response::new(proto::Void {}))
    }

    async fn compaction_message(
        &self,
        req: Request<proto::Compaction>,
    ) -> Result<Response<proto::Void>, Status> {
        let req = req.into_inner();
        let from = req.from as u64;
        let to = req.to as u64;

        let compaction = parse_utils::get_compaction_enum(req.compaction.unwrap());

        let msg: PaxosMsg<KeyValue> = PaxosMsg::Compaction(compaction);
        let paxos_msg = PaxosMessage { from, to, msg };
        let message = Message::SequencePaxos(paxos_msg);
        self.omnipaxos_server.receive_message(message);

        Ok(Response::new(proto::Void {}))
    }

    async fn accept_stop_sign_message(
        &self,
        req: Request<proto::AcceptStopSign>,
    ) -> Result<Response<proto::Void>, Status> {
        let req = req.into_inner();
        let from = req.from as u64;
        let to = req.to as u64;

        let n = parse_utils::get_ballot_struct(req.n.unwrap());
        // let seq_num = parse_utils::get_seq_num_struct(req.seq_num.unwrap());
        let ss = parse_utils::get_stopsign_struct(req.ss.unwrap());

        let accept_stop_sign = AcceptStopSign { n, ss };
        let msg: PaxosMsg<KeyValue> = PaxosMsg::AcceptStopSign(accept_stop_sign);
        let paxos_msg = PaxosMessage { from, to, msg };
        let message = Message::SequencePaxos(paxos_msg);
        self.omnipaxos_server.receive_message(message);

        Ok(Response::new(proto::Void {}))
    }

    async fn accepted_stop_sign_message(
        &self,
        req: Request<proto::AcceptedStopSign>,
    ) -> Result<Response<proto::Void>, Status> {
        let req = req.into_inner();
        let from = req.from as u64;
        let to = req.to as u64;

        let n = parse_utils::get_ballot_struct(req.n.unwrap());

        let accepted_stop_sign = AcceptedStopSign { n };
        let msg: PaxosMsg<KeyValue> = PaxosMsg::AcceptedStopSign(accepted_stop_sign);
        let paxos_msg = PaxosMessage { from, to, msg };
        let message = Message::SequencePaxos(paxos_msg);
        self.omnipaxos_server.receive_message(message);

        Ok(Response::new(proto::Void {}))
    }

    async fn decide_stop_sign_message(
        &self,
        req: Request<proto::DecideStopSign>,
    ) -> Result<Response<proto::Void>, Status> {
        let req = req.into_inner();
        let from = req.from as u64;
        let to = req.to as u64;

        let n = parse_utils::get_ballot_struct(req.n.unwrap());
        // let seq_num = parse_utils::get_seq_num_struct(req.seq_num.unwrap());

        let decide_stop_sign = DecideStopSign { n };
        let msg: PaxosMsg<KeyValue> = PaxosMsg::DecideStopSign(decide_stop_sign);
        let paxos_msg = PaxosMessage { from, to, msg };
        let message = Message::SequencePaxos(paxos_msg);
        self.omnipaxos_server.receive_message(message);

        Ok(Response::new(proto::Void {}))
    }

    async fn forward_stop_sign_message(
        &self,
        req: Request<proto::ForwardStopSign>,
    ) -> Result<Response<proto::Void>, Status> {
        let req = req.into_inner();
        let from = req.from as u64;
        let to = req.to as u64;

        let ss = parse_utils::get_stopsign_struct(req.ss.unwrap());
        let msg: PaxosMsg<KeyValue> = PaxosMsg::ForwardStopSign(ss);
        let paxos_msg = PaxosMessage { from, to, msg };
        let message = Message::SequencePaxos(paxos_msg);
        self.omnipaxos_server.receive_message(message);

        Ok(Response::new(proto::Void {}))
    }

    async fn heartbeat_request_message(
        &self,
        req: Request<proto::HeartbeatRequest>,
    ) -> Result<Response<proto::Void>, Status> {
        let req = req.into_inner();
        let from = req.from as u64;
        let to = req.to as u64;

        let round = req.round;

        let heartbeat_request = HeartbeatRequest { round };
        let msg = HeartbeatMsg::Request(heartbeat_request);
        let ble_msg = BLEMessage { from, to, msg };
        let message = Message::BLE(ble_msg);
        self.omnipaxos_server.receive_message(message);

        Ok(Response::new(proto::Void {}))
    }

    async fn heartbeat_reply_message(
        &self,
        req: Request<proto::HeartbeatReply>,
    ) -> Result<Response<proto::Void>, Status> {
        let req = req.into_inner();
        let from = req.from as u64;
        let to = req.to as u64;

        let round = req.round;
        let ballot = parse_utils::get_ballot_struct(req.ballot.unwrap());
        let quorum_connected = req.quorum_connected;

        let heartbeat_reply = HeartbeatReply {
            round,
            ballot,
            quorum_connected,
        };
        let msg = HeartbeatMsg::Reply(heartbeat_reply);
        let ble_msg = BLEMessage { from, to, msg };
        let message = Message::BLE(ble_msg);
        self.omnipaxos_server.receive_message(message);

        Ok(Response::new(proto::Void {}))
    }
}
