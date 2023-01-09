#[cfg(target_arch = "wasm32")]
pub mod js_interop_wasm32 {
    use std::{cell::RefCell, io::Write, rc::Rc};

    use crate::{
        input::{create_macroquad_input_stream, InputStream},
        recorder::{InputPlayer, InputRecorder},
    };

    extern "C" {
        fn init_version(version_str: *const u8);
        fn record_input(data: *const u8, len: usize);
        fn does_browser_support_ogg() -> i32;
    }

    #[derive(Default)]
    pub struct JsState {
        pub is_blurred: bool,
    }

    static mut JS_STATE: Option<JsState> = None;

    fn js_state_mut() -> &'static mut JsState {
        unsafe {
            if JS_STATE.is_none() {
                JS_STATE = Some(JsState::default());
            }
            JS_STATE.as_mut().unwrap()
        }
    }

    pub fn is_blurred() -> bool {
        js_state_mut().is_blurred
    }

    #[no_mangle]
    extern "C" fn set_blurred(is_blurred: i32) {
        js_state_mut().is_blurred = is_blurred != 0;
    }

    struct JsWriter();

    impl Write for JsWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            unsafe { record_input(buf.as_ptr(), buf.len()) }
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    pub fn init() {
        let version = format!("{}\0", env!("CARGO_PKG_VERSION"));
        unsafe { init_version(version.as_ptr()) }
    }

    pub fn is_ogg_supported() -> bool {
        unsafe { does_browser_support_ogg() != 0 }
    }

    pub fn create_input_stream() -> InputStream {
        // THIS IS EXTREMELY STUPID CODE WRITTEN TO MAKE
        // RUST STOP BUGGING ME ABOUT UNUSED CRAP IN THE
        // WASM BUILD
        if false {
            for _wtf in InputPlayer::new(vec![]) {}
        }

        let output = Rc::new(RefCell::new(JsWriter()));
        InputRecorder::new(create_macroquad_input_stream(), output)
    }
}
