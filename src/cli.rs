use argh::FromArgs;

const DEFAULT_START_POSITION: &str = "default";

#[derive(argh::FromArgs)]
/// macroquad-fun
pub struct Cli {
    #[argh(option, short = 'p', default = "String::from(DEFAULT_START_POSITION)")]
    /// starting position, defined by PlayerStart entities in LDtk
    pub start_position: String,
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
