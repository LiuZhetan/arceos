#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

#[cfg(feature = "axstd")]
use axstd::println;

#[cfg(feature = "axstd")]
use axstd::ax_terminate;

use riscv::register::satp;

const PLASH_START: usize = 0x22000000;

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_TERMINATE: usize = 3;
static mut ABI_TABLE: [usize; 16] = [0; 16];

fn register_abi(num: usize, handle: usize) {
    unsafe { ABI_TABLE[num] = handle; }
}

fn abi_hello() {
    println!("[ABI:Hello] Hello, Apps!");
}

fn abi_putchar(c: char) {
    println!("[ABI:Print] {c}");
}

#[cfg(feature = "axstd")]
fn abi_terminate() {
    ax_terminate();
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    // register abi
    register_abi(SYS_HELLO, abi_hello as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);
    #[cfg(feature = "axstd")]
    register_abi(SYS_TERMINATE, abi_terminate as usize);

    let plash_ptr = PLASH_START as *const u64;
    let app_num = unsafe { plash_ptr.read_volatile() };
    println!("Load payload, app num: {} ...", app_num);
    let mut plash_ptr = unsafe { plash_ptr.add(1) };

    // app running aspace
    // SBI(0x80000000) -> App <- Kernel(0x80200000)
    // 0xffff_ffc0_0000_0000
    const RUN_START: usize = 0xffff_ffc0_8010_0000;
    const APP_RUN_START: usize = 0x4010_0000;

    for _ in 0..app_num {
        let app_size = unsafe { plash_ptr.read_volatile() }; 
        let app_start = unsafe { plash_ptr.add(1) };
        let load_code = unsafe { core::slice::from_raw_parts(app_start as *const u8, app_size as usize) };
        // println!("content: {:?}: ", load_code);
        plash_ptr = unsafe { (app_start as *const u8).add(app_size as usize) } as *const u64;

        let run_code = unsafe {
            core::slice::from_raw_parts_mut(RUN_START as *mut u8, app_size as usize)
        };
        run_code.copy_from_slice(load_code);
        println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());

        println!("Execute app ...");
        println!("ABI_TABLE addr:{:#x}",unsafe {
            ABI_TABLE.as_ptr() as usize
        });

        // switch aspace from kernel to app
        unsafe { init_app_page_table(); }
        unsafe { switch_app_aspace(); }
        // execute app
        unsafe { core::arch::asm!("
            la     a7, {abi_table}
            li     t2, {run_start}
            jalr   t2", 
            run_start = const APP_RUN_START,
            abi_table = sym ABI_TABLE,
        )}
        println!("Load payload ok!");
    }

    /*
    // run exercise4
    println!("Execute app ...");
    let arg0: u8 = b'A';

    unsafe { core::arch::asm!("
       li     t0, {abi_num}
       slli   t0, t0, 3
       la     t1, {abi_table}
       add     t2, t1, t0
       ld     t2, (t2)
       jalr   t2
       li     t3, {run_start}
       jalr   t3
       li     t0, {terminate_num}
       slli   t0, t0, 3
       add     t2, t1, t0
       ld     t2, (t2)
       jalr   t2
       li     t3, {run_start}
       jalr   t3",
        run_start = const RUN_START,
        abi_table = sym ABI_TABLE,
        //abi_num = const SYS_HELLO,
        abi_num = const SYS_PUTCHAR,
        terminate_num = const SYS_TERMINATE,
        in("a0") arg0,
   )}*/
}

//
// App aspace
//
#[link_section = ".data.app_page_table"]
static mut APP_PT_SV39: [u64; 512] = [0; 512];

unsafe fn init_app_page_table() {
    // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[2] = (0x80000 << 10) | 0xef;
    // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[0x102] = (0x80000 << 10) | 0xef;
    // 0x0000_0000..0x4000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[0] = (0x00000 << 10) | 0xef;
    // For App aspace!
    // 0x4000_0000..0x8000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[1] = (0x80000 << 10) | 0xef;
}

unsafe fn switch_app_aspace() {
    let page_table_root = APP_PT_SV39.as_ptr() as usize -
    axconfig::PHYS_VIRT_OFFSET;
    satp::set(satp::Mode::Sv39, 0, page_table_root >> 12);
    
    riscv::asm::sfence_vma_all();
}

