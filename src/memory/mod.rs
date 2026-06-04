#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionKind {
    Usable,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    BadMemory,
    BootloaderReclaimable,
    KernelAndModules,
    Framebuffer,
    ReservedMapped,
    Unknown(u64),
}

impl MemoryRegionKind {
    pub const fn from_limine(value: u64) -> Self {
        match value {
            0 => Self::Usable,
            1 => Self::Reserved,
            2 => Self::AcpiReclaimable,
            3 => Self::AcpiNvs,
            4 => Self::BadMemory,
            5 => Self::BootloaderReclaimable,
            6 => Self::KernelAndModules,
            7 => Self::Framebuffer,
            8 => Self::ReservedMapped,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryRegion {
    pub base: u64,
    pub length: u64,
    pub kind: MemoryRegionKind,
}

impl MemoryRegion {
    #[cfg(target_os = "none")]
    pub const fn empty() -> Self {
        Self {
            base: 0,
            length: 0,
            kind: MemoryRegionKind::Reserved,
        }
    }
}

pub struct MemoryMap {
    entries: &'static [MemoryRegion],
}

impl MemoryMap {
    pub const fn empty() -> Self {
        Self { entries: &[] }
    }

    pub const fn new(entries: &'static [MemoryRegion]) -> Self {
        Self { entries }
    }

    pub const fn entries(&self) -> &'static [MemoryRegion] {
        self.entries
    }

    pub fn total_usable_bytes(&self) -> u64 {
        self.entries
            .iter()
            .filter(|entry| entry.kind == MemoryRegionKind::Usable)
            .map(|entry| entry.length)
            .sum()
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::{MemoryMap, MemoryRegion, MemoryRegionKind};

    static REGIONS: [MemoryRegion; 2] = [
        MemoryRegion {
            base: 0x1000,
            length: 0x3000,
            kind: MemoryRegionKind::Usable,
        },
        MemoryRegion {
            base: 0x4000,
            length: 0x1000,
            kind: MemoryRegionKind::Reserved,
        },
    ];

    #[test]
    fn sums_only_usable_memory() {
        let map = MemoryMap::new(&REGIONS);

        assert_eq!(map.total_usable_bytes(), 0x3000);
    }

    #[test]
    fn exposes_entries() {
        let map = MemoryMap::new(&REGIONS);

        assert_eq!(map.entries().len(), 2);
    }

    #[test]
    fn empty_map_has_no_regions() {
        assert_eq!(MemoryMap::empty().entries().len(), 0);
    }

    #[test]
    fn converts_limine_region_kinds() {
        assert_eq!(MemoryRegionKind::from_limine(0), MemoryRegionKind::Usable);
        assert_eq!(MemoryRegionKind::from_limine(7), MemoryRegionKind::Framebuffer);
        assert_eq!(MemoryRegionKind::from_limine(99), MemoryRegionKind::Unknown(99));
    }
}
