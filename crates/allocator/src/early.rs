extern crate alloc;
use log::{debug, warn};

use super::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};
use core::alloc::Layout;
use core::ptr::NonNull;

pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start_va: usize,
    total_bytes: usize,
    byte_index:usize,
    byte_used:usize,
    page_index:usize,
    page_used:usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self { 
            start_va: 0,
            total_bytes: 0, 
            byte_index: 0, 
            byte_used: 0,
            page_index: 0,
            page_used: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start_va = start;
        self.total_bytes = size;
        self.page_index = size;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        if self.start_va + self.total_bytes != start {
            Err(AllocError::MemoryOverlap)
        }
        else {
            self.total_bytes += size;
            Ok(())
        }
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        /*let layout = layout.pad_to_align();
        if self.byte_index + layout.size() < self.page_index {
            let ptr = NonNull::new((self.start_va + self.byte_index) as *mut u8).unwrap();
            self.byte_index += layout.size();
            self.byte_used += layout.size();
            /*debug!(
                "Allocate bytes Ok: start:{:#x}, size:{:#x}", 
                self.start_va + self.byte_index, 
                layout.size()
            );*/
            Ok(ptr)
        }
        else {
            warn!("Allocate bytes failed:NoMemory");
            Err(AllocError::NoMemory)
        }*/

        debug!("Allocate bytes, Layout.size:{:#x}, layout.align:{}", layout.size(),layout.align());
        let raw_addres = self.start_va + self.byte_index;
        let mod_res = raw_addres % layout.align();
        let address_align = if mod_res == 0 {
            raw_addres
        }
        else {
            raw_addres + layout.align() - mod_res
        };
        let layout = layout.pad_to_align();
        debug!("Padding addres, raw_address:{:#x}, addr_align:{:#x}, align:{}", raw_addres, address_align, layout.align());
        let size_need = layout.size() + address_align - raw_addres;
        let end_addr = address_align + layout.size();
        if end_addr < self.start_va + self.page_index {
            let ptr = NonNull::new((address_align) as *mut u8).unwrap();
            debug!(
                "Allocate bytes Ok: [{:#x},{:#x}], size:{:#x}", 
                address_align, 
                end_addr,
                size_need
            );
            self.byte_index += size_need;
            self.byte_used += size_need;
            Ok(ptr)
        }
        else {
            warn!("Allocate bytes failed:NoMemory");
            Err(AllocError::NoMemory)
        }
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        debug!(
            "Deallocate bytes Ok: start:{:#x}, size:{:#x}", 
            pos.as_ptr() as usize, 
            layout.size()
        );
        self.byte_used -= layout.size();
        if self.byte_used == 0 {
            self.byte_index = 0
        }
    }

    fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    fn used_bytes(&self) -> usize {
        self.byte_used
    }

    fn available_bytes(&self) -> usize {
        self.page_index - self.byte_index
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if align_pow2 % PAGE_SIZE != 0 {
            warn!("Allocate pages failed:InvalidParam");
            return Err(AllocError::InvalidParam);
        }
        let align_pow2 = align_pow2 / PAGE_SIZE;
        if !align_pow2.is_power_of_two() {
            warn!("Allocate pages failed:InvalidParam");
            return Err(AllocError::InvalidParam);
        }
        let res = self.page_used + num_pages % align_pow2;
        // let alloc_num = num_pages.max(align_pow2);
        let alloc_num = if res == 0 {num_pages} else {align_pow2 - res + num_pages};
        if self.available_pages() >= alloc_num {
            // update page_used and page_index
            self.page_used += alloc_num;
            self.page_index -= alloc_num * PAGE_SIZE;
            debug!(
                "Allocate Pages Ok: start:{:#x}, page_num:{:#x}", 
                self.start_va + self.page_index, 
                alloc_num
            );
            Ok(self.start_va + self.page_index)
        }
        else {
            warn!("Allocate pages failed: NoMemory");
            Err(AllocError::NoMemory)
        }
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        // TODO: not decrease `used_pages` if deallocation failed
        return;
    }

    fn total_pages(&self) -> usize {
        self.total_bytes / PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        self.page_used
    }

    fn available_pages(&self) -> usize {
        (self.page_index - self.byte_index) / PAGE_SIZE
    }
}