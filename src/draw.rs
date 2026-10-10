//! Pixel sprites, a tiny arcade font, and the frame renderer.

use crate::game::{
    Boom, BoomKind, Bunker, Game, Mode, BUNKER_H, BUNKER_W, CELL, FIELD_H, FIELD_W, INV_H, INV_W,
    PLAYER_H, PLAYER_W, PLAYER_Y, UFO_H, UFO_W, UFO_Y,
};
use macroquad::prelude::*;

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}

fn bg() -> Color {
    rgb(6, 8, 16)
}
fn green() -> Color {
    rgb(118, 255, 140)
}
fn cyan() -> Color {
    rgb(90, 220, 255)
}
fn magenta() -> Color {
    rgb(255, 96, 176)
}
fn lime() -> Color {
    rgb(196, 255, 80)
}
fn red() -> Color {
    rgb(255, 78, 96)
}
fn yellow() -> Color {
    rgb(255, 214, 90)
}
fn white() -> Color {
    rgb(236, 244, 236)
}
fn ink_dim() -> Color {
    rgb(128, 148, 138)
}
fn bunker_green() -> Color {
    rgb(64, 206, 108)
}

struct Vp {
    scale: f32,
    ox: f32,
    oy: f32,
}

impl Vp {
    fn new(shake: f32, time: f32) -> Self {
        let scale = (screen_width() / FIELD_W).min(screen_height() / FIELD_H);
        let mut ox = (screen_width() - FIELD_W * scale) * 0.5;
        let mut oy = (screen_height() - FIELD_H * scale) * 0.5;
        if shake > 0.0 {
            let mag = shake * shake * 7.0 * scale;
            ox += (time * 53.0).sin() * mag;
            oy += (time * 47.0).cos() * mag;
        }
        Self { scale, ox, oy }
    }

    fn rect(&self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        draw_rectangle(
            self.ox + x * self.scale,
            self.oy + y * self.scale,
            w * self.scale,
            h * self.scale,
            color,
        );
    }
}

pub fn render(game: &Game, muted: bool) {
    clear_background(BLACK);
    let vp = Vp::new(game.shake, game.time);
    vp.rect(0.0, 0.0, FIELD_W, FIELD_H, bg());
    draw_stars(&vp, game.time);

    if game.mode == Mode::Title {
        draw_title(&vp, game, muted);
        return;
    }

    draw_hud(&vp, game);
    if let Some(ufo) = &game.ufo {
        blit(&vp, UFO, ufo.x, UFO_Y, 2.0, red());
    }
    draw_invaders(&vp, game);
    draw_bunkers(&vp, &game.bunkers);
    draw_bullets(&vp, game);
    draw_player(&vp, game);
    for boom in &game.booms {
        draw_boom(&vp, boom);
    }
    for popup in &game.popups {
        let label = format!("{}", popup.pts);
        draw_text(&vp, &label, popup.x, popup.y, 2.0, white());
    }
    draw_ground(&vp);
    draw_footer(&vp, game, muted);
    draw_overlay(&vp, game);
}

fn draw_stars(vp: &Vp, time: f32) {
    let mut state = 0x1234_5678u32;
    for index in 0..72 {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        let x = (state % FIELD_W as u32) as f32;
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        let y = (state % FIELD_H as u32) as f32;
        let twinkle = ((time * 1.7 + index as f32 * 0.37).sin() * 0.5 + 0.5) * 90.0 + 50.0;
        vp.rect(x, y, 2.0, 2.0, Color::from_rgba(200, 220, 255, twinkle as u8));
    }
}

fn draw_hud(vp: &Vp, game: &Game) {
    draw_text(vp, "SCORE", 18.0, 10.0, 2.0, green());
    draw_text(vp, &format!("{:06}", game.score), 18.0, 28.0, 2.0, white());
    let hi = "HI-SCORE";
    let hi_x = FIELD_W - 18.0 - text_width(hi, 2.0);
    draw_text(vp, hi, hi_x, 10.0, 2.0, magenta());
    let value = format!("{:06}", game.high_score);
    let value_x = FIELD_W - 18.0 - text_width(&value, 2.0);
    draw_text(vp, &value, value_x, 28.0, 2.0, white());
    vp.rect(16.0, 44.0, FIELD_W - 32.0, 1.0, Color::from_rgba(80, 110, 90, 255));
}

fn draw_invaders(vp: &Vp, game: &Game) {
    for row in 0..crate::game::ROWS {
        for col in 0..crate::game::COLS {
            if !game.alive[row][col] {
                continue;
            }
            let (x, y) = game.pos(col, row);
            let sprite = invader_sprite(row, game.anim);
            blit(vp, sprite, x, y, 2.0, invader_color(row));
        }
    }
}

fn invader_color(row: usize) -> Color {
    match row {
        0 => magenta(),
        1 | 2 => cyan(),
        _ => lime(),
    }
}

fn draw_bunkers(vp: &Vp, bunkers: &[Bunker]) {
    for bunker in bunkers {
        for row in 0..BUNKER_H {
            for col in 0..BUNKER_W {
                if bunker.cells[row][col] {
                    vp.rect(
                        bunker.x + col as f32 * CELL,
                        bunker.y + row as f32 * CELL,
                        CELL,
                        CELL,
                        bunker_green(),
                    );
                }
            }
        }
    }
}

fn draw_bullets(vp: &Vp, game: &Game) {
    for bullet in &game.bullets {
        if bullet.from_player {
            vp.rect(bullet.x, bullet.y, bullet.w, bullet.h, white());
        } else {
            let phase = (bullet.y as i32 / 4).rem_euclid(2);
            for step in 0..4 {
                let dx = if (step + phase) % 2 == 0 { 0.0 } else { 2.0 };
                vp.rect(bullet.x + dx, bullet.y + step as f32 * 2.0, 2.0, 2.0, yellow());
            }
        }
    }
}

fn draw_player(vp: &Vp, game: &Game) {
    let hidden = matches!(game.mode, Mode::Dying | Mode::GameOver)
        || (game.invuln > 0.0 && (game.time * 14.0) as i32 % 2 == 0);
    if !hidden {
        blit(vp, PLAYER, game.player_x, PLAYER_Y, 2.0, green());
    }
}

fn draw_boom(vp: &Vp, boom: &Boom) {
    let color = match boom.kind {
        BoomKind::Player => yellow(),
        BoomKind::Ufo => red(),
        BoomKind::Alien => white(),
    };
    let reach = 3.0 + boom.t * 30.0;
    let size = if boom.t < 0.1 { 3.0 } else { 2.0 };
    let dots = [
        (0.0, -1.0),
        (0.0, 1.0),
        (-1.0, 0.0),
        (1.0, 0.0),
        (-0.7, -0.7),
        (0.7, -0.7),
        (-0.7, 0.7),
        (0.7, 0.7),
    ];
    for (dx, dy) in dots {
        vp.rect(
            boom.x + dx * reach - size * 0.5,
            boom.y + dy * reach - size * 0.5,
            size,
            size,
            color,
        );
    }
}

fn draw_ground(vp: &Vp) {
    let y = PLAYER_Y + PLAYER_H + 8.0;
    let mut x = 16.0;
    while x < FIELD_W - 16.0 {
        vp.rect(x, y, 10.0, 2.0, green());
        x += 16.0;
    }
}

fn draw_footer(vp: &Vp, game: &Game, muted: bool) {
    let y = FIELD_H - 20.0;
    let reserve = (game.lives - 1).max(0);
    for life in 0..reserve {
        blit(
            vp,
            PLAYER,
            16.0 + life as f32 * (PLAYER_W + 6.0),
            y - 2.0,
            2.0,
            green(),
        );
    }
    let wave = format!("WAVE {:02}", game.wave);
    draw_text(vp, &wave, FIELD_W - 18.0 - text_width(&wave, 2.0), y, 2.0, white());
    if muted {
        draw_text(vp, "MUTED", FIELD_W * 0.5 - text_width("MUTED", 2.0) * 0.5, y, 2.0, ink_dim());
    }
}

fn draw_overlay(vp: &Vp, game: &Game) {
    match game.mode {
        Mode::Intro => {
            shade(vp);
            let wave = format!("WAVE {:02}", game.wave);
            draw_text_centered(vp, &wave, 230.0, 4.0, white());
            draw_text_centered(vp, "GET READY", 280.0, 2.0, green());
        }
        Mode::WaveClear => {
            shade(vp);
            draw_text_centered(vp, "WAVE CLEARED", 250.0, 3.0, lime());
        }
        Mode::Paused => {
            shade(vp);
            draw_text_centered(vp, "PAUSED", 240.0, 4.0, white());
            draw_text_centered(vp, "P TO RESUME", 300.0, 2.0, green());
        }
        Mode::GameOver => {
            shade(vp);
            if game.invaded {
                draw_text_centered(vp, "THEY LANDED", 220.0, 3.0, red());
            } else {
                draw_text_centered(vp, "GAME OVER", 230.0, 4.0, red());
            }
            if game.timer > 0.45 && (game.time * 2.0) as i32 % 2 == 0 {
                draw_text_centered(vp, "PRESS SPACE", 300.0, 2.0, white());
            }
        }
        _ => {}
    }
}

fn shade(vp: &Vp) {
    vp.rect(0.0, 0.0, FIELD_W, FIELD_H, Color::from_rgba(0, 0, 0, 150));
}

fn draw_title(vp: &Vp, game: &Game, muted: bool) {
    draw_text_centered(vp, "SPACE INVADERS", 32.0, 4.0, lime());
    draw_text_centered(vp, "SCORE ADVANCE TABLE", 84.0, 2.0, cyan());
    draw_legend(vp, UFO, UFO_W, UFO_H, 118.0, "= ???", red());
    draw_legend(vp, SQUID[0], INV_W, INV_H, 156.0, "= 30", magenta());
    draw_legend(vp, CRAB[0], INV_W, INV_H, 196.0, "= 20", cyan());
    draw_legend(vp, OCTOPUS[0], INV_W, INV_H, 236.0, "= 10", lime());

    let hi = format!("HI-SCORE  {:06}", game.high_score);
    draw_text_centered(vp, &hi, 292.0, 2.0, magenta());

    if (game.time * 2.2) as i32 % 2 == 0 {
        draw_text_centered(vp, "PRESS SPACE", 348.0, 3.0, white());
    }

    draw_text_centered(vp, "ARROWS OR A/D TO MOVE", 410.0, 2.0, green());
    draw_text_centered(vp, "SPACE TO FIRE", 436.0, 2.0, green());
    let sound = if muted { "P PAUSE   M MUTE   SOUND OFF" } else { "P PAUSE    M MUTE" };
    draw_text_centered(vp, sound, 462.0, 2.0, ink_dim());
    draw_text_centered(vp, "F11 FULLSCREEN    ESC QUIT", 488.0, 2.0, ink_dim());
    draw_text_centered(vp, "BUILT WITH RUST", 516.0, 2.0, Color::from_rgba(90, 110, 100, 255));
}

fn draw_legend(vp: &Vp, sprite: &[&str], sw: f32, sh: f32, y: f32, label: &str, color: Color) {
    let width = sw + 16.0 + text_width(label, 2.0);
    let x = (FIELD_W - width) * 0.5;
    blit(vp, sprite, x, y, 2.0, color);
    draw_text(vp, label, x + sw + 16.0, y + (sh - 14.0) * 0.5, 2.0, white());
}

fn draw_text_centered(vp: &Vp, text: &str, y: f32, pix: f32, color: Color) {
    let x = (FIELD_W - text_width(text, pix)) * 0.5;
    draw_text(vp, text, x, y, pix, color);
}

fn text_width(text: &str, pix: f32) -> f32 {
    text.chars().count() as f32 * 6.0 * pix
}

fn draw_text(vp: &Vp, text: &str, mut x: f32, y: f32, pix: f32, color: Color) {
    for ch in text.chars() {
        let rows = glyph(ch.to_ascii_uppercase());
        for (row, bits) in rows.iter().enumerate() {
            for col in 0..5 {
                if bits & (1 << (4 - col)) != 0 {
                    vp.rect(x + col as f32 * pix, y + row as f32 * pix, pix, pix, color);
                }
            }
        }
        x += 6.0 * pix;
    }
}

fn blit(vp: &Vp, sprite: &[&str], x: f32, y: f32, pix: f32, color: Color) {
    for (row, line) in sprite.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if ch == '#' {
                vp.rect(x + col as f32 * pix, y + row as f32 * pix, pix, pix, color);
            }
        }
    }
}

fn invader_sprite(row: usize, anim: u8) -> &'static [&'static str] {
    let frame = (anim & 1) as usize;
    match row {
        0 => SQUID[frame],
        1 | 2 => CRAB[frame],
        _ => OCTOPUS[frame],
    }
}

const PLAYER: &[&str] = &[
    "......#......",
    ".....###.....",
    "....#####....",
    "...#######...",
    ".###########.",
    "#############",
    "##.#######.##",
    "...##...##...",
];

const SQUID: [&[&str]; 2] = [
    &[
        "..#....#....",
        ".#.#..#.#...",
        ".########...",
        "##.####.##..",
        "############",
        ".#.#..#.#...",
        "..#....#....",
        ".#......#...",
    ],
    &[
        "..#....#....",
        ".#.#..#.#...",
        ".########...",
        "##.####.##..",
        "############",
        ".#.#..#.#...",
        "#..#..#..#..",
        "..#....#....",
    ],
];

const CRAB: [&[&str]; 2] = [
    &[
        "#..........#",
        ".#........#.",
        ".##########.",
        "##.#.##.#.##",
        ".##########.",
        "..#.#..#.#..",
        ".#........#.",
        "#.#......#.#",
    ],
    &[
        "#..........#",
        ".#........#.",
        ".##########.",
        "##.#.##.#.##",
        ".##########.",
        ".#.#.#.#.#.#",
        "#..........#",
        ".#........#.",
    ],
];

const OCTOPUS: [&[&str]; 2] = [
    &[
        "...####.....",
        ".##########.",
        "############",
        "#.##.##.##.#",
        ".##########.",
        "..#..##..#..",
        ".#.#....#.#.",
        "#.#......#.#",
    ],
    &[
        "...####.....",
        ".##########.",
        "############",
        "#.##.##.##.#",
        ".##########.",
        ".#..#..#..#.",
        "#.#.#..#.#.#",
        ".#........#.",
    ],
];

const UFO: &[&str] = &[
    ".....######.....",
    "...##########...",
    ".##############.",
    "###.###..###.###",
    ".##############.",
    "...#.#.##.#.#...",
];

fn glyph(c: char) -> [u8; 7] {
    match c {
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        '?' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100],
        '/' => [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000],
        '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100],
        _ => [0, 0, 0, 0, 0, 0, 0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_grid(name: &str, sprite: &[&str], width: usize) {
        assert!(!sprite.is_empty(), "{name} is empty");
        for (index, row) in sprite.iter().enumerate() {
            assert_eq!(
                row.chars().count(),
                width,
                "{name} row {index} is {} wide",
                row.chars().count()
            );
        }
    }

    #[test]
    fn sprites_are_rectangular() {
        assert_grid("player", PLAYER, 13);
        assert_eq!(PLAYER.len(), 8);
        for (index, frame) in SQUID.iter().enumerate() {
            assert_grid(&format!("squid {index}"), frame, 12);
        }
        for (index, frame) in CRAB.iter().enumerate() {
            assert_grid(&format!("crab {index}"), frame, 12);
        }
        for (index, frame) in OCTOPUS.iter().enumerate() {
            assert_grid(&format!("octopus {index}"), frame, 12);
        }
        assert_grid("ufo", UFO, 16);
        assert_eq!(UFO.len(), 6);
    }

    #[test]
    fn title_copy_has_glyphs() {
        let text = "SPACE INVADERS SCORE ADVANCE TABLE PRESS ARROWS OR A/D TO MOVE FIRE PAUSE MUTE FULLSCREEN ESC QUIT BUILT WITH RUST WAVE GET READY CLEARED GAME OVER THEY LANDED HIT HI-SCORE ?!";
        for ch in text.chars() {
            if ch == ' ' {
                continue;
            }
            let rows = glyph(ch);
            assert!(rows.iter().any(|row| *row != 0), "missing glyph {ch}");
        }
    }
}
