#![cfg_attr(all(not(debug_assertions), not(test)), windows_subsystem = "windows")]

mod draw;
mod game;
mod sfx;

use game::{Game, Input};
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Space Invaders".to_owned(),
        window_width: 800,
        window_height: 900,
        window_resizable: true,
        high_dpi: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    install_panic_log();
    show_mouse(false);

    let mut audio = sfx::Audio::load().await;
    let mut game = Game::new(sfx::seed_from_clock(), Game::load_highscore());
    let mut fullscreen = false;

    loop {
        if is_key_pressed(KeyCode::Escape) || is_quit_requested() {
            break;
        }
        if is_key_pressed(KeyCode::F11) {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
        }
        if is_key_pressed(KeyCode::M) {
            let ufo_alive = game.ufo.is_some() && game.mode == game::Mode::Playing;
            audio.toggle_mute(ufo_alive);
        }

        let input = Input {
            left: is_key_down(KeyCode::Left) || is_key_down(KeyCode::A),
            right: is_key_down(KeyCode::Right) || is_key_down(KeyCode::D),
            fire_held: is_key_down(KeyCode::Space),
            start_pressed: is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter),
            pause_pressed: is_key_pressed(KeyCode::P),
        };

        game.update(get_frame_time(), &input);
        for event in std::mem::take(&mut game.events) {
            audio.play(event);
        }
        if game.high_score_dirty {
            game.save_highscore();
            game.high_score_dirty = false;
        }

        draw::render(&game, audio.muted);
        next_frame().await;
    }

    game.save_highscore();
}

fn install_panic_log() {
    std::panic::set_hook(Box::new(|info| {
        let line = format!("{info}");
        if let Ok(home) = std::env::var("USERPROFILE") {
            let path = std::path::Path::new(&home).join("space-invaders-panic.log");
            let _ = std::fs::write(path, line);
        }
    }));
}
