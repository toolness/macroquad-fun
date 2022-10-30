use std::{cell::RefCell, io::Write, rc::Rc};

use crate::input::{Buttons, InputStream};

#[derive(Serialize, Deserialize)]
struct RecordedFrame {
    frame_number: u64,
    buttons: Buttons,
}

pub struct InputRecorder<W: Write> {
    source: InputStream,
    output: Rc<RefCell<W>>,
    prev_buttons: Option<Buttons>,
    frame_number: u64,
}

impl<W: Write + 'static> InputRecorder<W> {
    pub fn new(source: InputStream, output: Rc<RefCell<W>>) -> InputStream {
        Box::new(InputRecorder {
            frame_number: 0,
            source,
            prev_buttons: None,
            output,
        })
    }
}

impl<W: Write> Iterator for InputRecorder<W> {
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
                self.output.borrow_mut().write(&serialized_frame).unwrap();
            }
            self.frame_number += 1;
        }
        next
    }
}

pub struct InputPlayer {
    frames: Vec<RecordedFrame>,
    frames_index: usize,
    frame_number: u64,
    latest_buttons: Buttons,
}

impl InputPlayer {
    pub fn new(input: Vec<u8>) -> InputStream {
        let mut frames: Vec<RecordedFrame> = vec![];
        let mut input_remaining: &[u8] = input.as_ref();
        while input_remaining.len() > 0 {
            let (frame, unused) = postcard::take_from_bytes::<RecordedFrame>(&input_remaining)
                .expect("Unable to deserialize RecordedFrame");
            input_remaining = unused;
            frames.push(frame);
        }
        println!("Loaded {} input events.", frames.len());
        Box::new(InputPlayer {
            frames,
            frames_index: 0,
            frame_number: 0,
            latest_buttons: Buttons::default(),
        })
    }
}

impl Iterator for InputPlayer {
    type Item = Buttons;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frames_index == self.frames.len() {
            None
        } else {
            let frame = &self.frames[self.frames_index];
            if frame.frame_number == self.frame_number {
                self.latest_buttons = frame.buttons;
                self.frames_index += 1;
                if self.frames_index == self.frames.len() {
                    println!("Recording playback ended.");
                }
            }
            self.frame_number += 1;
            Some(self.latest_buttons)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{input::Buttons, recorder::InputPlayer};

    use super::InputRecorder;

    #[test]
    fn test_it_works() {
        let buttons = vec![
            Buttons::empty(),
            Buttons::LEFT,
            Buttons::LEFT,
            Buttons::RIGHT,
        ];

        let recording = Rc::new(RefCell::new(vec![]));
        let recorder = InputRecorder::new(Box::new(buttons.clone().into_iter()), recording.clone());
        let recorder_output: Vec<Buttons> = recorder.collect();
        assert_eq!(recorder_output, buttons);

        let player = InputPlayer::new(recording.borrow().clone());
        let player_output: Vec<Buttons> = player.collect();
        assert_eq!(player_output, buttons);
    }
}
