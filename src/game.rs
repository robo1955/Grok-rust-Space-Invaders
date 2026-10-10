//! Arcade rules for Space Invaders, with no graphics dependency.
//! The window loop in `main` feeds keyboard input in and draws the result.

use std::path::PathBuf;

pub const FIELD_W: f32 = 480.0;
pub const FIELD_H: f32 = 540.0;
pub const MARGIN: f32 = 16.0;

pub const COLS: usize = 11;
pub const ROWS: usize = 5;
pub const INV_W: f32 = 24.0;
pub const INV_H: f32 = 16.0;
pub const PITCH_X: f32 = 32.0;
pub const PITCH_Y: f32 = 26.0;
pub const STEP_X: f32 = 8.0;
pub const STEP_Y: f32 = 14.0;

pub const PLAYER_W: f32 = 26.0;
pub const PLAYER_H: f32 = 16.0;
pub const PLAYER_Y: f32 = 486.0;
pub const PLAYER_SPEED: f32 = 220.0;
pub const PLAYER_BULLET_SPEED: f32 = 460.0;

pub const UFO_W: f32 = 32.0;
pub const UFO_H: f32 = 12.0;
pub const UFO_Y: f32 = 48.0;
pub const UFO_SPEED: f32 = 120.0;

pub const BUNKER_W: usize = 22;
pub const BUNKER_H: usize = 16;
pub const CELL: f32 = 2.0;

/// The cabinet did not roll a random number. It walked this table once per player shot.
const UFO_SCORES: [u32; 15] = [
    100, 50, 50, 100, 150, 100, 300, 100, 100, 50, 150, 100, 100, 50, 100,
];

#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub fire_held: bool,
    pub start_pressed: bool,
    pub pause_pressed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Title,
    Intro,
    Playing,
    Paused,
    Dying,
    WaveClear,
    GameOver,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sfx {
    Shoot,
    Step(u8),
    InvaderDie,
    PlayerDie,
    UfoLoop,
    UfoStop,
    UfoDie,
    Wave,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoomKind {
    Alien,
    Player,
    Ufo,
}

#[derive(Clone, Copy, Debug)]
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub vy: f32,
    pub from_player: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Ufo {
    pub x: f32,
    pub dir: f32,
    pub score: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Boom {
    pub x: f32,
    pub y: f32,
    pub t: f32,
    pub kind: BoomKind,
}

#[derive(Clone, Copy, Debug)]
pub struct Popup {
    pub x: f32,
    pub y: f32,
    pub pts: u32,
    pub t: f32,
}

#[derive(Clone, Debug)]
pub struct Bunker {
    pub x: f32,
    pub y: f32,
    pub cells: [[bool; BUNKER_W]; BUNKER_H],
}

pub struct Game {
    pub mode: Mode,
    pub timer: f32,
    pub time: f32,
    pub score: u32,
    pub high_score: u32,
    pub high_score_dirty: bool,
    pub lives: i32,
    pub wave: u32,
    pub player_x: f32,
    pub invuln: f32,
    pub shake: f32,
    pub invaded: bool,
    pub alive: [[bool; COLS]; ROWS],
    pub origin_x: f32,
    pub origin_y: f32,
    pub dir: f32,
    pub anim: u8,
    pub bunkers: Vec<Bunker>,
    pub bullets: Vec<Bullet>,
    pub ufo: Option<Ufo>,
    pub booms: Vec<Boom>,
    pub popups: Vec<Popup>,
    pub events: Vec<Sfx>,
    step_acc: f32,
    fire_cool: f32,
    fire_timer: f32,
    ufo_timer: f32,
    player_shots: u32,
    step_note: u8,
    rng: Rng,
}

struct Rng(u64);

impl Rng {
    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x as u32
    }

    fn gen(&mut self, n: u32) -> u32 {
        self.next_u32() % n
    }
}

pub fn player_spawn_x() -> f32 {
    (FIELD_W - PLAYER_W) * 0.5
}

fn formation_width() -> f32 {
    (COLS - 1) as f32 * PITCH_X + INV_W
}

pub fn points_for_row(row: usize) -> u32 {
    match row {
        0 => 30,
        1 | 2 => 20,
        _ => 10,
    }
}

pub fn highscore_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let in_target = dir
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == "release" || name == "debug");
            if in_target {
                if let Some(target) = dir.parent() {
                    if target.file_name().and_then(|name| name.to_str()) == Some("target") {
                        if let Some(root) = target.parent() {
                            return root.join("highscore.txt");
                        }
                    }
                }
            }
            return dir.join("highscore.txt");
        }
    }
    PathBuf::from("highscore.txt")
}

impl Game {
    pub fn new(seed: u64, high_score: u32) -> Self {
        Self {
            mode: Mode::Title,
            timer: 0.0,
            time: 0.0,
            score: 0,
            high_score,
            high_score_dirty: false,
            lives: 3,
            wave: 1,
            player_x: player_spawn_x(),
            invuln: 0.0,
            shake: 0.0,
            invaded: false,
            alive: [[false; COLS]; ROWS],
            origin_x: 0.0,
            origin_y: 0.0,
            dir: 1.0,
            anim: 0,
            bunkers: Vec::new(),
            bullets: Vec::new(),
            ufo: None,
            booms: Vec::new(),
            popups: Vec::new(),
            events: Vec::new(),
            step_acc: 0.0,
            fire_cool: 0.0,
            fire_timer: 1.2,
            ufo_timer: 8.0,
            player_shots: 0,
            step_note: 0,
            rng: Rng(seed | 1),
        }
    }

    pub fn load_highscore() -> u32 {
        std::fs::read_to_string(highscore_path())
            .ok()
            .and_then(|text| text.trim().parse().ok())
            .unwrap_or(0)
    }

    pub fn save_highscore(&self) {
        let path = highscore_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, self.high_score.to_string());
    }

    pub fn update(&mut self, dt: f32, input: &Input) {
        let dt = dt.clamp(0.0, 0.05);
        self.time += dt;

        if self.mode == Mode::Title {
            if input.start_pressed {
                self.start_game();
            }
            return;
        }

        if input.pause_pressed {
            match self.mode {
                Mode::Playing => {
                    if self.ufo.is_some() {
                        self.events.push(Sfx::UfoStop);
                    }
                    self.mode = Mode::Paused;
                }
                Mode::Paused => {
                    self.mode = Mode::Playing;
                    if self.ufo.is_some() {
                        self.events.push(Sfx::UfoLoop);
                    }
                }
                _ => {}
            }
        }
        if self.mode == Mode::Paused {
            return;
        }

        self.tick_fx(dt);

        if self.mode == Mode::GameOver {
            self.timer += dt;
            if input.start_pressed && self.timer > 0.45 {
                self.start_game();
            }
            return;
        }

        if self.mode == Mode::Dying {
            self.timer -= dt;
            if self.timer <= 0.0 {
                if self.lives > 0 {
                    self.player_x = player_spawn_x();
                    self.invuln = 2.0;
                    self.mode = Mode::Playing;
                } else {
                    self.mode = Mode::GameOver;
                    self.timer = 0.0;
                }
            }
            return;
        }

        if self.mode == Mode::WaveClear {
            self.timer -= dt;
            if self.timer <= 0.0 {
                self.wave += 1;
                self.spawn_wave();
                self.mode = Mode::Intro;
                self.timer = 1.35;
            }
            return;
        }

        if self.mode == Mode::Intro {
            self.timer -= dt;
            if self.timer <= 0.0 {
                self.mode = Mode::Playing;
            }
        }

        self.move_player(dt, input);
        self.try_fire(dt, input);
        self.move_bullets(dt);
        self.collide();

        if self.mode != Mode::Playing {
            return;
        }
        self.advance_invaders(dt);
        if self.mode != Mode::Playing {
            return;
        }
        self.enemy_fire(dt);
        self.update_ufo(dt);
    }

    fn start_game(&mut self) {
        if self.ufo.take().is_some() {
            self.events.push(Sfx::UfoStop);
        }
        self.score = 0;
        self.lives = 3;
        self.wave = 1;
        self.player_shots = 0;
        self.invaded = false;
        self.player_x = player_spawn_x();
        self.invuln = 0.0;
        self.fire_cool = 0.0;
        self.shake = 0.0;
        self.booms.clear();
        self.popups.clear();
        self.bullets.clear();
        self.bunkers = make_bunkers();
        self.spawn_wave();
        self.ufo_timer = 8.0;
        self.fire_timer = 1.4;
        self.mode = Mode::Intro;
        self.timer = 1.5;
        self.events.push(Sfx::Wave);
    }

    fn spawn_wave(&mut self) {
        self.alive = [[true; COLS]; ROWS];
        self.origin_x = (FIELD_W - formation_width()) * 0.5;
        let drop = (self.wave.saturating_sub(1) as f32) * 12.0;
        self.origin_y = 78.0 + drop.min(96.0);
        self.dir = 1.0;
        self.anim = 0;
        self.step_acc = 0.0;
        self.step_note = 0;
        self.bullets.clear();
        self.ufo = None;
    }

    fn tick_fx(&mut self, dt: f32) {
        self.shake = (self.shake - dt * 2.8).max(0.0);
        self.invuln = (self.invuln - dt).max(0.0);
        for boom in &mut self.booms {
            boom.t += dt;
        }
        self.booms.retain(|boom| {
            boom.t < if boom.kind == BoomKind::Player { 0.7 } else { 0.38 }
        });
        for popup in &mut self.popups {
            popup.t += dt;
            popup.y -= 22.0 * dt;
        }
        self.popups.retain(|popup| popup.t < 0.75);
    }

    fn move_player(&mut self, dt: f32, input: &Input) {
        if input.left {
            self.player_x -= PLAYER_SPEED * dt;
        }
        if input.right {
            self.player_x += PLAYER_SPEED * dt;
        }
        let max_x = FIELD_W - MARGIN - PLAYER_W;
        self.player_x = self.player_x.clamp(MARGIN, max_x);
    }

    fn try_fire(&mut self, dt: f32, input: &Input) {
        self.fire_cool = (self.fire_cool - dt).max(0.0);
        let armed = self.bullets.iter().any(|bullet| bullet.from_player);
        if input.fire_held && !armed && self.fire_cool <= 0.0 {
            self.bullets.push(Bullet {
                x: self.player_x + PLAYER_W * 0.5 - 1.5,
                y: PLAYER_Y - 12.0,
                w: 3.0,
                h: 10.0,
                vy: -PLAYER_BULLET_SPEED,
                from_player: true,
            });
            self.fire_cool = 0.16;
            self.player_shots = self.player_shots.saturating_add(1);
            self.events.push(Sfx::Shoot);
        }
    }

    fn move_bullets(&mut self, dt: f32) {
        for bullet in &mut self.bullets {
            bullet.y += bullet.vy * dt;
        }
        self.bullets
            .retain(|bullet| bullet.y > -24.0 && bullet.y < FIELD_H + 8.0);
    }

    fn collide(&mut self) {
        let snapshot = self.bullets.clone();
        let mut dead = vec![false; snapshot.len()];
        let active = self.mode == Mode::Playing || self.mode == Mode::Intro;

        for (index, bullet) in snapshot.iter().enumerate() {
            if self.mode == Mode::Dying {
                break;
            }
            if bullet.from_player {
                if let Some(ufo) = self.ufo {
                    if overlaps(bullet, ufo.x, UFO_Y, UFO_W, UFO_H) {
                        dead[index] = true;
                        let pts = ufo.score;
                        self.ufo = None;
                        self.add_score(pts);
                        self.popups.push(Popup {
                            x: ufo.x,
                            y: UFO_Y,
                            pts,
                            t: 0.0,
                        });
                        self.booms.push(Boom {
                            x: ufo.x + UFO_W * 0.5,
                            y: UFO_Y + UFO_H * 0.5,
                            t: 0.0,
                            kind: BoomKind::Ufo,
                        });
                        self.events.push(Sfx::UfoStop);
                        self.events.push(Sfx::UfoDie);
                        continue;
                    }
                }
                if let Some((row, col)) = self.find_invader(bullet) {
                    self.alive[row][col] = false;
                    dead[index] = true;
                    let pts = points_for_row(row);
                    let (x, y) = self.pos(col, row);
                    self.add_score(pts);
                    self.popups.push(Popup { x, y, pts, t: 0.0 });
                    self.booms.push(Boom {
                        x: x + INV_W * 0.5,
                        y: y + INV_H * 0.5,
                        t: 0.0,
                        kind: BoomKind::Alien,
                    });
                    self.events.push(Sfx::InvaderDie);
                    continue;
                }
            } else if active && self.invuln <= 0.0 && self.player_overlaps(bullet) {
                dead[index] = true;
                self.kill_player();
                break;
            }

            if self.damage_bunkers(bullet) {
                dead[index] = true;
            }
        }

        if self.mode == Mode::Dying {
            self.bullets.clear();
        } else {
            let mut index = 0;
            self.bullets.retain(|_| {
                let keep = !dead[index];
                index += 1;
                keep
            });
        }

        if self.alive_count() == 0 && active && self.mode != Mode::Dying {
            if self.ufo.take().is_some() {
                self.events.push(Sfx::UfoStop);
            }
            self.mode = Mode::WaveClear;
            self.timer = 1.5;
            self.events.push(Sfx::Wave);
        }
    }

    fn advance_invaders(&mut self, dt: f32) {
        self.step_acc += dt;
        if self.step_acc < self.step_interval() {
            return;
        }
        self.step_acc = 0.0;

        let Some((min_x, max_x, _)) = self.bounds() else {
            return;
        };
        let dx = self.dir * STEP_X;
        let hit_edge = if self.dir > 0.0 {
            max_x + dx > FIELD_W - MARGIN
        } else {
            min_x + dx < MARGIN
        };
        if hit_edge {
            self.dir = -self.dir;
            self.origin_y += STEP_Y;
        } else {
            self.origin_x += dx;
        }
        self.anim ^= 1;
        self.step_note = (self.step_note + 1) % 4;
        self.events.push(Sfx::Step(self.step_note));
        self.erode_bunkers_under_invaders();
        self.check_landed();
        if self.mode == Mode::Dying {
            self.bullets.clear();
        }
    }

    fn step_interval(&self) -> f32 {
        let alive = self.alive_count().max(1) as f32;
        let total = (ROWS * COLS) as f32;
        let wave_pressure = self.wave.saturating_sub(1) as f32 * 0.07;
        let base = (0.72 - wave_pressure).max(0.16);
        (base * alive / total).max(0.045)
    }

    fn check_landed(&mut self) {
        let Some((_, _, bottom)) = self.bounds() else {
            return;
        };
        if bottom >= PLAYER_Y - 2.0 {
            self.invaded = true;
            self.lives = 0;
            self.kill_player();
        }
    }

    fn enemy_fire(&mut self, dt: f32) {
        self.fire_timer -= dt;
        if self.fire_timer > 0.0 {
            return;
        }
        let cap = match self.wave {
            1 => 1,
            2 => 2,
            _ => 3,
        };
        self.fire_timer = (1.15 - self.wave as f32 * 0.1).max(0.34);
        let enemy_shots = self.bullets.iter().filter(|bullet| !bullet.from_player).count();
        if enemy_shots >= cap {
            return;
        }
        let Some((x, y)) = self.random_shooter_pos() else {
            return;
        };
        let speed = 140.0 + self.wave as f32 * 18.0;
        self.bullets.push(Bullet {
            x: x - 1.5,
            y,
            w: 3.0,
            h: 8.0,
            vy: speed,
            from_player: false,
        });
    }

    fn update_ufo(&mut self, dt: f32) {
        let gone = if let Some(ufo) = self.ufo.as_mut() {
            ufo.x += ufo.dir * UFO_SPEED * dt;
            if ufo.dir > 0.0 {
                ufo.x > FIELD_W + 4.0
            } else {
                ufo.x + UFO_W < -4.0
            }
        } else {
            false
        };
        if self.ufo.is_some() {
            if gone {
                self.ufo = None;
                self.events.push(Sfx::UfoStop);
            }
            return;
        }

        self.ufo_timer -= dt;
        if self.ufo_timer > 0.0 {
            return;
        }
        self.ufo_timer = 12.0 + self.rng.gen(9) as f32;
        let from_left = self.rng.gen(2) == 0;
        let score = UFO_SCORES[(self.player_shots as usize) % UFO_SCORES.len()];
        self.ufo = Some(Ufo {
            x: if from_left { -UFO_W } else { FIELD_W },
            dir: if from_left { 1.0 } else { -1.0 },
            score,
        });
        self.events.push(Sfx::UfoLoop);
    }

    fn random_shooter_pos(&mut self) -> Option<(f32, f32)> {
        let mut count = 0u32;
        for col in 0..COLS {
            if (0..ROWS).any(|row| self.alive[row][col]) {
                count += 1;
            }
        }
        if count == 0 {
            return None;
        }
        let pick = self.rng.gen(count);
        let mut seen = 0u32;
        for col in 0..COLS {
            if !(0..ROWS).any(|row| self.alive[row][col]) {
                continue;
            }
            if seen == pick {
                for row in (0..ROWS).rev() {
                    if self.alive[row][col] {
                        let (x, y) = self.pos(col, row);
                        return Some((x + INV_W * 0.5, y + INV_H - 2.0));
                    }
                }
            }
            seen += 1;
        }
        None
    }

    fn kill_player(&mut self) {
        if self.mode == Mode::Dying || self.mode == Mode::GameOver {
            return;
        }
        if self.lives > 0 {
            self.lives -= 1;
        }
        self.mode = Mode::Dying;
        self.timer = 1.45;
        if self.ufo.take().is_some() {
            self.events.push(Sfx::UfoStop);
        }
        self.booms.push(Boom {
            x: self.player_x + PLAYER_W * 0.5,
            y: PLAYER_Y + PLAYER_H * 0.5,
            t: 0.0,
            kind: BoomKind::Player,
        });
        self.events.push(Sfx::PlayerDie);
        self.shake = 1.0;
        self.bullets.clear();
    }

    fn add_score(&mut self, pts: u32) {
        self.score = self.score.saturating_add(pts);
        if self.score > self.high_score {
            self.high_score = self.score;
            self.high_score_dirty = true;
        }
    }

    pub fn pos(&self, col: usize, row: usize) -> (f32, f32) {
        (
            self.origin_x + col as f32 * PITCH_X,
            self.origin_y + row as f32 * PITCH_Y,
        )
    }

    pub fn alive_count(&self) -> usize {
        self.alive.iter().flatten().filter(|alive| **alive).count()
    }

    fn bounds(&self) -> Option<(f32, f32, f32)> {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut bottom = f32::MIN;
        let mut any = false;
        for row in 0..ROWS {
            for col in 0..COLS {
                if !self.alive[row][col] {
                    continue;
                }
                any = true;
                let (x, y) = self.pos(col, row);
                min_x = min_x.min(x);
                max_x = max_x.max(x + INV_W);
                bottom = bottom.max(y + INV_H);
            }
        }
        any.then_some((min_x, max_x, bottom))
    }

    fn find_invader(&self, bullet: &Bullet) -> Option<(usize, usize)> {
        for row in 0..ROWS {
            for col in 0..COLS {
                if !self.alive[row][col] {
                    continue;
                }
                let (x, y) = self.pos(col, row);
                if overlaps(bullet, x, y, INV_W, INV_H) {
                    return Some((row, col));
                }
            }
        }
        None
    }

    fn player_overlaps(&self, bullet: &Bullet) -> bool {
        overlaps(
            bullet,
            self.player_x + 3.0,
            PLAYER_Y + 2.0,
            PLAYER_W - 6.0,
            PLAYER_H - 3.0,
        )
    }

    fn damage_bunkers(&mut self, bullet: &Bullet) -> bool {
        for bunker in &mut self.bunkers {
            let width = BUNKER_W as f32 * CELL;
            let height = BUNKER_H as f32 * CELL;
            if !overlaps(bullet, bunker.x, bunker.y, width, height) {
                continue;
            }
            let x0 = cell_index(bullet.x - bunker.x, BUNKER_W);
            let x1 = cell_index(bullet.x + bullet.w - 0.01 - bunker.x, BUNKER_W);
            let y0 = cell_index(bullet.y - bunker.y, BUNKER_H);
            let y1 = cell_index(bullet.y + bullet.h - 0.01 - bunker.y, BUNKER_H);
            let mut y = if bullet.from_player { y1 } else { y0 };
            let end = if bullet.from_player { y0 } else { y1 };
            let step: i32 = if bullet.from_player { -1 } else { 1 };
            loop {
                for x in x0..=x1 {
                    if bunker.cells[y as usize][x as usize] {
                        erode(bunker, x, y);
                        return true;
                    }
                }
                if y == end {
                    break;
                }
                y += step;
            }
        }
        false
    }

    fn erode_bunkers_under_invaders(&mut self) {
        for row in 0..ROWS {
            for col in 0..COLS {
                if !self.alive[row][col] {
                    continue;
                }
                let (ix, iy) = self.pos(col, row);
                for bunker in &mut self.bunkers {
                    for cy in 0..BUNKER_H {
                        for cx in 0..BUNKER_W {
                            if !bunker.cells[cy][cx] {
                                continue;
                            }
                            let x = bunker.x + cx as f32 * CELL;
                            let y = bunker.y + cy as f32 * CELL;
                            if overlaps_raw(ix, iy, INV_W, INV_H, x, y, CELL, CELL) {
                                bunker.cells[cy][cx] = false;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn cell_index(local: f32, limit: usize) -> i32 {
    let index = (local / CELL).floor() as i32;
    index.clamp(0, limit as i32 - 1)
}

fn erode(bunker: &mut Bunker, cx: i32, cy: i32) {
    for dy in -2..=2 {
        for dx in -2..=2 {
            if dx * dx + dy * dy > 5 {
                continue;
            }
            let x = cx + dx;
            let y = cy + dy;
            if x >= 0 && y >= 0 && (x as usize) < BUNKER_W && (y as usize) < BUNKER_H {
                bunker.cells[y as usize][x as usize] = false;
            }
        }
    }
}

fn overlaps(bullet: &Bullet, x: f32, y: f32, w: f32, h: f32) -> bool {
    overlaps_raw(bullet.x, bullet.y, bullet.w, bullet.h, x, y, w, h)
}

fn overlaps_raw(ax: f32, ay: f32, aw: f32, ah: f32, bx: f32, by: f32, bw: f32, bh: f32) -> bool {
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

fn bunker_mask() -> [&'static str; BUNKER_H] {
    [
        "......##########......",
        "....##############....",
        "...################...",
        "..##################..",
        ".####################.",
        "######################",
        "######################",
        "######################",
        "######################",
        "######################",
        "#####............#####",
        "####..............####",
        "###................###",
        "##..................##",
        "#....................#",
        "#....................#",
    ]
}

fn make_bunkers() -> Vec<Bunker> {
    let mask = bunker_mask();
    let width = BUNKER_W as f32 * CELL;
    let gap = (FIELD_W - width * 4.0) / 5.0;
    let y = PLAYER_Y - 92.0;
    (0..4)
        .map(|index| {
            let mut cells = [[false; BUNKER_W]; BUNKER_H];
            for (row, line) in mask.iter().enumerate() {
                for (col, ch) in line.chars().enumerate() {
                    cells[row][col] = ch == '#';
                }
            }
            Bunker {
                x: gap + index as f32 * (width + gap),
                y,
                cells,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn playing(seed: u64) -> Game {
        let mut game = Game::new(seed, 0);
        game.start_game();
        game.mode = Mode::Playing;
        game.timer = 0.0;
        game
    }

    #[test]
    fn bunker_mask_is_a_full_grid() {
        let rows = bunker_mask();
        assert_eq!(rows.len(), BUNKER_H);
        for row in rows {
            assert_eq!(row.chars().count(), BUNKER_W);
        }
    }

    #[test]
    fn player_stays_inside_the_field() {
        let mut game = playing(1);
        game.player_x = 0.0;
        game.update(0.1, &Input { left: true, ..Input::default() });
        assert!((game.player_x - MARGIN).abs() < 0.01);

        game.player_x = FIELD_W;
        game.update(0.1, &Input { right: true, ..Input::default() });
        let max_x = FIELD_W - MARGIN - PLAYER_W;
        assert!((game.player_x - max_x).abs() < 0.01);
    }

    #[test]
    fn only_one_player_bullet_is_allowed() {
        let mut game = playing(1);
        let fire = Input { fire_held: true, ..Input::default() };
        game.update(0.016, &fire);
        game.update(0.016, &fire);
        let shots = game.bullets.iter().filter(|bullet| bullet.from_player).count();
        assert_eq!(shots, 1);
    }

    #[test]
    fn shot_scores_depend_on_the_row() {
        let mut game = playing(1);
        let mut expected = 0;
        for (row, pts) in [(0, 30), (2, 20), (4, 10)] {
            let (x, y) = game.pos(0, row);
            game.bullets.push(Bullet {
                x: x + 2.0,
                y: y + 2.0,
                w: 4.0,
                h: 4.0,
                vy: 0.0,
                from_player: true,
            });
            game.update(0.0, &Input::default());
            expected += pts;
            assert!(!game.alive[row][0]);
            assert_eq!(game.score, expected);
        }
        assert_eq!(game.high_score, expected);
        assert!(game.high_score_dirty);
    }

    #[test]
    fn invaders_drop_and_reverse_at_the_edge() {
        let mut game = playing(1);
        let (_, max_x, _) = game.bounds().unwrap();
        game.origin_x += FIELD_W - MARGIN - max_x - 1.0;
        game.dir = 1.0;
        game.step_acc = 10.0;
        let y = game.origin_y;
        game.update(1.0 / 60.0, &Input::default());
        assert!(game.dir < 0.0);
        assert!((game.origin_y - (y + STEP_Y)).abs() < 0.01);
    }

    #[test]
    fn bunker_stops_a_bullet_and_loses_pixels() {
        let mut game = playing(1);
        let bunker = &game.bunkers[0];
        let mut found = None;
        for row in (0..BUNKER_H).rev() {
            for col in 0..BUNKER_W {
                if bunker.cells[row][col] {
                    found = Some((col, row));
                    break;
                }
            }
            if found.is_some() {
                break;
            }
        }
        let (col, row) = found.expect("bunker has a solid cell");
        let x = game.bunkers[0].x + col as f32 * CELL;
        let y = game.bunkers[0].y + row as f32 * CELL;
        game.bullets.push(Bullet {
            x,
            y,
            w: CELL,
            h: CELL,
            vy: -10.0,
            from_player: true,
        });
        game.update(0.0, &Input::default());
        assert!(!game.bunkers[0].cells[row][col]);
        assert!(game.bullets.is_empty());
    }

    #[test]
    fn clearing_the_formation_starts_the_next_wave() {
        let mut game = playing(1);
        game.alive = [[false; COLS]; ROWS];
        game.score = 40;
        game.update(0.0, &Input::default());
        assert_eq!(game.mode, Mode::WaveClear);
        game.timer = 0.01;
        game.update(0.05, &Input::default());
        assert_eq!(game.wave, 2);
        assert_eq!(game.mode, Mode::Intro);
        assert_eq!(game.alive_count(), ROWS * COLS);
        assert_eq!(game.score, 40);
    }

    #[test]
    fn three_hits_ends_the_run() {
        let mut game = playing(1);
        for _ in 0..3 {
            game.mode = Mode::Playing;
            game.invuln = 0.0;
            game.bullets.push(Bullet {
                x: game.player_x + 4.0,
                y: PLAYER_Y + 2.0,
                w: 3.0,
                h: 6.0,
                vy: 30.0,
                from_player: false,
            });
            game.update(0.0, &Input::default());
            if game.lives > 0 {
                game.timer = 0.01;
                game.mode = Mode::Dying;
                game.update(0.05, &Input::default());
                assert_eq!(game.mode, Mode::Playing);
            }
        }
        game.timer = 0.01;
        game.update(0.05, &Input::default());
        assert_eq!(game.lives, 0);
        assert_eq!(game.mode, Mode::GameOver);
    }

    #[test]
    fn invaders_that_reach_the_cannon_end_the_run() {
        let mut game = playing(1);
        game.origin_y = PLAYER_Y;
        game.step_acc = 10.0;
        game.update(1.0 / 60.0, &Input::default());
        assert!(game.invaded);
        assert_eq!(game.lives, 0);
        assert_eq!(game.mode, Mode::Dying);
    }

    #[test]
    fn a_minute_of_play_does_not_panic() {
        let mut game = playing(7);
        let input = Input {
            right: true,
            fire_held: true,
            ..Input::default()
        };
        for frame in 0..3600 {
            game.update(1.0 / 60.0, &input);
            if frame % 120 == 0 {
                game.events.clear();
            }
            assert!(game.player_x.is_finite());
            assert!(game.lives >= 0);
            assert!(game.high_score >= game.score);
        }
    }
}
