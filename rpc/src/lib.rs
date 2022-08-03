use codec::Codec;
use jsonrpsee::{
	core::{Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use sp_api::{BlockId, BlockT, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use std::{marker::PhantomData, sync::Arc};

pub use pallet_supersig_rpc_runtime_api::SuperSigApi as SuperSigRuntimeApi;

use pallet_supersig::{rpc::ProposalState, CallId, Role, SupersigId};

#[rpc(client, server)]
pub trait SuperSigApi<BlockHash, AccountId> {
	#[method(name = "superSig_getSupersigId")]
	fn get_supersig_id(
		&self,
		supersig_account: AccountId,
		at: Option<BlockHash>,
	) -> RpcResult<Option<SupersigId>>;
	#[method(name = "superSig_getUserSupersigs")]
	fn get_user_supersigs(
		&self,
		who: AccountId,
		at: Option<BlockHash>,
	) -> RpcResult<Vec<SupersigId>>;
	#[method(name = "superSig_listMembers")]
	fn list_members(
		&self,
		supersig_id: SupersigId,
		at: Option<BlockHash>,
	) -> RpcResult<Vec<(AccountId, Role)>>;
	#[method(name = "superSig_listProposals")]
	fn list_proposals(
		&self,
		supersig_id: SupersigId,
		at: Option<BlockHash>,
	) -> RpcResult<(Vec<ProposalState<AccountId>>, u32)>;
	#[method(name = "superSig_getProposalState")]
	fn get_proposal_state(
		&self,
		supersig_id: SupersigId,
		call_id: CallId,
		at: Option<BlockHash>,
	) -> RpcResult<Option<(ProposalState<AccountId>, u32)>>;
}

/// SuperSig RPC methods.
pub struct SuperSig<Client, Block> {
	client: Arc<Client>,
	_marker: PhantomData<Block>,
}

impl<Client, Block> SuperSig<Client, Block> {
	/// Create new `Supersig` with the given reference to the client.
	pub fn new(client: Arc<Client>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

impl<Client, Block, AccountId> SuperSigApiServer<<Block as BlockT>::Hash, AccountId>
	for SuperSig<Client, Block>
where
	Block: BlockT,
	Client: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	Client::Api: SuperSigRuntimeApi<Block, AccountId>,
	AccountId: Codec,
{
	fn get_supersig_id(
		&self,
		supersig_account: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Option<SupersigId>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		api.get_supersig_id(&at, supersig_account).map_err(runtime_error_into_rpc_err)
	}

	fn get_user_supersigs(
		&self,
		who: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Vec<SupersigId>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		api.get_user_supersigs(&at, who).map_err(runtime_error_into_rpc_err)
	}

	fn list_members(
		&self,
		supersig_id: SupersigId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Vec<(AccountId, Role)>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		api.list_members(&at, supersig_id).map_err(runtime_error_into_rpc_err)
	}

	fn list_proposals(
		&self,
		supersig_id: SupersigId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<(Vec<ProposalState<AccountId>>, u32)> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		api.list_proposals(&at, supersig_id).map_err(runtime_error_into_rpc_err)
	}

	fn get_proposal_state(
		&self,
		supersig_id: SupersigId,
		call_id: CallId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Option<(ProposalState<AccountId>, u32)>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		api.get_proposal_state(&at, supersig_id, call_id)
			.map_err(runtime_error_into_rpc_err)
	}
}

const RUNTIME_ERROR: i32 = 1;

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> JsonRpseeError {
	CallError::Custom(ErrorObject::owned(
		RUNTIME_ERROR,
		"Runtime error",
		Some(format!("{:?}", err)),
	))
	.into()
}
