use aec_rs::{Aec, AecConfig};

#[no_mangle]
pub extern "C" fn AecNew(
    frame_size: usize,
    filter_length: i32,
    sample_rate: u32,
    enable_preprocess: bool,
) -> *mut Aec {
    let config = AecConfig {
        frame_size,
        filter_length,
        sample_rate,
        enable_preprocess,
    };
    let aec = Box::new(Aec::new(&config));
    Box::into_raw(aec)
}

#[no_mangle]
pub extern "C" fn AecCancelEcho(
    aec_ptr: *mut Aec,
    rec_buffer: *const i16,
    echo_buffer: *const i16,
    out_buffer: *mut i16,
    buffer_length: usize,
) {
    if let Some(aec) = unsafe { aec_ptr.as_ref() } {
        unsafe {
            aec.cancel_echo(
                std::slice::from_raw_parts(rec_buffer, buffer_length),
                std::slice::from_raw_parts(echo_buffer, buffer_length),
                std::slice::from_raw_parts_mut(out_buffer, buffer_length),
            );
        }
    }
}

#[no_mangle]
pub extern "C" fn AecDestroy(aec_ptr: *mut Aec) {
    if !aec_ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(aec_ptr);
        };
    }
}

#[no_mangle]
pub extern "C" fn AecPlayback(aec_ptr: *mut Aec, play: *const i16, len: usize) {
    if let Some(aec) = unsafe { aec_ptr.as_ref() } {
        let play = unsafe { std::slice::from_raw_parts(play, len) };
        aec.playback(play);
    }
}

#[no_mangle]
pub extern "C" fn AecCapture(aec_ptr: *mut Aec, rec: *const i16, out: *mut i16, len: usize) {
    if let Some(aec) = unsafe { aec_ptr.as_ref() } {
        unsafe {
            aec.capture(
                std::slice::from_raw_parts(rec, len),
                std::slice::from_raw_parts_mut(out, len),
            );
        }
    }
}

#[no_mangle]
pub extern "C" fn AecSetSamplingRate(aec_ptr: *mut Aec, rate: u32) {
    if let Some(aec) = unsafe { aec_ptr.as_ref() } {
        aec.set_sampling_rate(rate);
    }
}
