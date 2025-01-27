//! Example Rust-based BPF program that tests call depth and stack usage

use safecoin_program::{custom_panic_default, entrypoint::SUCCESS, log::sol_log_64, msg};

#[inline(never)]
pub fn recurse(data: &mut [u8]) {
    if data.len() <= 1 {
        return;
    }
    recurse(&mut data[1..]);
    sol_log_64(line!() as u64, 0, 0, 0, data[0] as u64);
}

/// # Safety
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    msg!("Call depth");
    let depth = *(input.add(16) as *mut u8);
    sol_log_64(line!() as u64, 0, 0, 0, depth as u64);
    let mut data = Vec::with_capacity(depth as usize);
    for i in 0_u8..depth {
        data.push(i);
    }
    recurse(&mut data);
    SUCCESS
}

custom_panic_default!();
