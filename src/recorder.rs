use std::fs::File;

use crate::input::{Buttons, InputStream};

pub struct InputRecorder {
    source: InputStream,
    output: File,
}

impl InputRecorder {
    pub fn new(source: InputStream, output: File) -> InputStream {
        println!("Creating InputRecorder!");
        Box::new(InputRecorder { source, output })
    }
}

impl Iterator for InputRecorder {
    type Item = Buttons;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: Write to output!
        self.source.next()
    }
}
