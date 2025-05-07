use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager as IcMemoryManager};

pub const ORBIT_STATION_MEMORY_ID: MemoryId = MemoryId::new(1);

pub const USER_STORAGE_MEMORY_ID: MemoryId = MemoryId::new(10);
pub const USERNAMES_MEMORY_ID: MemoryId = MemoryId::new(11);

thread_local! {
    /// Memory manager
    pub static MEMORY_MANAGER: IcMemoryManager<DefaultMemoryImpl> = IcMemoryManager::init(DefaultMemoryImpl::default());
}
