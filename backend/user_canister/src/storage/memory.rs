use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager as IcMemoryManager};

pub const OWNER_MEMORY_ID: MemoryId = MemoryId::new(1);
pub const ORCHESTRATOR_MEMORY_ID: MemoryId = MemoryId::new(2);
pub const OWNER_PUBLIC_KEY_MEMORY_ID: MemoryId = MemoryId::new(3);

pub const FILE_COUNT_MEMORY_ID: MemoryId = MemoryId::new(10);
pub const FILE_ID_TO_PATH_MEMORY_ID: MemoryId = MemoryId::new(11);
pub const FILE_PATH_TO_ID_MEMORY_ID: MemoryId = MemoryId::new(12);
pub const OWNED_FILES_MEMORY_ID: MemoryId = MemoryId::new(13);
pub const FILE_DATA_MEMORY_ID: MemoryId = MemoryId::new(14);
pub const FILE_ALIAS_INDEX_MEMORY_ID: MemoryId = MemoryId::new(15);
pub const FILE_SHARES_MEMORY_ID: MemoryId = MemoryId::new(16);
pub const FILE_CONTENTS_MEMORY_ID: MemoryId = MemoryId::new(17);

thread_local! {
  /// Memory manager
  pub static MEMORY_MANAGER: IcMemoryManager<DefaultMemoryImpl> = IcMemoryManager::init(DefaultMemoryImpl::default());
}
