extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::rc::Rc;

use cli::Cli;
use config::load_config;
use debug_mode::DebugMode;
use fps::FpsCounter;
use game_assets::load_game_assets;
use input::{InputState, InputStream};
use level_runtime::{FrameResult, LevelRuntime, SavedLevelRuntime};
use macroquad::prelude::*;
use player::create_player;
use time::FixedGameTime;
use time_stream::{create_fixed_fps_time_stream, create_real_time_stream, TimeStream};
use world::World;

mod animator;
mod aseprite;
mod attachment;
mod audio;
mod camera;
mod cli;
mod collision;
mod config;
mod crate_entity;
mod debug_mode;
mod drawing;
mod dynamic_collider;
mod entity;
mod floor_switch;
mod flying_eye;
mod font;
mod foreground_tiles;
mod fps;
mod game_assets;
mod gem_counter;
mod hierarchy;
mod input;
mod js_interop;
mod ldtk;
mod level;
mod level_runtime;
mod life_transfer;
mod materials;
mod math_util;
mod moving_platform;
mod mushroom;
mod physics;
mod pickups;
mod player;
mod push;
mod recorder;
mod route;
mod running;
mod sprite_component;
mod sprite_renderer;
mod steering;
mod switch;
mod text;
mod time;
mod time_stream;
mod trigger;
mod world;
mod xy_range_iterator;
mod z_index;

#[cfg(target_arch = "wasm32")]
use js_interop::js_interop_wasm32 as js;

const CONFIG_PATH: &str = "media/config.json";

const EXPORT_FRAMES_FPS: u64 = 30;

fn window_conf() -> Conf {
    #[cfg(target_arch = "wasm32")]
    return Default::default();

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Unfortunately this function isn't async, so we can't use Macroquad's WASM-compatible
        // async filesystem functions here. So all this code is specific to native builds.
        //
        // MacOS in particular won't let us change window dimensions after launching, so we'll
        // need to parse the config synchronously here and pass the correct dimensions to it.
        let config = config::parse_config(&std::fs::read_to_string(CONFIG_PATH).unwrap())
            .expect("parse_config() must succeed");

        Conf {
            window_title: "Macroquad Fun".to_owned(),
            window_width: config.screen_width as i32,
            window_height: config.screen_height as i32,
            window_resizable: false,
            ..Default::default()
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn create_input_stream(_args: &Cli) -> InputStream {
    js::create_input_stream()
}

#[cfg(not(target_arch = "wasm32"))]
fn create_input_stream(args: &Cli) -> InputStream {
    use crate::recorder::InputPlayer;
    use input::create_macroquad_input_stream;
    use recorder::InputRecorder;
    use std::{cell::RefCell, io::BufWriter};

    if let Some(filename) = &args.record {
        println!("Writing recording to '{}'.", filename);
        // Ideally we should be keeping a reference to the output and flushing it explicitly
        // when we're done recording, but eh, we'll just assume all filesystem operations are
        // successful for now (the BufWriter will flush on drop, it will just silently fail
        // if it doesn't work).
        let output = Rc::new(RefCell::new(BufWriter::new(
            std::fs::File::create(filename).expect("Unable to create recording file"),
        )));
        InputRecorder::new(create_macroquad_input_stream(), output)
    } else if let Some(filename) = &args.playback {
        println!("Playing back recording from '{}'.", filename);
        Box::new(
            InputPlayer::new(
                std::fs::read(filename).expect("Unable to open recording file for reading"),
            )
            .chain(if args.stop_at_end {
                Box::new(std::iter::empty())
            } else {
                create_macroquad_input_stream()
            }),
        )
    } else {
        create_macroquad_input_stream()
    }
}

fn create_time_stream(args: &Cli) -> TimeStream {
    if args.export_frames.is_some() {
        create_fixed_fps_time_stream(EXPORT_FRAMES_FPS)
    } else {
        create_real_time_stream()
    }
}

fn draw_pause_overlay(is_browser: bool) {
    draw_rectangle(
        0.,
        0.,
        screen_width(),
        screen_height(),
        color_u8!(0., 0., 0., 128.),
    );
    let font = &game_assets::game_assets().font;
    let text = if is_browser {
        "Game paused (press ESC to unpause)"
    } else {
        "Game paused (press Q to quit)"
    };
    font.draw_centered_text(text, screen_width() / 2., screen_height() / 2., WHITE);
}

#[macroquad::main(window_conf)]
async fn main() {
    let args = Cli::get_for_platform();

    load_config(CONFIG_PATH)
        .await
        .expect("load_config() must succeed");
    load_game_assets()
        .await
        .expect("load_game_sprites() must succeed");
    let world = Rc::new(
        World::load("media/world.ldtk")
            .await
            .expect("World::load() must succeed"),
    );

    let mut level_runtime = new_game(&args.start_position, world.clone());
    let config = config::config();

    #[cfg(target_arch = "wasm32")]
    {
        // I'm unclear on whether this is actually needed on wasm32, but
        // just in case...
        //
        // (Note that native builds will have already set the screen size
        // properly via our window_conf function.)
        request_new_screen_size(config.screen_width, config.screen_height);
        next_frame().await;

        js::init();
    }

    let mut fixed_time = FixedGameTime::new(config.fixed_fps, get_time());
    let mut enable_debug_mode = false;
    let mut opt_debug_mode: Option<DebugMode> = None;
    let mut render_fps = FpsCounter::default();
    let mut fixed_fps = FpsCounter::default();
    let mut input_state = InputState::default();
    let mut input_stream = create_input_stream(&args);
    let mut saved_state: Option<(SavedLevelRuntime, FixedGameTime)> = None;
    let mut time_stream = create_time_stream(&args);
    let mut frame_number: u64 = 0;
    let is_browser = cfg!(target_arch = "wasm32");

    'outer: loop {
        let Some(now) = time_stream.next() else {
            break;
        };
        if !fixed_time.is_paused() {
            fixed_time.update(now);

            for time in fixed_time.iter_fixed_frames() {
                let Some(new_buttons) = input_stream.next() else {
                    break 'outer;
                };
                input_state.update(new_buttons);
                fixed_fps.update(time.now);
                match level_runtime.advance_one_frame(&time, &input_state) {
                    FrameResult::Ok => {}
                    FrameResult::MainPlayerDied => {
                        level_runtime = new_game(&args.start_position, world.clone());
                    }
                }
            }
        }

        render_fps.update(now);
        level_runtime.draw();

        if let Some(basename) = &args.export_frames {
            let screen_data = get_screen_data();
            let filename = format!("{}_{:05}.png", basename, frame_number);
            println!("Writing {}.", filename);
            screen_data.export_png(&filename);
        }

        if is_key_released(KeyCode::Escape) {
            fixed_time.toggle_pause(now);
        }

        #[cfg(target_arch = "wasm32")]
        if js::is_blurred() && !fixed_time.is_paused() {
            fixed_time.set_paused(true, now);
        }

        if is_key_pressed(KeyCode::G) {
            enable_debug_mode = !enable_debug_mode;
        }

        if is_key_released(KeyCode::F5) {
            saved_state = Some((level_runtime.save(), fixed_time.create_paused_clone()));
            println!("Saved state.");
        }

        if is_key_released(KeyCode::F9) {
            if let Some((saved_level_runtime, paused_time)) = saved_state.as_ref() {
                level_runtime = LevelRuntime::from_saved(saved_level_runtime.clone());
                input_state = InputState::default();
                let mut new_fixed_time = paused_time.clone();
                new_fixed_time.set_paused(fixed_time.is_paused(), now);
                fixed_time = new_fixed_time;
                println!("Loaded state.");
            } else {
                println!("No saved state exists!");
            }
        }

        if enable_debug_mode {
            let debug_mode = opt_debug_mode.get_or_insert_with(|| DebugMode::default());
            debug_mode
                .update(&level_runtime, &fixed_fps, &render_fps)
                .expect("Generating debug text should work!");
            debug_mode.draw(&level_runtime);
        }

        if fixed_time.is_paused() {
            draw_pause_overlay(is_browser);
            if !is_browser && is_key_released(KeyCode::Q) {
                break;
            }
        }

        next_frame().await;
        frame_number += 1;
    }
}

fn new_game(start_position: &str, world: Rc<World>) -> LevelRuntime {
    let (level_start, player_start) = world.player_start(start_position).expect(&format!(
        "World does not define a PlayerStart entity called '{}'!",
        start_position
    ));
    LevelRuntime::new(create_player(player_start, "PLAYER"), level_start, world)
}
