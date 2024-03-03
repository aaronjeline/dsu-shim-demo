use std::fs;
use std::cell::RefCell;
use std::time::SystemTime;
use common::ControlFlow;
mod util;
use util::*;

const PATH : &str = "/data/runner/target/release/librunner.so";

thread_local! { 
    static LAST_MODIFIED : RefCell<SystemTime> = RefCell::new(SystemTime::now());
}

type UpdateHook = extern "C" fn() -> ControlFlow;


fn main() {
    set_last_updated();
    let mut lib =  unsafe { Library::new(PATH) }.unwrap();
    let mut entry : Symbol<UpdateHook, ControlFlow> = unsafe { lib.get("entrypoint") }.unwrap();
    loop {
        match entry.call(check_update) { 
            ControlFlow::Continue => break,
            ControlFlow::Break => { 
                drop(entry);
                lib.unload().unwrap();
                lib = unsafe { Library::new(PATH) }.unwrap();
                entry = unsafe { lib.get("entrypoint") }.unwrap();
            }
        };
    }
}


fn set_last_updated() -> (SystemTime, SystemTime) {
    let time = fs::metadata(PATH).unwrap().modified().unwrap();
    (LAST_MODIFIED.with(|cell| cell.replace(time)), time)
}

fn new_version_available() -> bool { 
    let (last, current) = set_last_updated();
    last < current  
}


extern "C" fn check_update() -> ControlFlow { 
    if new_version_available() { 
        ControlFlow::Break
    } else { 
        ControlFlow::Continue
    }
}
