use std::sync::Arc;
use vrcx_0_application_core::{Error, Result, RuntimeAuthScope};
use vrcx_0_contracts::{
    SavedGroupCollectionCreateInput, SavedGroupCollectionDeleteInput, SavedGroupFavoriteAddInput,
    SavedGroupFavoriteRemoveInput, SavedGroupFavoritesSnapshot,
};
use vrcx_0_core::{vrchat_ids::is_group_id, OwnerId};

pub trait SavedGroupFavoritesPort: Send + Sync {
    fn snapshot(&self, owner: &OwnerId) -> Result<SavedGroupFavoritesSnapshot>;
    fn create_collection(&self, owner: &OwnerId, collection_id: &str, name: &str) -> Result<i64>;
    fn delete_collection(&self, owner: &OwnerId, collection_id: &str) -> Result<i64>;
    fn add_group(&self, owner: &OwnerId, collection_id: &str, group_id: &str) -> Result<i64>;
    fn remove_group(&self, owner: &OwnerId, group_id: &str) -> Result<i64>;
    fn favorites_changed(&self);
}

#[derive(Clone)]
pub struct SavedGroupFavoritesRuntime {
    port: Arc<dyn SavedGroupFavoritesPort>,
    auth_scope: RuntimeAuthScope,
}

impl SavedGroupFavoritesRuntime {
    pub fn new(port: Arc<dyn SavedGroupFavoritesPort>, auth_scope: RuntimeAuthScope) -> Self {
        Self { port, auth_scope }
    }

    fn owner(&self) -> Result<OwnerId> {
        let scope = self.auth_scope.snapshot();
        if !scope.active {
            return Err(Error::Custom(
                "Saved group favorites require an authenticated session.".into(),
            ));
        }
        Ok(OwnerId::new(scope.current_user_id))
    }

    pub fn snapshot(&self) -> Result<SavedGroupFavoritesSnapshot> {
        self.port.snapshot(&self.owner()?)
    }

    pub fn create_collection(&self, input: SavedGroupCollectionCreateInput) -> Result<i64> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(Error::Custom(
                "Saved group collection name is required.".into(),
            ));
        }
        let affected =
            self.port
                .create_collection(&self.owner()?, &uuid::Uuid::new_v4().to_string(), name)?;
        self.port.favorites_changed();
        Ok(affected)
    }

    pub fn delete_collection(&self, input: SavedGroupCollectionDeleteInput) -> Result<i64> {
        let affected = self
            .port
            .delete_collection(&self.owner()?, &input.collection_id)?;
        self.port.favorites_changed();
        Ok(affected)
    }

    pub fn add_group(&self, input: SavedGroupFavoriteAddInput) -> Result<i64> {
        if !is_group_id(input.group_id.trim()) {
            return Err(Error::Custom(
                "Saved group favorite requires a canonical group ID.".into(),
            ));
        }
        let affected =
            self.port
                .add_group(&self.owner()?, &input.collection_id, &input.group_id)?;
        self.port.favorites_changed();
        Ok(affected)
    }

    pub fn remove_group(&self, input: SavedGroupFavoriteRemoveInput) -> Result<i64> {
        if !is_group_id(input.group_id.trim()) {
            return Err(Error::Custom(
                "Saved group favorite requires a canonical group ID.".into(),
            ));
        }
        let affected = self.port.remove_group(&self.owner()?, &input.group_id)?;
        self.port.favorites_changed();
        Ok(affected)
    }
}

#[cfg(test)]
mod tests;
