use codec::Codec;
use jsonrpsee::{
	core::{Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use sp_api::BlockId;
use sp_api::BlockT;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use std::{marker::PhantomData, sync::Arc};

pub use pallet_supersig_rpc_runtime_api::SuperSigApi as SuperSigRuntimeApi;

#[rpc(client, server)]
pub trait SuperSigApi<BlockHash, AccountId> {
	#[method(name = "superSig_getAccountSupersigs")]
	fn get_account_supersigs(&self, who: AccountId, at: Option<BlockHash>) -> RpcResult<Vec<u128>>;
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
	fn get_account_supersigs(
		&self,
		who: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Vec<u128>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		api.get_account_supersigs(&at, who).map_err(runtime_error_into_rpc_err)
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
