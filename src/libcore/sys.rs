#[doc = "Misc low level stuff"];

export type_desc;
export get_type_desc;
export size_of;
export min_align_of;
export pref_align_of;
export refcount;
export log_str;
export lock_and_signal, condition, methods;

enum type_desc = {
    first_param: **libc::c_int,
    size: libc::size_t,
    align: libc::size_t
    // Remaining fields not listed
};

type rust_cond_lock = *libc::c_void;

#[abi = "cdecl"]
native mod rustrt {
    pure fn refcount(t: *()) -> libc::intptr_t;
    fn unsupervise();
    pure fn shape_log_str(t: *sys::type_desc, data: *()) -> str;

    fn rust_create_cond_lock() -> rust_cond_lock;
    fn rust_destroy_cond_lock(lock: rust_cond_lock);
    fn rust_lock_cond_lock(lock: rust_cond_lock);
    fn rust_unlock_cond_lock(lock: rust_cond_lock);
    fn rust_wait_cond_lock(lock: rust_cond_lock);
    fn rust_signal_cond_lock(lock: rust_cond_lock) -> bool;
}

#[abi = "rust-intrinsic"]
native mod rusti {
    fn get_tydesc<T>() -> *();
    fn size_of<T>() -> uint;
    fn pref_align_of<T>() -> uint;
    fn min_align_of<T>() -> uint;
}

#[doc = "
Returns a pointer to a type descriptor.

Useful for calling certain function in the Rust runtime or otherwise
performing dark magick.
"]
pure fn get_type_desc<T>() -> *type_desc {
    unchecked { rusti::get_tydesc::<T>() as *type_desc }
}

#[doc = "Returns the size of a type"]
pure fn size_of<T>() -> uint unsafe {
    unchecked { rusti::size_of::<T>() }
}

#[doc = "
Returns the ABI-required minimum alignment of a type

This is the alignment used for struct fields. It may be smaller
than the preferred alignment.
"]
pure fn min_align_of<T>() -> uint unsafe {
    unchecked { rusti::min_align_of::<T>() }
}

#[doc = "Returns the preferred alignment of a type"]
pure fn pref_align_of<T>() -> uint unsafe {
    unchecked { rusti::pref_align_of::<T>() }
}

#[doc = "Returns the refcount of a shared box"]
pure fn refcount<T>(t: @T) -> uint {
    unsafe {
        ret rustrt::refcount(unsafe::reinterpret_cast(t)) as uint;
    }
}

pure fn log_str<T>(t: T) -> str {
    unsafe {
        let data_ptr: *() = unsafe::reinterpret_cast(ptr::addr_of(t));
        rustrt::shape_log_str(get_type_desc::<T>(), data_ptr)
    }
}

resource lock_and_signal(lock: rust_cond_lock) {
    rustrt::rust_destroy_cond_lock(lock);
}

enum condition {
    condition_(rust_cond_lock)
}

resource unlock(lock: rust_cond_lock) {
    rustrt::rust_unlock_cond_lock(lock);
}

fn create_lock() -> lock_and_signal {
    lock_and_signal(rustrt::rust_create_cond_lock())
}

impl methods for lock_and_signal {
    fn lock<T>(f: fn() -> T) -> T {
        rustrt::rust_lock_cond_lock(*self);
        let _r = unlock(*self);
        f()
    }

    fn lock_cond<T>(f: fn(condition) -> T) -> T {
        rustrt::rust_lock_cond_lock(*self);
        let _r = unlock(*self);
        f(condition_(*self))
    }
}

impl methods for condition {
    fn wait() {
        rustrt::rust_wait_cond_lock(*self);
    }

    fn signal() -> bool {
        rustrt::rust_signal_cond_lock(*self)
    }
}

#[cfg(test)]
mod tests {
    use std;
    import std::arc;

    #[test]
    fn size_of_basic() {
        assert size_of::<u8>() == 1u;
        assert size_of::<u16>() == 2u;
        assert size_of::<u32>() == 4u;
        assert size_of::<u64>() == 8u;
    }

    #[test]
    #[cfg(target_arch = "x86")]
    #[cfg(target_arch = "arm")]
    fn size_of_32() {
        assert size_of::<uint>() == 4u;
        assert size_of::<*uint>() == 4u;
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn size_of_64() {
        assert size_of::<uint>() == 8u;
        assert size_of::<*uint>() == 8u;
    }

    #[test]
    fn align_of_basic() {
        assert pref_align_of::<u8>() == 1u;
        assert pref_align_of::<u16>() == 2u;
        assert pref_align_of::<u32>() == 4u;
    }

    #[test]
    #[cfg(target_arch = "x86")]
    #[cfg(target_arch = "arm")]
    fn align_of_32() {
        assert pref_align_of::<uint>() == 4u;
        assert pref_align_of::<*uint>() == 4u;
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn align_of_64() {
        assert pref_align_of::<uint>() == 8u;
        assert pref_align_of::<*uint>() == 8u;
    }

    #[test]
    fn condition_variable() {
        let lock = arc::arc(create_lock());
        let lock2 = arc::clone(&lock);

        task::spawn {|move lock2|
            let lock = arc::get(&lock2);
            (*lock).lock_cond {|c|
                c.wait();
            }
        }

        let mut signaled = false;
        while !signaled {
            (*arc::get(&lock)).lock_cond {|c|
                signaled = c.signal()
            }
        }
    }
}

// Local Variables:
// mode: rust;
// fill-column: 78;
// indent-tabs-mode: nil
// c-basic-offset: 4
// buffer-file-coding-system: utf-8-unix
// End:
