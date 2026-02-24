mod ffi;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use std::os::raw::c_void;
/// See https://www.speex.org/docs/api/speex-api-reference/speex__echo_8h.html

#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct AecConfig {
    pub frame_size: usize,
    pub filter_length: i32,
    pub sample_rate: u32,
    pub enable_preprocess: bool,
}

impl Default for AecConfig {
    /// Default for 16khz
    fn default() -> Self {
        Self {
            frame_size: 160,
            filter_length: 1600,
            sample_rate: 16000,
            enable_preprocess: true,
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Aec {
    echo_state: *mut aec_rs_sys::SpeexEchoState,
    preprocess_state: Option<*mut aec_rs_sys::SpeexPreprocessState>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Aec {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn new(config: &AecConfig) -> Self {
        let echo_state = unsafe {
            aec_rs_sys::speex_echo_state_init(config.frame_size as i32, config.filter_length)
        };

        // Set sampling rate immediately — Speex defaults to 8000 Hz internally
        unsafe {
            let mut rate = config.sample_rate as i32;
            aec_rs_sys::speex_echo_ctl(
                echo_state,
                aec_rs_sys::SPEEX_ECHO_SET_SAMPLING_RATE as _,
                &mut rate as *mut _ as *mut c_void,
            );
        }

        let preprocess_state = if config.enable_preprocess {
            unsafe {
                let den = aec_rs_sys::speex_preprocess_state_init(
                    config.frame_size as i32,
                    config.sample_rate as _,
                );
                aec_rs_sys::speex_preprocess_ctl(
                    den,
                    aec_rs_sys::SPEEX_PREPROCESS_SET_ECHO_STATE as _,
                    echo_state as *mut c_void,
                );

                // Aggressively suppress residual echo (default -40 is OK for silence)
                let mut echo_suppress: i32 = -60;
                aec_rs_sys::speex_preprocess_ctl(
                    den,
                    aec_rs_sys::SPEEX_PREPROCESS_SET_ECHO_SUPPRESS as _,
                    &mut echo_suppress as *mut _ as *mut c_void,
                );

                // KEY FIX: Default ECHO_SUPPRESS_ACTIVE is only -15 dB.
                // During speech (Pframe→1), the preprocessor interpolates toward
                // this value, providing almost zero additional suppression on top
                // of the linear filter's ~15 dB. Set to -45 dB so the preprocessor
                // aggressively suppresses residual echo even when speech is detected.
                let mut echo_suppress_active: i32 = -60;
                aec_rs_sys::speex_preprocess_ctl(
                    den,
                    aec_rs_sys::SPEEX_PREPROCESS_SET_ECHO_SUPPRESS_ACTIVE as _,
                    &mut echo_suppress_active as *mut _ as *mut c_void,
                );

                Some(den)
            }
        } else {
            None
        };

        Aec {
            echo_state,
            preprocess_state,
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn cancel_echo(&self, rec_buffer: &[i16], echo_buffer: &[i16], out_buffer: &mut [i16]) {
        unsafe {
            aec_rs_sys::speex_echo_cancellation(
                self.echo_state,
                rec_buffer.as_ptr(),
                echo_buffer.as_ptr(),
                out_buffer.as_mut_ptr(),
            );
            if let Some(preprocess_state) = self.preprocess_state {
                aec_rs_sys::speex_preprocess_run(preprocess_state, out_buffer.as_mut_ptr());
            }
        }
    }

    /// Feed reference (playback) audio into Speex's internal ring buffer.
    /// Use with `capture()` for async echo cancellation.
    pub fn playback(&self, play_buffer: &[i16]) {
        unsafe {
            aec_rs_sys::speex_echo_playback(self.echo_state, play_buffer.as_ptr());
        }
    }

    /// Process mic audio through Speex AEC using internally buffered reference.
    /// Speex pairs the mic frame with the correct historical reference automatically.
    pub fn capture(&self, rec_buffer: &[i16], out_buffer: &mut [i16]) {
        unsafe {
            aec_rs_sys::speex_echo_capture(self.echo_state, rec_buffer.as_ptr(), out_buffer.as_mut_ptr());
            if let Some(preprocess_state) = self.preprocess_state {
                aec_rs_sys::speex_preprocess_run(preprocess_state, out_buffer.as_mut_ptr());
            }
        }
    }

    /// Set Speex echo canceller sampling rate (defaults to 8000 if not set).
    pub fn set_sampling_rate(&self, rate: u32) {
        unsafe {
            let mut r = rate as i32;
            aec_rs_sys::speex_echo_ctl(
                self.echo_state,
                aec_rs_sys::SPEEX_ECHO_SET_SAMPLING_RATE as _,
                &mut r as *mut _ as *mut std::os::raw::c_void,
            );
        }
    }
}

impl Drop for Aec {
    fn drop(&mut self) {
        unsafe {
            aec_rs_sys::speex_echo_state_destroy(self.echo_state);
            if let Some(preprocess_state) = self.preprocess_state {
                aec_rs_sys::speex_preprocess_state_destroy(preprocess_state);
            }
        }
    }
}
