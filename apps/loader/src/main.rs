#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

#[cfg(feature = "axstd")]
use axstd::println;
const PLASH_START: usize = 0x22000000;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let plash_ptr = PLASH_START as *const u64;
    let app_num = unsafe { plash_ptr.read_volatile() };
    println!("Load payload, app num: {} ...", app_num);
    let mut plash_ptr = unsafe { plash_ptr.add(1) };

    // app running aspace
    // SBI(0x80000000) -> App <- Kernel(0x80200000)
    // 0xffff_ffc0_0000_0000
    const RUN_START: usize = 0xffff_ffc0_8010_0000;

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
        // execute app
        unsafe { core::arch::asm!("
           li     t2, {run_start}
           jalr   t2", 
           run_start = const RUN_START,
        )}
        println!("Load payload ok!");
    }
}
