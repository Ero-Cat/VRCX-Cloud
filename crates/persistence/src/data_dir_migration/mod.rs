mod filesystem;
mod journal;
mod lifecycle;
mod types;

#[cfg(test)]
mod tests;

pub use filesystem::{
    cleanup_manifest_size, cleanup_migrated_data, clear_data_dir_migration_staging,
    copy_frozen_database_to_staging, copy_frozen_database_to_staging_cancellable,
    data_dir_available_space, data_dir_migration_required_bytes, finalize_data_dir_migration,
    install_staged_data_dir_database,
};
pub use journal::{
    has_pending_data_dir_migration, migration_journal_path, read_data_dir_cleanup_pending,
    read_data_dir_cleanup_pendings, read_pending_data_dir_migration,
    remove_data_dir_cleanup_pending, remove_pending_data_dir_migration,
    take_data_dir_migration_result, write_data_dir_cleanup_pending,
    write_data_dir_migration_result, write_pending_data_dir_migration,
};
pub use lifecycle::{
    cleanup_interrupted_data_dir_migration, complete_data_dir_migration, dismiss_data_dir_cleanup,
    inspect_data_dir_migration_target, record_data_dir_migration_database_open_failure,
};
pub use types::{
    DataDirCleanupPending, DataDirCleanupReport, DataDirMigrationFinalizeOutcome,
    DataDirMigrationJournalPhase, DataDirMigrationResult, DataDirMigrationResultStatus,
    DataDirMigrationTargetState, DataDirMigrationWarning, PendingDataDirMigration,
    StagedDataDirMigration, DATA_DIR_CLEANUP_PENDING_FILE_NAME,
    DATA_DIR_MIGRATION_JOURNAL_FILE_NAME, DATA_DIR_MIGRATION_REPLACED_PREFIX,
    DATA_DIR_MIGRATION_RESULT_FILE_NAME, DATA_DIR_MIGRATION_SPACE_MARGIN_BYTES,
    DATA_DIR_MIGRATION_STAGING_DIRECTORY,
};
