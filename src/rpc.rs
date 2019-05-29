//! Remote Procedure Calls

// TODO: docs for everything
#![allow(missing_docs)]

use crate::{
    prost::encoding::{decode_varint, encoded_len_varint},
    prost::Message,
};

use bytes::IntoBuf;
use lazy_static::lazy_static;
use sha2::{Digest, Sha256};
use std::io::Cursor;
use std::io::{self, Read};
use std::io::{Error, ErrorKind};
use tendermint::amino_types::*;

/// Maximum size of an RPC message
pub const MAX_MSG_LEN: usize = 1024;

/// Requests to the KMS
pub enum Request {
    /// Sign the given message
    SignProposal(SignProposalRequest),
    SignVote(SignVoteRequest),
    ShowPublicKey(PubKeyRequest),

    // PingRequest is a PrivValidatorSocket message to keep the connection alive.
    ReplyPing(PingRequest),
}

/// Responses from the KMS
pub enum Response {
    /// Signature response
    SignedVote(SignedVoteResponse),
    SignedProposal(SignedProposalResponse),
    Ping(PingResponse),
    PublicKey(PubKeyResponse),
}

pub trait TendermintRequest: SignableMsg {
    // TODO(ismail): this should take an error as an argument:
    fn build_response(self) -> Response;
}

fn compute_prefix(name: &str) -> (Vec<u8>) {
    let mut sh = Sha256::default();
    sh.input(name.as_bytes());
    let output = sh.result();

    let prefix_bytes: Vec<u8> = output
        .iter()
        .filter(|&x| *x != 0x00)
        .skip(3)
        .filter(|&x| *x != 0x00)
        .cloned()
        .take(4)
        .collect();

    prefix_bytes
}

// pre-compute registered types prefix (this is probably sth. our amino library should
// provide instead)
lazy_static! {
    static ref VOTE_PREFIX: Vec<u8> = compute_prefix(VOTE_AMINO_NAME);
    static ref PROPOSAL_PREFIX: Vec<u8> = compute_prefix(PROPOSAL_AMINO_NAME);
    static ref PUBKEY_PREFIX: Vec<u8> = compute_prefix(PUBKEY_AMINO_NAME);
    static ref PING_PREFIX: Vec<u8> = compute_prefix(PING_AMINO_NAME);
}

impl Request {
    /// Read a request from the given readable
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        // this buffer contains the overall length and the amino prefix (for the registered types)
        let mut buf = vec![0; MAX_MSG_LEN];
        let bytes_read = r.read(&mut buf)?;
        if bytes_read < 4 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Did not read enough bytes to continue.",
            ));
        }

        let buff: &mut Cursor<Vec<u8>> = &mut buf.into_buf();
        let len = decode_varint(buff).unwrap();
        if len > MAX_MSG_LEN as u64 {
            return Err(Error::new(ErrorKind::InvalidData, "RPC message too large."));
        }
        let mut amino_pre = vec![0; 4];
        buff.read_exact(&mut amino_pre)?;
        buff.set_position(0);
        let total_len = encoded_len_varint(len).checked_add(len as usize).unwrap();
        let rem = buff.get_ref()[..total_len].to_vec();
        match amino_pre {
            ref vt if *vt == *VOTE_PREFIX => Ok(Request::SignVote(SignVoteRequest::decode(&rem)?)),
            ref pr if *pr == *PROPOSAL_PREFIX => {
                Ok(Request::SignProposal(SignProposalRequest::decode(&rem)?))
            }
            ref pubk if *pubk == *PUBKEY_PREFIX => {
                Ok(Request::ShowPublicKey(PubKeyRequest::decode(&rem)?))
            }
            ref ping if *ping == *PING_PREFIX => Ok(Request::ReplyPing(PingRequest::decode(&rem)?)),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "Received unknown RPC message.",
            )),
        }
    }
}

impl TendermintRequest for SignVoteRequest {
    fn build_response(self) -> Response {
        Response::SignedVote(SignedVoteResponse {
            vote: self.vote,
            err: None,
        })
    }
}

impl TendermintRequest for SignProposalRequest {
    fn build_response(self) -> Response {
        Response::SignedProposal(SignedProposalResponse {
            proposal: self.proposal,
            err: None,
        })
    }
}
