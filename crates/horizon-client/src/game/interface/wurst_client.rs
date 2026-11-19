//! Connects the native Rust Wurst Client (`horizon_client::wurst`) into `Game`:
//! per-tick hack execution, `.command` chat interception, keybinds, and HUD/ClickGUI rendering.
use super::*;
use horizon_client::text::Text;
use horizon_client::wurst::{Category, CommandAction, HackId, UiSettingId};

fn keycode_name(key: KeyCode) -> Option<&'static str> {
    Some(match key {
        KeyCode::KeyA => "A",
        KeyCode::KeyB => "B",
        KeyCode::KeyC => "C",
        KeyCode::KeyD => "D",
        KeyCode::KeyE => "E",
        KeyCode::KeyF => "F",
        KeyCode::KeyG => "G",
        KeyCode::KeyH => "H",
        KeyCode::KeyI => "I",
        KeyCode::KeyJ => "J",
        KeyCode::KeyK => "K",
        KeyCode::KeyL => "L",
        KeyCode::KeyM => "M",
        KeyCode::KeyN => "N",
        KeyCode::KeyO => "O",
        KeyCode::KeyP => "P",
        KeyCode::KeyQ => "Q",
        KeyCode::KeyR => "R",
        KeyCode::KeyS => "S",
        KeyCode::KeyT => "T",
        KeyCode::KeyU => "U",
        KeyCode::KeyV => "V",
        KeyCode::KeyW => "W",
        KeyCode::KeyX => "X",
        KeyCode::KeyY => "Y",
        KeyCode::KeyZ => "Z",
        _ => return None,
    })
}

impl Game<'_> {
    /// Runs every client tick when Wurst is active in this Horizon instance.
    pub(super) fn tick_wurst(&mut self) {
        if !self.wurst.active || self.title.is_some() {
            return;
        }

        // Fullbright (FullbrightHack.java)
        self.hold_fullbright();

        let jump_down = self.movement_key_held(KeyCode::Space);
        let shift_down = self.movement_key_held(KeyCode::ShiftLeft)
            || self.movement_key_held(KeyCode::ShiftRight);
        let forward_down = self.movement_key_held(KeyCode::KeyW);
        let back_down = self.movement_key_held(KeyCode::KeyS);
        let left_down = self.movement_key_held(KeyCode::KeyA);
        let right_down = self.movement_key_held(KeyCode::KeyD);
        let moving = forward_down || back_down || left_down || right_down;
        let jump_just_pressed = jump_down && !self.wurst.was_jump_held;
        self.wurst.was_jump_held = jump_down;

        let yaw_rad = (self.yaw as f64).to_radians();
        let fwd_x = -yaw_rad.sin();
        let fwd_z = yaw_rad.cos();
        let right_x = -fwd_z;
        let right_z = fwd_x;

        let mut move_fwd = 0.0_f64;
        let mut move_strafe = 0.0_f64;
        if forward_down {
            move_fwd += 1.0;
        }
        if back_down {
            move_fwd -= 1.0;
        }
        if right_down {
            move_strafe += 1.0;
        }
        if left_down {
            move_strafe -= 1.0;
        }
        let dir_len = (move_fwd * move_fwd + move_strafe * move_strafe).sqrt();
        let (dir_x, dir_z) = if dir_len > 1e-6 {
            let nf = move_fwd / dir_len;
            let ns = move_strafe / dir_len;
            (fwd_x * nf + right_x * ns, fwd_z * nf + right_z * ns)
        } else {
            (0.0, 0.0)
        };

        let Some(player) = self.player.as_mut() else {
            return;
        };

        // CreativeFlightHack.java
        if self.wurst.is_enabled(HackId::CreativeFlight) {
            if self.wurst.cfly_was_may_fly.is_none() {
                self.wurst.cfly_was_may_fly = Some(player.may_fly);
            }
            player.may_fly = true;
        } else if let Some(prev) = self.wurst.cfly_was_may_fly.take() {
            player.may_fly = prev;
            if !prev {
                player.flying = false;
            }
        }

        // NoClipHack.java
        if self.wurst.is_enabled(HackId::NoClip) {
            if self.wurst.noclip_was_spectator.is_none() {
                self.wurst.noclip_was_spectator = Some(player.spectator);
            }
            player.spectator = true;
            player.flying = true;
        } else if let Some(prev) = self.wurst.noclip_was_spectator.take() {
            player.spectator = prev;
        }

        // FlightHack.java: zero delta movement + horizontal/vertical speed + anti-kick
        if self.wurst.is_enabled(HackId::Flight) {
            player.flying = false;
            let h_speed = self.wurst.settings.flight_horizontal_speed;
            let v_speed = self.wurst.flight_actual_vertical_speed(player.invulnerable);
            let mut vy = 0.0;
            if jump_down {
                vy += v_speed;
            }
            if shift_down {
                vy -= v_speed;
            }
            player.velocity = Vec3::new(dir_x * h_speed, vy, dir_z * h_speed);
            player.fall_distance = 0.0;

            if self.wurst.settings.flight_anti_kick {
                let interval = self.wurst.settings.flight_anti_kick_interval;
                let dist = self.wurst.settings.flight_anti_kick_distance;
                if self.wurst.flight_tick_counter > interval + 1 {
                    self.wurst.flight_tick_counter = 0;
                }
                match self.wurst.flight_tick_counter {
                    0 => {
                        if player.velocity.y <= -dist {
                            self.wurst.flight_tick_counter = 2;
                        } else {
                            player.velocity.y = -dist;
                        }
                    }
                    1 => {
                        player.velocity.y = dist;
                    }
                    _ => {}
                }
                self.wurst.flight_tick_counter += 1;
            }
        }

        // JetpackHack.java
        if self.wurst.is_enabled(HackId::Jetpack) && jump_down {
            player.velocity.y = 0.42;
        }

        // AirJumpHack.java
        if self.wurst.is_enabled(HackId::AirJump) && jump_just_pressed && !player.ground.on_ground {
            player.velocity.y = 0.42 + self.wurst.highjump_added_power();
            player.fall_distance = 0.0;
        }

        // HighJumpHack.java
        if self.wurst.is_enabled(HackId::HighJump) && jump_just_pressed && player.ground.on_ground {
            let added = self.wurst.highjump_added_power();
            if added > 0.0 {
                player.velocity.y = 0.42 + added;
            }
        }

        // AutoSprintHack.java
        if self.wurst.is_enabled(HackId::AutoSprint)
            && !player.collision.horizontal
            && !shift_down
            && !player.swimming
            && moving
            && (self.wurst.settings.autosprint_omni || forward_down)
        {
            player.sprinting = true;
        }

        // NoFallHack.java
        if self.wurst.is_enabled(HackId::NoFall)
            && player.fall_distance >= self.wurst.settings.nofall_min_fall_distance
        {
            player.fall_distance = 0.0;
        }

        // SpiderHack.java
        if self.wurst.is_enabled(HackId::Spider)
            && player.collision.horizontal
            && player.velocity.y < 0.2
        {
            player.velocity.y = 0.2;
        }

        // GlideHack.java
        if self.wurst.is_enabled(HackId::Glide)
            && !player.ground.on_ground
            && player.velocity.y < -self.wurst.settings.glide_fall_speed
            && !(self.wurst.settings.glide_pause_on_sneak && shift_down)
        {
            player.velocity.y = -self.wurst.settings.glide_fall_speed;
            let factor = self.wurst.settings.glide_move_speed;
            if moving {
                player.velocity.x *= factor.min(1.15);
                player.velocity.z *= factor.min(1.15);
            }
        }

        // SpeedHackHack.java
        if self.wurst.is_enabled(HackId::SpeedHack)
            && !self.wurst.is_enabled(HackId::Flight)
            && moving
            && !shift_down
        {
            let target = 0.28 * self.wurst.settings.speed_multiplier;
            let cur = (player.velocity.x * player.velocity.x + player.velocity.z * player.velocity.z)
                .sqrt();
            if cur < target {
                player.velocity.x = dir_x * target;
                player.velocity.z = dir_z * target;
            }
        }

        // BunnyHopHack.java
        if self.wurst.is_enabled(HackId::BunnyHop)
            && moving
            && player.ground.on_ground
            && !shift_down
        {
            player.velocity.y = 0.42 + self.wurst.highjump_added_power();
        }

        // DolphinHack.java
        if self.wurst.is_enabled(HackId::Dolphin)
            && player.fluids.was_touching_water
            && !shift_down
            && player.velocity.y < 0.15
        {
            player.velocity.y += 0.04;
        }

        // JesusHack.java
        if self.wurst.is_enabled(HackId::Jesus)
            && (player.fluids.was_touching_water || player.fluids.was_eye_in_water)
            && !shift_down
        {
            if player.velocity.y < 0.0 {
                player.velocity.y = 0.0;
            }
            player.velocity.y = player.velocity.y.max(0.08);
            player.fall_distance = 0.0;
        }

        // NoSlowdownHack.java
        if self.wurst.is_enabled(HackId::NoSlowdown) {
            player.item_use_speed = None;
            player.item_use_slows_sprint = false;
            player.stuck_multiplier = Vec3::ZERO;
        }

        // AntiBlindHack.java
        if self.wurst.is_enabled(HackId::AntiBlind) {
            player.mobility_restricted = false;
        }

        // StepHack.java: lift cleanly onto 1-block ledges on horizontal collision while grounded
        if self.wurst.is_enabled(HackId::Step)
            && player.collision.horizontal
            && player.ground.on_ground
            && moving
        {
            player.position.y += self.wurst.settings.step_height.min(1.0);
        }

        // SafeWalkHack.java: mimic sneak-edge clamping while walking on ground
        if self.wurst.is_enabled(HackId::SafeWalk) && player.ground.on_ground && !shift_down {
            player.local_crouching = true;
        }

        // ParkourHack.java: auto-jump if moving forward near a drop
        if self.wurst.is_enabled(HackId::Parkour)
            && player.ground.on_ground
            && moving
            && !shift_down
            && player.velocity.y <= 0.0
        {
            let ahead_x = (player.position.x + dir_x * 0.45).floor() as i32;
            let ahead_y = (player.position.y - 0.5).floor() as i32;
            let ahead_z = (player.position.z + dir_z * 0.45).floor() as i32;
            if self.world.block(ahead_x, ahead_y, ahead_z).0 == 0 {
                player.velocity.y = 0.42 + self.wurst.highjump_added_power();
            }
        }
    }

    /// Handles a `.`-prefixed Wurst chat command. Returns `true` if consumed.
    pub(super) fn handle_wurst_command(&mut self, message: &str) -> bool {
        if !self.wurst.active || !message.starts_with('.') {
            return false;
        }
        let actions = self.wurst.run_command(message);
        for action in actions {
            match action {
                CommandAction::Message(msg) => {
                    self.system_message(Text::literal(msg).with_color(0x55_FF55));
                }
                CommandAction::Error(err) => {
                    self.system_message(Text::literal(err).with_color(0xFF_5555));
                }
                CommandAction::VClip(dy) => {
                    if let Some(p) = self.player.as_mut() {
                        p.position.y += dy;
                        p.fall_distance = 0.0;
                        p.needs_sync = true;
                    }
                }
                CommandAction::Teleport(x, y, z) => {
                    if let Some(p) = self.player.as_mut() {
                        p.position = Vec3::new(x, y, z);
                        p.fall_distance = 0.0;
                        p.needs_sync = true;
                    }
                }
                CommandAction::Say(text) => {
                    if self.remote.is_some() {
                        self.send_chat_from_screen(&text);
                    } else {
                        self.system_message(Text::literal(text));
                    }
                }
                CommandAction::ClearChat => {
                    self.ui.chat.clear();
                }
                CommandAction::GameMode(mode) => {
                    self.creative = mode == 1;
                    if let Some(p) = self.player.as_mut() {
                        p.may_fly = mode == 1 || mode == 3;
                        p.invulnerable = mode == 1 || mode == 3;
                        p.spectator = mode == 3;
                        if mode == 3 {
                            p.flying = true;
                        }
                    }
                    self.system_message(
                        Text::literal(format!("[Wurst] Local gamemode set to {mode}."))
                            .with_color(0x55_FF55),
                    );
                }
            }
        }
        true
    }

    /// Handles Wurst keybinds and the Right-Shift ClickGUI when playing.
    /// Returns `true` if the key was consumed by the Wurst menu.
    pub(super) fn handle_wurst_key(&mut self, key: KeyCode) -> bool {
        if !self.wurst.active || self.title.is_some() || self.screen != Screen::Playing {
            return false;
        }
        if key == KeyCode::ShiftRight {
            self.wurst.menu_open = !self.wurst.menu_open;
            self.wurst.gui_dragging = None;
            self.capture(!self.wurst.menu_open);
            return true;
        }
        if self.wurst.menu_open {
            if key == KeyCode::Escape {
                self.wurst.menu_open = false;
                self.wurst.gui_dragging = None;
                self.capture(true);
                return true;
            }
            return true;
        }
        // TabGui.java arrow key navigation
        if self.wurst.settings.show_tabgui {
            let cats = Category::ALL;
            match key {
                KeyCode::ArrowDown => {
                    if !self.wurst.tab_opened {
                        self.wurst.tab_selected = (self.wurst.tab_selected + 1) % cats.len();
                        self.wurst.tab_feature_selected = 0;
                    } else {
                        let n = Self::window_hacks(cats[self.wurst.tab_selected]).len().max(1);
                        self.wurst.tab_feature_selected = (self.wurst.tab_feature_selected + 1) % n;
                    }
                    return true;
                }
                KeyCode::ArrowUp => {
                    if !self.wurst.tab_opened {
                        self.wurst.tab_selected = (self.wurst.tab_selected + cats.len() - 1) % cats.len();
                        self.wurst.tab_feature_selected = 0;
                    } else {
                        let n = Self::window_hacks(cats[self.wurst.tab_selected]).len().max(1);
                        self.wurst.tab_feature_selected = (self.wurst.tab_feature_selected + n - 1) % n;
                    }
                    return true;
                }
                KeyCode::ArrowRight if !self.wurst.tab_opened => {
                    self.wurst.tab_opened = true;
                    self.wurst.tab_feature_selected = 0;
                    return true;
                }
                KeyCode::ArrowLeft if self.wurst.tab_opened => {
                    self.wurst.tab_opened = false;
                    return true;
                }
                KeyCode::Enter if self.wurst.tab_opened => {
                    let hacks = Self::window_hacks(cats[self.wurst.tab_selected]);
                    if let Some(&id) = hacks.get(self.wurst.tab_feature_selected) {
                        self.wurst.toggle(id);
                    }
                    return true;
                }
                _ => {}
            }
        }
        if let Some(name) = keycode_name(key)
            && let Some(&id) = self.wurst.binds.get(name)
        {
            let on = self.wurst.toggle(id);
            self.system_message(
                Text::literal(format!(
                    "[Wurst] {} {}",
                    id.name(),
                    if on { "ON" } else { "OFF" }
                ))
                .with_color(if on { 0x55_FF55 } else { 0xAA_AAAA }),
            );
            return true;
        }
        false
    }

    pub(super) fn handle_wurst_mouse_move(&mut self, x: f32, y: f32) {
        self.wurst.gui_cursor = [x, y];
        if let Some((idx, ox, oy)) = self.wurst.gui_dragging
            && let Some(win) = self.wurst.gui_windows.get_mut(idx)
        {
            win.x = (x - ox).max(0.0);
            win.y = (y - oy).max(0.0);
        }
    }

    fn window_hacks(cat: Category) -> Vec<HackId> {
        HackId::ALL
            .iter()
            .copied()
            .filter(|h| h.category() == cat)
            .collect()
    }

    fn ui_setting_label(&self, s: UiSettingId) -> String {
        match s {
            UiSettingId::WurstLogo => format!(
                "WurstLogo: {}",
                if self.wurst.settings.show_watermark { "ON" } else { "OFF" }
            ),
            UiSettingId::HackList => format!(
                "HackList: {}",
                if self.wurst.settings.show_hacklist { "ON" } else { "OFF" }
            ),
            UiSettingId::TabGui => format!(
                "TabGUI: {}",
                if self.wurst.settings.show_tabgui { "ON" } else { "OFF" }
            ),
            UiSettingId::FlightHSpeed => {
                format!("Flight H: {:.2}", self.wurst.settings.flight_horizontal_speed)
            }
            UiSettingId::FlightVSpeed => {
                format!("Flight V: {:.2}", self.wurst.settings.flight_vertical_speed)
            }
            UiSettingId::FlightAntiKick => format!(
                "Anti-Kick: {}",
                if self.wurst.settings.flight_anti_kick { "ON" } else { "OFF" }
            ),
            UiSettingId::SpeedMultiplier => {
                format!("Speed: {:.2}x", self.wurst.settings.speed_multiplier)
            }
            UiSettingId::HighJumpHeight => {
                format!("HighJump: {:.0}m", self.wurst.settings.highjump_height)
            }
            UiSettingId::StepHeight => {
                format!("Step: {:.1}m", self.wurst.settings.step_height)
            }
            UiSettingId::TimerSpeed => {
                format!("Timer: {:.2}x", self.wurst.settings.timer_speed)
            }
            UiSettingId::GlideFallSpeed => {
                format!("Glide: {:.3}", self.wurst.settings.glide_fall_speed)
            }
        }
    }

    fn ui_setting_active(&self, s: UiSettingId) -> bool {
        match s {
            UiSettingId::WurstLogo => self.wurst.settings.show_watermark,
            UiSettingId::HackList => self.wurst.settings.show_hacklist,
            UiSettingId::TabGui => self.wurst.settings.show_tabgui,
            UiSettingId::FlightAntiKick => self.wurst.settings.flight_anti_kick,
            _ => false,
        }
    }

    fn step_ui_setting(&mut self, s: UiSettingId, delta: f64) {
        match s {
            UiSettingId::WurstLogo => {
                self.wurst.settings.show_watermark = !self.wurst.settings.show_watermark;
            }
            UiSettingId::HackList => {
                self.wurst.settings.show_hacklist = !self.wurst.settings.show_hacklist;
            }
            UiSettingId::TabGui => {
                self.wurst.settings.show_tabgui = !self.wurst.settings.show_tabgui;
            }
            UiSettingId::FlightHSpeed => {
                let v = self.wurst.settings.flight_horizontal_speed + delta * 0.25;
                self.wurst.settings.flight_horizontal_speed = if v > 10.0 { 0.25 } else { v.clamp(0.05, 10.0) };
            }
            UiSettingId::FlightVSpeed => {
                let v = self.wurst.settings.flight_vertical_speed + delta * 0.25;
                self.wurst.settings.flight_vertical_speed = if v > 5.0 { 0.25 } else { v.clamp(0.05, 5.0) };
            }
            UiSettingId::FlightAntiKick => {
                self.wurst.settings.flight_anti_kick = !self.wurst.settings.flight_anti_kick;
            }
            UiSettingId::SpeedMultiplier => {
                let v = self.wurst.settings.speed_multiplier + delta * 0.25;
                self.wurst.settings.speed_multiplier = if v > 5.0 { 1.0 } else { v.clamp(1.0, 5.0) };
            }
            UiSettingId::HighJumpHeight => {
                let v = self.wurst.settings.highjump_height + delta * 1.0;
                self.wurst.settings.highjump_height = if v > 20.0 { 1.0 } else { v.clamp(1.0, 100.0) };
            }
            UiSettingId::StepHeight => {
                let v = self.wurst.settings.step_height + delta * 0.5;
                self.wurst.settings.step_height = if v > 3.0 { 0.5 } else { v.clamp(0.5, 10.0) };
            }
            UiSettingId::TimerSpeed => {
                let v = self.wurst.settings.timer_speed + delta * 0.25;
                self.wurst.settings.timer_speed = if v > 5.0 { 0.5 } else { v.clamp(0.1, 10.0) };
            }
            UiSettingId::GlideFallSpeed => {
                let v = self.wurst.settings.glide_fall_speed + delta * 0.025;
                self.wurst.settings.glide_fall_speed = if v > 0.25 { 0.025 } else { v.clamp(0.005, 0.25) };
            }
        }
        self.wurst.save();
    }

    pub(super) fn handle_wurst_mouse(&mut self, button: MouseButton, pressed: bool) -> bool {
        if !self.wurst.active || !self.wurst.menu_open {
            return false;
        }
        if !pressed {
            if button == MouseButton::Left {
                self.wurst.gui_dragging = None;
            }
            return true;
        }
        let [mx, my] = self.wurst.gui_cursor;
        let title_h = 13.0_f32;
        let row_h = 13.0_f32;

        // Check top-most window first (reverse draw order)
        for idx in (0..self.wurst.gui_windows.len()).rev() {
            let win = self.wurst.gui_windows[idx].clone();
            if mx < win.x || mx > win.x + win.width || my < win.y {
                continue;
            }
            // Title bar
            if my <= win.y + title_h {
                if button == MouseButton::Left {
                    if mx >= win.x + win.width - 12.0 {
                        self.wurst.gui_windows[idx].minimized = !win.minimized;
                    } else if mx >= win.x + win.width - 24.0 {
                        self.wurst.gui_windows[idx].pinned = !win.pinned;
                    } else {
                        self.wurst.gui_dragging = Some((idx, mx - win.x, my - win.y));
                    }
                } else if button == MouseButton::Right {
                    self.wurst.gui_windows[idx].minimized = !win.minimized;
                }
                return true;
            }
            if win.minimized {
                continue;
            }
            let body_y = win.y + title_h + 2.0;
            if let Some(cat) = win.category {
                let hacks = Self::window_hacks(cat);
                let body_h = (hacks.len() as f32) * row_h + 2.0;
                if my <= body_y + body_h {
                    let row = ((my - body_y) / row_h) as usize;
                    if let Some(&id) = hacks.get(row)
                        && button == MouseButton::Left
                    {
                        self.wurst.toggle(id);
                    }
                    return true;
                }
            } else {
                let body_h = (UiSettingId::ALL.len() as f32) * row_h + 2.0;
                if my <= body_y + body_h {
                    let row = ((my - body_y) / row_h) as usize;
                    if let Some(&s) = UiSettingId::ALL.get(row) {
                        let delta = if button == MouseButton::Right { -1.0 } else { 1.0 };
                        self.step_ui_setting(s, delta);
                    }
                    return true;
                }
            }
        }
        true
    }

    /// Draws Wurst7's `WurstLogo`, `HackListHUD`, and draggable `ClickGui` windows.
    pub(super) fn draw_wurst_hud(&self, batch: &mut HudBatch) {
        if !self.wurst.active {
            return;
        }
        let atlas = &self.gui.atlas;
        let (w, _h) = (batch.width, batch.height);

        // 1. WurstLogo.java: translucent white bar + WURST badge + version in black
        if self.wurst.settings.show_watermark {
            let ver = format!(
                "v{} MC{}",
                horizon_client::wurst::VERSION,
                horizon_client::wurst::MC_VERSION
            );
            let bar_w = 56.0 + (ver.len() as f32) * 5.8 + 8.0;
            batch.fill(atlas, 0.0, 6.0, bar_w, 12.0, 0x80_FFFFFF);
            batch.fill(atlas, 2.0, 4.0, 46.0, 15.0, 0xE8_FFFFFF);
            batch.text_styled(atlas, &Text::literal("WURST"), 7.0, 7.5, 0xFF_000000, false);
            batch.text_styled(atlas, &Text::literal(ver), 52.0, 8.0, 0xFF_000000, false);
        }

        // 2. TabGui.java: (2, 23), 0x7F_404040 bg, 0x7F_101010 border, 10px rows
        if self.wurst.settings.show_tabgui {
            let tx = 2.0_f32;
            let ty = 23.0_f32;
            let tw = 64.0_f32;
            let cats = Category::ALL;
            let th = (cats.len() as f32) * 10.0;
            batch.fill(atlas, tx - 1.0, ty - 1.0, tw + 2.0, th + 2.0, 0x7F_101010);
            batch.fill(atlas, tx, ty, tw, th, 0x7F_404040);
            for (i, cat) in cats.iter().enumerate() {
                let prefix = if i == self.wurst.tab_selected {
                    if self.wurst.tab_opened { "<" } else { ">" }
                } else {
                    ""
                };
                let label = format!("{prefix}{}", cat.name());
                batch.text_styled(
                    atlas,
                    &Text::literal(label),
                    tx + 2.0,
                    ty + 1.0 + (i as f32) * 10.0,
                    0xFF_F0F0F0,
                    false,
                );
            }
            if self.wurst.tab_opened {
                let hacks = Self::window_hacks(cats[self.wurst.tab_selected]);
                if !hacks.is_empty() {
                    let sx = tx + tw + 2.0;
                    let sw = 78.0_f32;
                    let sh = (hacks.len() as f32) * 10.0;
                    batch.fill(atlas, sx - 1.0, ty - 1.0, sw + 2.0, sh + 2.0, 0x7F_101010);
                    batch.fill(atlas, sx, ty, sw, sh, 0x7F_404040);
                    for (i, &id) in hacks.iter().enumerate() {
                        let prefix = if i == self.wurst.tab_feature_selected { ">" } else { "" };
                        let label = format!("{prefix}{}", id.name());
                        let color = if self.wurst.is_enabled(id) {
                            0xFF_55FF55
                        } else {
                            0xFF_F0F0F0
                        };
                        batch.text_styled(
                            atlas,
                            &Text::literal(label),
                            sx + 2.0,
                            ty + 1.0 + (i as f32) * 10.0,
                            color,
                            false,
                        );
                    }
                }
            }
        }

        // 3. HackListHUD.java: clean white text list top-right
        if self.wurst.settings.show_hacklist {
            let active_hacks = self.wurst.enabled_render_names();
            let mut y = 4.0_f32;
            for name in active_hacks {
                let text_w = (name.len() as f32) * 5.8;
                let x = (w - text_w - 4.0).max(4.0);
                batch.text_styled(atlas, &Text::literal(name), x, y, 0xFF_FFFFFF, true);
                y += 10.0;
            }
        }

        // 4. ClickGui.java + Window.java + FeatureButton.java (pinned windows stay visible!)
        let [mx, my] = self.wurst.gui_cursor;
        let title_h = 13.0_f32;
        let row_h = 12.0_f32;
        let mut tooltip: Option<&'static str> = None;

        for win in &self.wurst.gui_windows {
            if !self.wurst.menu_open && !win.pinned {
                continue;
            }
            // Wurst7 ClickGuiHack colors: acColor = 0x101010 border/title, bgColor = 0x404040 @ 0.5F
            batch.fill(atlas, win.x - 1.0, win.y - 1.0, win.width + 2.0, title_h + 2.0, 0x7F_101010);
            batch.fill(atlas, win.x, win.y, win.width, title_h, 0xBF_101010);
            batch.text_styled(
                atlas,
                &Text::literal(win.title),
                win.x + 4.0,
                win.y + 2.5,
                0xFF_F0F0F0,
                false,
            );
            let pin_color = if win.pinned { 0xFF_55FF55 } else { 0xFF_AAAAAA };
            batch.text_styled(
                atlas,
                &Text::literal("P"),
                win.x + win.width - 21.0,
                win.y + 2.5,
                pin_color,
                false,
            );
            let min_label = if win.minimized { "+" } else { "-" };
            batch.text_styled(
                atlas,
                &Text::literal(min_label),
                win.x + win.width - 10.0,
                win.y + 2.5,
                0xFF_F0F0F0,
                false,
            );

            if win.minimized {
                continue;
            }

            let body_y = win.y + title_h + 1.0;
            if let Some(cat) = win.category {
                let hacks = Self::window_hacks(cat);
                let body_h = (hacks.len() as f32) * row_h + 2.0;
                batch.fill(atlas, win.x - 1.0, body_y, win.width + 2.0, body_h + 1.0, 0x7F_101010);
                batch.fill(atlas, win.x, body_y, win.width, body_h, 0x7F_404040);

                for (i, &id) in hacks.iter().enumerate() {
                    let bx = win.x + 2.0;
                    let by = body_y + 1.0 + (i as f32) * row_h;
                    let bw = win.width - 4.0;
                    let bh = row_h - 1.0;
                    let hovered = self.wurst.menu_open
                        && mx >= bx && mx <= bx + bw && my >= by && my <= by + bh;
                    let on = self.wurst.is_enabled(id);
                    if hovered {
                        tooltip = Some(id.description());
                    }
                    let bg = match (on, hovered) {
                        (true, true) => 0xA0_00E000,
                        (true, false) => 0x80_00C000,
                        (false, true) => 0x90_606060,
                        (false, false) => 0x60_404040,
                    };
                    batch.fill(atlas, bx, by, bw, bh, bg);
                    batch.text_styled(
                        atlas,
                        &Text::literal(id.name()),
                        bx + 4.0,
                        by + 1.5,
                        0xFF_F0F0F0,
                        false,
                    );
                }
            } else {
                let body_h = (UiSettingId::ALL.len() as f32) * row_h + 2.0;
                batch.fill(atlas, win.x - 1.0, body_y, win.width + 2.0, body_h + 1.0, 0x7F_101010);
                batch.fill(atlas, win.x, body_y, win.width, body_h, 0x7F_404040);

                for (i, &s) in UiSettingId::ALL.iter().enumerate() {
                    let bx = win.x + 2.0;
                    let by = body_y + 1.0 + (i as f32) * row_h;
                    let bw = win.width - 4.0;
                    let bh = row_h - 1.0;
                    let hovered = self.wurst.menu_open
                        && mx >= bx && mx <= bx + bw && my >= by && my <= by + bh;
                    let on = self.ui_setting_active(s);
                    let bg = match (on, hovered) {
                        (true, true) => 0xA0_00E000,
                        (true, false) => 0x80_00C000,
                        (false, true) => 0x90_606060,
                        (false, false) => 0x60_404040,
                    };
                    batch.fill(atlas, bx, by, bw, bh, bg);
                    batch.text_styled(
                        atlas,
                        &Text::literal(self.ui_setting_label(s)),
                        bx + 4.0,
                        by + 1.5,
                        0xFF_F0F0F0,
                        false,
                    );
                }
            }
        }

        // Hover tooltip box (matching ClickGui.renderTooltip)
        if self.wurst.menu_open
            && let Some(tip) = tooltip
        {
            let tw = (tip.len() as f32) * 5.8 + 8.0;
            let tx = (mx + 8.0).min((w - tw - 4.0).max(4.0));
            let ty = my + 10.0;
            batch.fill(atlas, tx - 1.0, ty - 1.0, tw + 2.0, 14.0, 0x7F_101010);
            batch.fill(atlas, tx, ty, tw, 12.0, 0xC0_404040);
            batch.text_styled(atlas, &Text::literal(tip), tx + 4.0, ty + 2.0, 0xFF_F0F0F0, false);
        }
    }
}
