use futures_util::future::BoxFuture;

use serde_json::Value;
use vrcx_0_application_core::Result;
use vrcx_0_contracts::world_collections::{
    WorldCollectionCreatePayload, WorldCollectionCreateResponse, WorldCollectionSnapshotResponse,
    WorldCollectionTokenMintResponse, WorldOpenRegisterPayload,
};
use vrcx_0_contracts::WorldSummaryOutput;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldMemo {
    pub world_id: String,
    pub memo: String,
}

pub trait WorldCollectionStore: Send + Sync {
    fn world_summaries(&self, world_ids: &[String]) -> Result<Vec<WorldSummaryOutput>>;
    fn world_memos(&self, world_ids: &[String]) -> Result<Vec<WorldMemo>>;
    fn world_summary(&self, world_id: &str) -> Result<Option<WorldSummaryOutput>>;
    fn read_owner_tokens(&self) -> Result<Value>;
    fn write_owner_tokens(&self, owner_tokens: &Value) -> Result<()>;
}

pub type WorldCollectionFuture<'a, T> = BoxFuture<'a, Result<T>>;

pub trait WorldCollectionRemote: Send + Sync {
    fn mint_token<'a>(
        &'a self,
        owner_hint: &'a str,
    ) -> WorldCollectionFuture<'a, WorldCollectionTokenMintResponse>;
    fn create_collection<'a>(
        &'a self,
        token: &'a str,
        payload: &'a WorldCollectionCreatePayload,
    ) -> WorldCollectionFuture<'a, WorldCollectionCreateResponse>;
    fn fetch_collection<'a>(
        &'a self,
        id: &'a str,
    ) -> WorldCollectionFuture<'a, WorldCollectionSnapshotResponse>;
    fn register_world<'a>(
        &'a self,
        token: &'a str,
        payload: &'a WorldOpenRegisterPayload,
    ) -> WorldCollectionFuture<'a, ()>;
}
