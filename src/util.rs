use thiserror::Error;
use std::ffi::CStr;
use std::borrow::Cow;
use libc::*;
use std::marker::PhantomData;
use std::ffi::CString;
use std::mem::transmute;

#[derive(Debug,Clone,Error)]
pub enum Error { 
    #[error("{0}")]
    Msg(Cow<'static, str>),
    #[error("An error condition was reported but nothing from `dlerror`")]
    NoMsg
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Library {
    handle : *mut c_void
}

impl Library { 
    pub unsafe fn new(filename : impl AsRef<str>) -> Result<Self> {
        let str = CString::new(filename.as_ref()).unwrap();
        let ptr = dlopen(str.as_ptr(), RTLD_LAZY | RTLD_LOCAL);
        if ptr.is_null() { 
            Err(find_dlerror())
        } else { 
            Ok(Self { handle : ptr })
        }
    }

    pub unsafe fn get<I, O>(&'_ self, name : impl AsRef<str>) -> Result<Symbol<'_, I, O>> {
        let string = CString::new(name.as_ref()).unwrap();
        let result = dlsym(self.handle, string.as_ptr());
        if result.is_null() { 
            Err(find_dlerror())
        } else { 
            Ok(Symbol::new(self, transmute(result)))
        }
    }

    pub fn unload(self) -> Result<()> { 
        unsafe { self.borrowed_unload() }
    }

    unsafe fn borrowed_unload(&self) -> Result<()> { 
        let result = dlclose(self.handle);
        if result == -1 { 
            Err(find_dlerror())
        } else { 
            Ok(())
        }
    }

}

impl std::ops::Drop for Library { 
    fn drop(&mut self) {
        unsafe { 
            let _ = self.borrowed_unload();
        }
    }

}

pub struct Symbol<'a, I, O> { 
    func : extern "C" fn(I) -> O,
    owner : PhantomData<&'a ()>,
}

impl<'a, I, O> Symbol<'a,I, O> { 
    fn new(_lib : &'a Library, func : extern "C" fn(I) -> O) -> Self { 
        Self { 
            func,
            owner : PhantomData
        }
    }

    pub fn call(&self, i : I) -> O { 
        (self.func)(i)
    }

}

    


fn find_dlerror() -> Error { 
    let cptr = unsafe { dlerror() };
    if cptr.is_null() {
        Error::NoMsg
    } else {
        let cstr = unsafe { CStr::from_ptr(cptr) };
        Error::Msg(cstr.to_string_lossy())
    }
}
