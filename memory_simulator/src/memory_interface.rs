use libloading::{Library, Symbol};
use std::error::Error;
use std::ffi::{c_char, c_float, c_longlong, c_void, CString};

#[repr(C)]
pub struct Request {
    pub addr: i64,
    pub addr_vec: Vec<i32>,
    pub type_id: i32,
    pub source_id: i32,
    pub command: i32,
    pub final_command: i32,
    pub is_stat_updated: bool,
    pub arrive: i64,
    pub depart: i64,
    pub scratchpad: [i32; 4],
    pub callback: Option<extern "C" fn(*mut Request)>,
    pub m_payload: *mut c_void,
}

type MyWrapper = *mut c_void;
type RequestCallback = extern "C" fn(*mut Request, *mut c_void);

pub struct MemoryInterface<'lib> {
    lib: &'lib Library,
    wrapper: MyWrapper,

    dram_init: Symbol<'lib, unsafe extern "C" fn(MyWrapper, *const c_char)>,
    send_request: Symbol<
        'lib,
        unsafe extern "C" fn(MyWrapper, i64, bool, RequestCallback, *mut c_void) -> bool,
    >,
    frontend_tick: Symbol<'lib, unsafe extern "C" fn(MyWrapper)>,
    memory_system_tick: Symbol<'lib, unsafe extern "C" fn(MyWrapper)>,
    dram_delete: Symbol<'lib, unsafe extern "C" fn(MyWrapper)>,
    MyWrapper_finish: Symbol<'lib, unsafe extern "C" fn(MyWrapper)>,
}

impl<'lib> MemoryInterface<'lib> {
    pub unsafe fn new(lib: &'lib Library) -> Result<Self, Box<dyn Error>> {
        let dram_new: Symbol<unsafe extern "C" fn() -> MyWrapper> = lib.get(b"dram_new")?;
        let wrapper = dram_new();

        Ok(Self {
            lib,
            wrapper,
            dram_init: lib.get(b"dram_init")?,
            send_request: lib.get(b"send_request")?,
            frontend_tick: lib.get(b"frontend_tick")?,
            memory_system_tick: lib.get(b"memory_system_tick")?,
            dram_delete: lib.get(b"dram_delete")?,
            MyWrapper_finish: lib.get(b"MyWrapper_finish")?,
        })
    }

    pub unsafe fn init(&self, config_path: &str) {
        let c_path = CString::new(config_path).unwrap();
        (self.dram_init)(self.wrapper, c_path.as_ptr());
    }

    pub unsafe fn frontend_tick(&self) {
        (self.frontend_tick)(self.wrapper);
    }

    pub unsafe fn memory_tick(&self) {
        (self.memory_system_tick)(self.wrapper);
    }

    pub unsafe fn send_request(
        &self,
        addr: i64,
        is_write: bool,
        callback: RequestCallback,
        ctx: *mut c_void,
    ) -> bool {
        (self.send_request)(self.wrapper, addr, is_write, callback, ctx)
    }

    pub unsafe fn finish(&self) {
        (self.MyWrapper_finish)(self.wrapper);
    }
}

impl<'lib> Drop for MemoryInterface<'lib> {
    fn drop(&mut self) {
        unsafe {
            (self.dram_delete)(self.wrapper);
        }
    }
}
