use super::runtime::*;
use crate::memory_interface::MemoryInterface;
use crate::memory_interface::Request;
use libloading::Library;
use num_bigint::{BigInt, BigUint};
use rand::seq::SliceRandom;
use std::collections::VecDeque;
use std::sync::Arc;

pub struct Simulator {
    pub stamp: usize,
    pub array_2a959: Array<i32>, 
    pub array_2aa85: Array<u32>,
    pub MemUser_57a15_triggered: bool,
    pub MemUser_57a15_event: VecDeque<usize>,
    pub MemUser_57a15_rdata: FIFO<u32>,
    pub Driver_triggered: bool,
    pub Driver_event: VecDeque<usize>,
    pub SRAM_2a9ed_triggered: bool,
    pub _2a969_value: Option<bool>,
    pub _2aa41_value: Option<i16>,
    pub _2a991_value: Option<bool>,
    pub _2aa91_value: Option<u32>,
    pub mem_interface: Arc<MemoryInterface<'static>>,
    _lib: Arc<Library>,
}

impl Simulator {
    pub fn new() -> Self {
        let lib = Arc::new(
            unsafe { Library::new("/tmp/memory_simulator/src/libwrapper.so") }
                .expect("Failed to load library"),
        );
        let lib_ref = lib.clone();

        let mem = unsafe {
            let lib_static = Box::leak(Box::new(lib.clone()));

            Arc::new(MemoryInterface::new(&*lib_static).expect("Failed to create MemoryInterface"))
        };
        Simulator {
            stamp: 0,
            array_2a959: Array::new(1), // cycle
            array_2aa85: Array::new(512),  // sram size
            MemUser_57a15_triggered: false,
            MemUser_57a15_event: VecDeque::new(),
            MemUser_57a15_rdata: FIFO::new(), // read data
            Driver_triggered: false,
            Driver_event: VecDeque::new(),
            SRAM_2a9ed_triggered: false,
            _2a969_value: None,
            _2aa41_value: None,
            _2a991_value: None,
            _2aa91_value: None,
            mem_interface: mem,
            _lib: lib,
        }
    }

    pub fn print_rdata_state(&mut self) {
        println!("\n=== MemUser_57a15_rdata State at {} ===", cyclize(self.stamp));
        
        // Print payload
        println!("Payload Queue:");
        if self.MemUser_57a15_rdata.payload.is_empty() {
            println!("  [Empty]");
        } else {
            for (i, value) in self.MemUser_57a15_rdata.payload.iter().enumerate() {
                println!("  [{:2}]: {:?}", i, value);
            }
        }

        // Print push queue (XEQ<FIFOPush>)
        println!("\nPush Queue (XEQ):");
        if self.MemUser_57a15_rdata.push.is_empty() {
            println!("  [Empty]");
        } else {
            // Print each push event's cycle and pusher
            for (cycle, event) in self.MemUser_57a15_rdata.push.q.iter() {
                println!("  Cycle: {}", cyclize(*cycle));
            }
        }

        // Print pop queue (XEQ<FIFOPop>)
        println!("\nPop Queue (XEQ):");
        if self.MemUser_57a15_rdata.pop.is_empty() {
            println!("  [Empty]");
        } else {
            // Print each pop event's cycle and pusher
            for (cycle, event) in self.MemUser_57a15_rdata.pop.q.iter() {
                println!("  Cycle: {}", cyclize(*cycle));
            }
        }
        println!("=====================================\n");
    }

    fn event_valid(&self, event: &VecDeque<usize>) -> bool {
        event.front().map_or(false, |x| *x <= self.stamp)
    }

    pub fn reset_downstream(&mut self) {
        self.MemUser_57a15_triggered = false;
        self.Driver_triggered = false;
        self.SRAM_2a9ed_triggered = false;
        self._2a969_value = None;
        self._2aa41_value = None;
        self._2a991_value = None;
        self._2aa91_value = None;
    }

    pub fn tick_registers(&mut self) {
        // self.array_2a959.tick(self.stamp); // the cycle
        // self.array_2aa85.tick(self.stamp); // the SRAM
        // self.MemUser_57a15_rdata.tick(self.stamp); // FIFO
        // Pre-check FIFO state before ticking
        // if !self.MemUser_57a15_rdata.payload.is_empty() || 
        //    !self.MemUser_57a15_rdata.push.is_empty() {
        //     self.MemUser_57a15_rdata.tick(self.stamp);
        // }
        
        // // Only tick arrays if they have pending writes
        
        self.array_2a959.tick(self.stamp);
        
        //we tick that one into the payload when the write is not empty.
        if !self.array_2aa85.write.is_empty() {
            //println!("write is not empty, put the value into payload at cycle:{}", (self.stamp - self.stamp % 100)/100);
            self.array_2aa85.tick(self.stamp);
            println!("now write finish his job.")
        }
        // Reset the write queues after ticking
        if !self.MemUser_57a15_rdata.push.is_empty() &&    
        !self.MemUser_57a15_rdata.pop.is_empty() {
            //println!("looks like u never reach here");
         self.MemUser_57a15_rdata.tick(self.stamp);
     }
    }

    // simulate the memory user, 
    fn simulate_MemUser_57a15(&mut self) {
        //println!("Simulating MemUser_57a15 at cycle {}", self.stamp);
        if self.event_valid(&self.MemUser_57a15_event) {
            let succ = super::modules::MemUser_57a15(self);
            //println!("memuser success or not: {}", succ);
            if succ {
                self.MemUser_57a15_event.pop_front();
            } else {
            }
            self.MemUser_57a15_triggered = succ;
        } // close event condition
    } // close function

    fn simulate_Driver(&mut self) {
        //println!("Simulating Driver at cycle {}", self.stamp);
        if self.event_valid(&self.Driver_event) {
            //println!("Driver event at cycle {}", self.stamp);
            let succ = super::modules::Driver(self);
            if succ {
                self.Driver_event.pop_front();
            } else {
                self._2a969_value = None;
                self._2aa41_value = None;
                self._2a991_value = None;
                self._2aa91_value = None;
            }
            self.Driver_triggered = succ;
        } // close event condition
    } // close function

    fn simulate_SRAM_2a9ed(&mut self) {
        //println!("Simulating SRAM_2a9ed at cycle {}", self.stamp);
        if self.Driver_triggered {
            let succ = super::modules::SRAM_2a9ed(self);
            self.SRAM_2a9ed_triggered = succ;
        } // close event condition
    } // close function
}

pub fn simulate() {
    unsafe {
        let mut sim = Simulator::new();
        unsafe {
            sim.mem_interface
                .init("/tmp/memory_simulator/config/example_config.yaml");
        }
        let simulators: Vec<fn(&mut Simulator)> = vec![
            Simulator::simulate_MemUser_57a15,
            Simulator::simulate_Driver,
        ];
        let downstreams: Vec<fn(&mut Simulator)> = vec![Simulator::simulate_SRAM_2a9ed];

        for i in 1..=200 {
            sim.Driver_event.push_back(i * 100); // stamp is 100 to 20000, totally 200 cycles
        }
        let mut idle_count = 0;
        for i in 1..=200 {
            sim.stamp = i * 100; // starts from the cycle one.
            sim.reset_downstream();

            for simulate in simulators.iter() {
                simulate(&mut sim);
            }

            for simulate in downstreams.iter() {
                simulate(&mut sim);
            }

            let any_module_triggered = sim.MemUser_57a15_triggered || sim.Driver_triggered;

            // Handle idle threshold
            if !any_module_triggered {
                idle_count += 1;
                if idle_count >= 200 {
                    println!("Simulation stopped due to reaching idle threshold of 200");
                    break;
                }
            } else {
                idle_count = 0;
            }

            sim.stamp += 50;
            sim.tick_registers();
            unsafe {
                sim.mem_interface.frontend_tick();
                sim.mem_interface.memory_tick();
            }
            //sim.print_rdata_state();
        }
    }
}
