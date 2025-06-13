use super::runtime::*;
use super::simulator::Simulator;
use crate::memory_interface::MemoryInterface;
use crate::memory_interface::Request;
use libloading::{Library, Symbol};
use num_bigint::{BigInt, BigUint};
use std::collections::VecDeque;
use std::ffi::CString;
use std::ffi::{c_char, c_float, c_longlong, c_void};
use std::sync::Arc;

extern "C" fn rust_callback(req: *mut Request, ctx: *mut c_void) {
    unsafe {
        //println!("we successfully push data!!!!!!");
        let req = &*req;
        let sim: &mut Simulator = &mut *(ctx as *mut Simulator);
        // skip the write request.
        let cycles = (req.depart - req.arrive) as usize;
        let stamp = sim.stamp;
        //println!("Request arrived at {}", stamp + 50 + 100 * cycles);
        println!("req address is: {}, the data is: {}, and the latency is: {}", req.addr, sim.array_2aa85.payload[req.addr as usize], cyclize(stamp + 50 + 100 * cycles));
        sim.MemUser_57a15_rdata.push.push(FIFOPush::new(
            stamp + 50 + 100 * cycles,
            sim.array_2aa85.payload[req.addr as usize].clone(),
            "SRAM_2a9ed",
        ));
        //sim.print_rdata_state();
    }
}

// Elaborating module MemUser_57a15
pub fn MemUser_57a15(sim: &mut Simulator) -> bool {
    // but we do not know whether the rdata has value, so we need to check it.
    let _27689 = {
        let stamp = sim.stamp - sim.stamp % 100 + 50; // 150
        sim.MemUser_57a15_rdata
            .pop
            .push(FIFOPop::new(stamp, "MemUser_57a15"));

        // Add safe unwrapping with default value or early return
        match sim.MemUser_57a15_rdata.payload.front() {
            Some(value) => value.clone(),
            None => return false, // No data available yet
        }
    };

    let _2a915 = ValueCastTo::<i32>::cast(&_27689);
    let _2a921 = ValueCastTo::<i32>::cast(&_2a915) + ValueCastTo::<i32>::cast(&128i32);

    print!(
        "@line:{:<5} {:<10}: [MemUser_57a15]\t",
        line!(),
        cyclize(sim.stamp)
    );
    println!("{} + {} = {}", _2a915, 128i32, _2a921);

    true
}
// Elaborating module Driver
pub fn Driver(sim: &mut Simulator) -> bool {
    // read the cycle
    let _2a961 = { sim.array_2a959.payload[0u8 as usize].clone() };
    let _2a969 = {
        {
            let a = ValueCastTo::<u64>::cast(&_2a961);
            let mask = u64::from_str_radix("1", 2).unwrap();
            let res = (a >> 0) & mask;
            ValueCastTo::<bool>::cast(&res)
        }
    };
    sim._2a969_value = Some(_2a969.clone());
    let _2a991 = { !_2a969 };
    sim._2a991_value = Some(_2a991.clone());
    let _2a9b5 = { ValueCastTo::<i32>::cast(&_2a961) + ValueCastTo::<i32>::cast(&1i32) };
    let _2a9cd = {
        {
            let a = ValueCastTo::<u64>::cast(&_2a9b5);
            let mask = u64::from_str_radix("111111111", 2).unwrap();
            let res = (a >> 0) & mask;
            ValueCastTo::<u16>::cast(&res)
        }
    };
    let _2a9f5 = {
        {
            let a = ValueCastTo::<u64>::cast(&_2a961);
            let mask = u64::from_str_radix("111111111", 2).unwrap();
            let res = (a >> 0) & mask;
            ValueCastTo::<u16>::cast(&res)
        }
    };
    let _2aa21 = {
        if _2a969 {
            _2a9cd
        } else {
            _2a9f5
        }
    };
    let _2aa41 = { ValueCastTo::<i16>::cast(&_2aa21) };
    sim._2aa41_value = Some(_2aa41.clone());
    {
        let stamp = sim.stamp - sim.stamp % 100 + 50;
        // push to the cycle.
        sim.array_2a959.write.push(ArrayWrite::new(
            stamp,
            false as usize,
            _2a9b5.clone(),
            "Driver",
        ));
    };
    //read enable
    let _2aa91 = { ValueCastTo::<u32>::cast(&_2a961) };
    sim._2aa91_value = Some(_2aa91.clone());
    if _2a991 {
        {
            let stamp = sim.stamp - sim.stamp % 100 + 100;
            sim.MemUser_57a15_event.push_back(stamp)
        };
    }

    true
}
// Elaborating module SRAM_2a9ed
pub fn SRAM_2a9ed(sim: &mut Simulator) -> bool {
    //println!("go inside the SRAM_2a9ed module");
    //that's the write
    // if if let Some(x) = &sim._2a969_value {
    //                         x
    //                       } else {
    //                         panic!("Value _2a969 invalid!");
    //                       }.clone() {
    //     {
    //               let stamp = sim.stamp - sim.stamp % 100 + 50;
    //               sim.array_2aa85.write.push(
    //                 ArrayWrite::new(stamp, if let Some(x) = &sim._2aa41_value {
    //                         x
    //                       } else {
    //                         panic!("Value _2aa41 invalid!");
    //                       }.clone() as usize, if let Some(x) = &sim._2aa91_value {
    //                         x
    //                       } else {
    //                         panic!("Value _2aa91 invalid!");
    //                       }.clone().clone(), "SRAM_2a9ed"));
    //             };
    //   }
    // that's the read
    // if if let Some(x) = &sim._2a991_value {
    //                         x
    //                       } else {
    //                         panic!("Value _2a991 invalid!");
    //                       }.clone() {
    //     let _2ab35 = { sim.array_2aa85.payload[if let Some(x) = &sim._2aa41_value {
    //                         x
    //                       } else {
    //                         panic!("Value _2aa41 invalid!");
    //                       }.clone() as usize].clone() };
    //     {
    //               let stamp = sim.stamp;
    //               sim.MemUser_57a15_rdata.push.push(
    //                 FIFOPush::new(stamp + 50, _2ab35.clone(), "SRAM_2a9ed"));
    //             };
    //     ();
    //   }

    // write.
    if let Some(write_enable) = sim._2a969_value {
        //println!("write_enable: {}", write_enable);
        if write_enable {
            let addr = match sim._2aa41_value {
                Some(addr) => addr as i64,
                None => return false,
            };

            let data = match sim._2aa91_value {
                Some(data) => data,
                None => return false,
            };

            unsafe {
                let mem_interface = Arc::clone(&sim.mem_interface);
                let success = mem_interface.as_ref().send_request(
                    addr,
                    true, // is_write = true
                    rust_callback,
                    sim as *mut Simulator as *mut c_void,
                );

                if success {
                    let stamp = sim.stamp - sim.stamp % 100 + 50;
                    // write to the array
                    sim.array_2aa85.write.push(ArrayWrite::new(
                        stamp,
                        addr as usize,
                        data,
                        "SRAM_2a9ed",
                    ));
                    println!("Requesting write to address: {}, data: {}", addr, data);
                } else {
                    sim.stamp = sim.stamp - sim.stamp % 100 + 50;
                    return false;
                }
            }
        }
    }

    // Read operation
    if let Some(read_enable) = sim._2a991_value {
        //println!("read_enable: {}", read_enable);
        if read_enable {
            let addr = match sim._2aa41_value {
                Some(addr) => addr as i64,
                None => return false,
            };

            unsafe {
                let mem_interface = Arc::clone(&sim.mem_interface);
                println!("Requesting read from address: {}", addr);
                let success = mem_interface.as_ref().send_request(
                    addr,
                    false, // is_write = false
                    rust_callback,
                    sim as *mut Simulator as *mut c_void,
                );
                if !success {
                    return false;
                }
            }
        }
    }

    true
}
