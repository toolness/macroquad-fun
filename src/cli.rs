use argh::FromArgs;

const DEFAULT_START_POSITION: &str = "default";

#[derive(argh::FromArgs)]
/// macroquad-fun
pub struct Cli {
    #[argh(option, short = 'p', default = "String::from(DEFAULT_START_POSITION)")]
    /// starting position, defined by PlayerStart entities in LDtk
    pub start_position: String,

    #[argh(option)]
    /// filename to record to
    pub record: Option<String>,

    #[argh(option)]
    /// filename to play back recording from
    pub playback: Option<String>,

    #[argh(switch)]
    /// exit at end of playback (only used with --playback).
    pub stop_at_end: bool,

    #[argh(option)]
    /// base filename to export frame images to
    pub export_frames: Option<String>,

    #[argh(option)]
    /// override screen width
    pub width: Option<u32>,

    #[argh(option)]
    /// override screen height
    pub height: Option<u32>,

    #[argh(switch)]
    /// enable fullscreen mode
    pub fullscreen: bool,
}

impl Cli {
    pub fn get_for_platform() -> Self {
        if cfg!(target_arch = "wasm32") {
            Cli::from_args(&["macroquad-fun"], &[]).unwrap()
        } else {
            argh::from_env()
        }
    }
}
