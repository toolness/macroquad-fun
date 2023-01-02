#[cfg(target_arch = "wasm32")]
pub mod js_interop_wasm32 {
    use std::{cell::RefCell, io::Write, rc::Rc};

    use crate::{
        input::{create_macroquad_input_stream, InputStream},
        recorder::{InputPlayer, InputRecorder},
    };

    extern "C" {
        fn hi_from_js();
    }

    struct JsWriter();

    impl Write for JsWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            // TODO: Give the buffer to JS.
            unsafe {
                hi_from_js();
            }
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
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
