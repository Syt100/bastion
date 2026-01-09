mod fetch;
mod list;
mod types;

pub use list::{list_run_entries_children, list_run_entries_children_with_options};
pub use types::{ListRunEntriesChildrenOptions, RunEntriesChild, RunEntriesChildrenResponse};

pub(super) use fetch::fetch_entries_index;
pub(super) use types::EntryRecord;

pub(super) type ListChildrenFromEntriesIndexOptions = types::ListChildrenFromEntriesIndexOptions;

pub(super) fn list_children_from_entries_index(
    entries_path: &std::path::Path,
    options: ListChildrenFromEntriesIndexOptions,
) -> Result<RunEntriesChildrenResponse, anyhow::Error> {
    list::list_children_from_entries_index(entries_path, options)
}
