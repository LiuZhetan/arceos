#![no_std]

extern crate dtb;
extern crate log;

use dtb::Reader;
use log::debug;

fn bytes_to_num(buffer: &[u8]) -> usize {
    let mut res = 0usize;
    for i in buffer {
        res = res | *i as usize;
        res = res << 8;
    }
    res
}

const MMIO_MAX:usize = 128;

pub struct DtbInfo {
    pub memory_addr: usize,
    pub memory_size: usize,
    pub mmio_regions: [(usize,usize);MMIO_MAX],
    pub mmio_num:usize,
}

pub fn parse_dtb(address:usize) -> Result<DtbInfo, &'static str>{
    let reader = unsafe { Reader::read_from_address(address).unwrap() };
    /*for entry in reader.reserved_mem_entries() {
        debug!("reserved: {:?} bytes at {:?}", entry.size, entry.address);
    }
    let mut indent = 0;
    for entry in reader.struct_items() {
        match entry {
            StructItem::BeginNode { name } => {
                debug!("{:indent$}{} {{", "", name, indent = indent);
                indent += 2;
            }
            StructItem::EndNode => {
                indent -= 2;
                debug!("{:indent$}}}", "", indent = indent);
            }
            StructItem::Property { name, value } => {
                debug!("{:indent$}{}: {:?}", "", name, value, indent = indent)
            }
        }
    }*/
    let root = reader.struct_items();
    let (prop, _) = 
        root.path_struct_items("/memory/reg").next().unwrap();
    let value = prop.value().unwrap();
    let memory_addr = bytes_to_num(&value[..value.len()/2]);
    let memory_size = bytes_to_num(&value[value.len()/2..]);
    debug!("Memory from {:x} to {:x}", memory_addr, memory_addr + memory_size);
    
    let mut mmio_regions = [(0,0);MMIO_MAX];
    let mut mmio_num = 0;
    for (prop, prop_list) in root.path_struct_items("/soc/virtio_mmio") {
        debug!("/soc/virtio_mmio {:?}", prop.name());
        let (reg, _) = prop_list.path_struct_items("reg").next().unwrap();
        let reg_value = reg.value().unwrap();
        let mmio_start = bytes_to_num(&reg_value[..reg_value.len()/2]);
        let mmio_len = bytes_to_num(&reg_value[reg_value.len()/2..]);
        debug!("{} start: {:x} size: {:x}", prop.name().unwrap(), mmio_start, mmio_len);
        mmio_regions[mmio_num] = (mmio_start, mmio_len);
        mmio_num += 1;
    }

    Ok(DtbInfo {
        memory_addr,
        memory_size,
        mmio_regions,
        mmio_num,
    })
}