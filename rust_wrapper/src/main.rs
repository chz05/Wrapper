use libloading::{Library, Symbol};
use std::error::Error;
use std::ffi::CString;
use std::ffi::{c_char, c_float, c_longlong, c_void};
mod memory_interface;
use memory_interface::{MemoryInterface, Request};


pub struct counter {
    pub value: i64,
}

extern "C" fn rust_callback(req: *mut Request, ctx: *mut c_void) {
    unsafe {
        let req = &*req;
        let count: &mut counter = &mut *(ctx as *mut counter);
        println!("Callback called! Addr = {:#x}, counter = {}, latency = {}", req.addr, count.value, req.depart - req.arrive);
        count.value += 1;
    }
}

fn main() -> Result<(), Box<dyn Error>>{
    unsafe {
        let lib = Library::new("/root/wrapper/rust_wrapper/src/libwrapper.so")?;
        let mem = MemoryInterface::new(&lib)?;
        mem.init("/root/wrapper/configs/example_config.yaml");

        let mut count = counter { value: 0 };
        let count_ptr = &mut count as *mut _ as *mut std::ffi::c_void;

        for i in 0..100 {
            let addr = 0x1000 + i * 64;
            let a = mem.send_request(addr, false, rust_callback, count_ptr);
            if a {
                 println!("Request sent for address {:#x}, success: {}", addr, a);
            }
           
            mem.frontend_tick();
            mem.memory_tick();
        }

        mem.finish();
    }
    println!("all good!");
    Ok(())
}
