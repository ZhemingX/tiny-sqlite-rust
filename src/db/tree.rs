use crate::service::Row;

macro_rules! field_size {
    ($t:ident :: $field:ident) => {{
        let m = core::mem::MaybeUninit::<$t>::uninit();
        let p = unsafe {
            core::ptr::addr_of!((*(&m as *const _ as *const $t)).$field)
        };

        const fn size_of_raw<T>(_: *const T) -> usize {
            core::mem::size_of::<T>()
        }
        size_of_raw(p)
    }};
}

const ID_SIZE: usize = field_size!(Row::id);
const USERNAME_SIZE: usize = field_size!(Row::username);
const EMAIL_SIZE: usize = field_size!(Row::email);

