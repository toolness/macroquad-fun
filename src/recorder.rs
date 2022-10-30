use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
};

use crate::input::{Buttons, InputStream};

#[derive(Serialize, Deserialize)]
struct RecordedFrame {
    frame_number: u64,
    buttons: Buttons,
}

pub struct InputRecorder {
    source: InputStream,
    output: BufWriter<File>,
    prev_buttons: Option<Buttons>,
    frame_number: u64,
}

impl InputRecorder {
    pub fn new(source: InputStream, output: File) -> InputStream {
        Box::new(InputRecorder {
            frame_number: 0,
            source,
            prev_buttons: None,
            output: BufWriter::new(output),
        })
    }
}

impl Drop for InputRecorder {
    fn drop(&mut self) {
        // TODO: It's probably not a great idea to do something that could
        // panic during drop().
        self.output.flush().unwrap();
    }
}

impl Iterator for InputRecorder {
    type Item = Buttons;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.source.next();
        if let Some(buttons) = next {
            let did_buttons_change = match self.prev_buttons {
                Some(prev_buttons) => buttons != prev_buttons,
                None => true,
            };
            if did_buttons_change {
                self.prev_buttons = Some(buttons);
                let frame = RecordedFrame {
                    frame_number: self.frame_number,
                    buttons,
                };
                let mut buf = [0u8; 1024];
                let serialized_frame = postcard::to_slice(&frame, &mut buf).unwrap();

                // Ideally we should have a separate thread that does this, to minimize latency.
                self.output.write(&serialized_frame).unwrap();
            }
            self.frame_number += 1;
        }
        next
    }
}

pub struct InputPlayer {
    input: BufReader<File>,
    next_frame: Option<RecordedFrame>,
}

impl InputPlayer {
    pub fn new(input: File) -> InputStream {
        Box::new(InputPlayer {
            input: BufReader::new(input),
            next_frame: None,
        })
    }
}

impl Iterator for InputPlayer {
    type Item = Buttons;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO
        println!("Recording playback ended.");
        None
    }
}
