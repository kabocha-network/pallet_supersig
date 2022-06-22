#[rpc]
pub trait SupersigApi<BlockHash> {
    #[rpc(name = "get_supersigs_connected_to_an_account")]
    fn get_supersigs_connected_to_an_account(
        &self,
        at: Option<BlockHash>
    ) -> Result<u32>;
    #[rpc(name = "get_members_connected_to_each_supersigs")]

    fn get_members_connected_to_each_supersigs(
        &self,
        at: Option<BlockHash>
    ) -> Result<u32>;
    #[rpc(name = "get_list_of_proposals_connected_to_supersig")]

    fn get_list_of_proposals_connected_to_supersig(
        &self,
        at: Option<BlockHash>
    ) -> Result<u32>;
    #[rpc(name = "get_voting_state_from_proposal")]

    fn get_voting_state_from_proposal(
        &self,
        at: Option<BlockHash>
    ) -> Result<u32>;
}

pub struct Supersig<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Supersig<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client, _marker: Default::default() }
    }
}

impl<C, Block> SupersigApi<<Block as BlockT>::Hash>
    for Supersig<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi,
    C: HeaderBackend<Block>,
    C::Api: SupersigRuntimeApi<Block>,
{
    // fn get_sum(
    //     &self,
    //     at: Option<<Block as BlockT>::Hash>
    // ) -> Result<u32> {

    //     let api = self.client.runtime_api();
    //     let at = BlockId::hash(at.unwrap_or_else(||
    //         // If the block hash is not supplied assume the best block.
    //         self.client.info().best_hash
    //     ));

    //     let runtime_api_result = api.get_sum(&at);
    //     runtime_api_result.map_err(|e| RpcError {
    //         code: ErrorCode::ServerError(9876), // No real reason for this value
    //         message: "Something wrong".into(),
    //         data: Some(format!("{:?}", e).into()),
    //     })
    // }

    fn get_supersigs_connected_to_an_account(
        &self,
        at: Option<<Block as BlockT>::Hash>
    ) -> Result<u32> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            self.client.info().best_hash
        ));

        let runtime_api_result = api.members(&at);
        // runtime_api_result.map_err(|e| RpcError {
        //     code: ErrorCode::ServerError(9876),
        //     message: "Something wrong".into(),
        //     data: Some(format!("{:?}", e).into()),
        // })
    }

    fn get_members_connected_to_each_supersigs(
        &self,
        at: Option<<Block as BlockT>::Hash>
    ) -> Result<u32> {
    }

    fn get_list_of_proposals_connected_to_supersig(
        &self,
        at: Option<<Block as BlockT>::Hash>
    ) -> Result<u32> {
    }

    fn get_voting_state_from_proposal(
        &self,
        at: Option<<Block as BlockT>::Hash>
    ) -> Result<u32> {
    }
}