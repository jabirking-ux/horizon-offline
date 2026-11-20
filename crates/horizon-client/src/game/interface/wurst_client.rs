//! 1:1 port of Wurst7's `WurstLogo`, `TabGui`, `HackListHUD`, `ClickGui` (`Window`,
//! `ClickGuiIcons`, `FeatureButton`), and `WurstOptionsScreen` for `horizon-client`.
use super::*;
use horizon_client::hud::rgba;
use horizon_client::text::Text;
use horizon_client::wurst::{Category, CommandAction, HackId, UiSettingId, WURST_LOGO_RLE};

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

const MAX_INNER_HEIGHT: f32 = 200.0;

impl Game<'_> {
    /// Renders `wurst_128.png` (`72x18` base RLE) at `(x, y)` scaled by `(sx, sy)`.
    pub(super) fn draw_wurst_logo(
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        x: f32,
        y: f32,
        sx: f32,
        sy: f32,
    ) {
        for &(px, py, pw, color) in WURST_LOGO_RLE {
            batch.fill(
                atlas,
                x + (px as f32) * sx,
                y + (py as f32) * sy,
                (pw as f32) * sx,
                sy,
                color,
            );
        }
    }

    /// Runs every client tick when Wurst is active in this Horizon instance.
    pub(super) fn tick_wurst(&mut self) {
        if !self.wurst.active || self.title.is_some() {
            return;
        }

        self.hold_fullbright();
        self.tick_wurst_combat();

        let jump_down = self.movement_key_held(KeyCode::Space);
        let shift_down = self.movement_key_held(KeyCode::ShiftLeft)
            || self.movement_key_held(KeyCode::ShiftRight);
        let forward_down = self.movement_key_held(KeyCode::KeyW)
            || self.wurst.is_enabled(HackId::AutoWalk);
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

        if self.wurst.is_enabled(HackId::NoClip) || self.wurst.is_enabled(HackId::Freecam) {
            if self.wurst.noclip_was_spectator.is_none() {
                self.wurst.noclip_was_spectator = Some(player.spectator);
            }
            player.spectator = true;
            player.flying = true;
        } else if let Some(prev) = self.wurst.noclip_was_spectator.take() {
            player.spectator = prev;
        }

        if self.wurst.is_enabled(HackId::Flight) || self.wurst.is_enabled(HackId::BoatFly) {
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

        if self.wurst.is_enabled(HackId::Jetpack) && jump_down {
            player.velocity.y = 0.42;
        }

        if self.wurst.is_enabled(HackId::AirJump) && jump_just_pressed && !player.ground.on_ground {
            player.velocity.y = 0.42 + self.wurst.highjump_added_power();
            player.fall_distance = 0.0;
        }

        if self.wurst.is_enabled(HackId::HighJump) && jump_just_pressed && player.ground.on_ground {
            let added = self.wurst.highjump_added_power();
            if added > 0.0 {
                player.velocity.y = 0.42 + added;
            }
        }

        if self.wurst.is_enabled(HackId::AutoSprint)
            && !player.collision.horizontal
            && !shift_down
            && !player.swimming
            && moving
            && (self.wurst.settings.autosprint_omni || forward_down)
        {
            player.sprinting = true;
        }

        if self.wurst.is_enabled(HackId::AutoSwim) && player.fluids.was_touching_water && forward_down {
            player.swimming = true;
            player.sprinting = true;
        }

        if self.wurst.is_enabled(HackId::Sneak) {
            player.local_crouching = true;
            player.shift_key_down = true;
        }

        if self.wurst.is_enabled(HackId::NoFall)
            && player.fall_distance >= self.wurst.settings.nofall_min_fall_distance
        {
            player.fall_distance = 0.0;
        }

        if self.wurst.is_enabled(HackId::Spider)
            && player.collision.horizontal
            && player.velocity.y < 0.2
        {
            player.velocity.y = 0.2;
        }

        if self.wurst.is_enabled(HackId::FastLadder)
            && player.on_climbable
            && player.collision.horizontal
            && moving
        {
            player.velocity.y = 0.2872;
        }

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

        if self.wurst.is_enabled(HackId::BunnyHop)
            && moving
            && player.ground.on_ground
            && !shift_down
        {
            player.velocity.y = 0.42 + self.wurst.highjump_added_power();
        }

        if (self.wurst.is_enabled(HackId::Dolphin) || self.wurst.is_enabled(HackId::Fish))
            && player.fluids.was_touching_water
            && !shift_down
            && player.velocity.y < 0.15
        {
            player.velocity.y += 0.04;
        }

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

        if self.wurst.is_enabled(HackId::NoSlowdown) || self.wurst.is_enabled(HackId::NoWeb) {
            player.item_use_speed = None;
            player.item_use_slows_sprint = false;
            player.stuck_multiplier = Vec3::ZERO;
        }

        if self.wurst.is_enabled(HackId::SnowShoe) {
            player.leather_boots = true;
        }

        if self.wurst.is_enabled(HackId::AntiBlind) {
            player.mobility_restricted = false;
        }

        if self.wurst.is_enabled(HackId::Step)
            && player.collision.horizontal
            && player.ground.on_ground
            && moving
        {
            player.position.y += self.wurst.settings.step_height.min(1.0);
        }

        if self.wurst.is_enabled(HackId::SafeWalk) && player.ground.on_ground && !shift_down {
            player.local_crouching = true;
        }

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

    pub(super) fn handle_wurst_key(&mut self, key: KeyCode) -> bool {
        if !self.wurst.active || self.title.is_some() {
            return false;
        }
        if self.wurst.wurst_options_open {
            if key == KeyCode::Escape {
                self.wurst.wurst_options_open = false;
                if self.wurst.wurst_options_from_pause {
                    self.set_screen(Screen::Paused);
                } else {
                    self.capture(true);
                }
                return true;
            }
            return true;
        }
        if self.screen != Screen::Playing {
            return false;
        }
        if key == KeyCode::ShiftRight {
            self.wurst.navigator_open = !self.wurst.navigator_open;
            self.wurst.menu_open = false;
            self.wurst.gui_dragging = None;
            if self.wurst.navigator_open {
                self.wurst.navigator_query.clear();
                self.wurst.navigator_scroll = 0;
            }
            self.capture(!self.wurst.navigator_open);
            return true;
        }
        if key == KeyCode::ControlRight {
            self.wurst.menu_open = !self.wurst.menu_open;
            self.wurst.navigator_open = false;
            self.wurst.gui_dragging = None;
            self.capture(!self.wurst.menu_open);
            return true;
        }
        if self.wurst.navigator_open {
            match key {
                KeyCode::Escape | KeyCode::Backspace if self.wurst.navigator_feature.is_some() => {
                    self.wurst.navigator_feature = None;
                }
                KeyCode::Escape => {
                    self.wurst.navigator_open = false;
                    self.wurst.navigator_feature = None;
                    self.capture(true);
                }
                KeyCode::Backspace => {
                    self.wurst.navigator_query.pop();
                    self.wurst.navigator_scroll = 0;
                }
                KeyCode::Space => {
                    self.wurst.navigator_query.push(' ');
                    self.wurst.navigator_scroll = 0;
                }
                KeyCode::ArrowDown => {
                    self.wurst.navigator_scroll = self.wurst.navigator_scroll.saturating_add(1);
                }
                KeyCode::ArrowUp => {
                    self.wurst.navigator_scroll = self.wurst.navigator_scroll.saturating_sub(1);
                }
                _ => {
                    if let Some(ch) = keycode_name(key) {
                        self.wurst.navigator_query.push_str(&ch.to_ascii_lowercase());
                        self.wurst.navigator_scroll = 0;
                    }
                }
            }
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
                        self.wurst.tab_selected =
                            (self.wurst.tab_selected + cats.len() - 1) % cats.len();
                        self.wurst.tab_feature_selected = 0;
                    } else {
                        let n = Self::window_hacks(cats[self.wurst.tab_selected]).len().max(1);
                        self.wurst.tab_feature_selected =
                            (self.wurst.tab_feature_selected + n - 1) % n;
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
            self.wurst.toggle(id);
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

    pub(super) fn handle_wurst_scroll(&mut self, delta: f32) -> bool {
        if !self.wurst.active {
            return false;
        }
        if self.wurst.navigator_open {
            if delta < 0.0 {
                self.wurst.navigator_scroll = self.wurst.navigator_scroll.saturating_add(1);
            } else if delta > 0.0 {
                self.wurst.navigator_scroll = self.wurst.navigator_scroll.saturating_sub(1);
            }
            return true;
        }
        if !self.wurst.menu_open {
            return false;
        }
        let [mx, my] = self.wurst.gui_cursor;
        for idx in (0..self.wurst.gui_windows.len()).rev() {
            let win = &mut self.wurst.gui_windows[idx];
            if win.minimized || mx < win.x || mx > win.x + win.width || my < win.y + 13.0 {
                continue;
            }
            let count = match win.category {
                Some(cat) => Self::window_hacks(cat).len(),
                None => UiSettingId::ALL.len(),
            };
            let inner_h = 2.0 + (count as f32) * 13.0;
            let outer_h = inner_h.min(MAX_INNER_HEIGHT);
            if my <= win.y + 13.0 + outer_h && inner_h > MAX_INNER_HEIGHT {
                let min_scroll = -(inner_h - MAX_INNER_HEIGHT);
                win.scroll = (win.scroll + delta * 15.0).clamp(min_scroll, 0.0);
                return true;
            }
        }
        false
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
            UiSettingId::WurstLogo => "WurstLogo".to_owned(),
            UiSettingId::HackList => "HackList".to_owned(),
            UiSettingId::TabGui => "TabGui".to_owned(),
            UiSettingId::KeybindManager => "KeybindManager".to_owned(),
            UiSettingId::WurstOptions => "WurstOptions".to_owned(),
            UiSettingId::FlightHSpeed => {
                format!("Flight H: {:.2}", self.wurst.settings.flight_horizontal_speed)
            }
            UiSettingId::FlightVSpeed => {
                format!("Flight V: {:.2}", self.wurst.settings.flight_vertical_speed)
            }
            UiSettingId::FlightAntiKick => "Flight Anti-Kick".to_owned(),
            UiSettingId::AutoSprintOmni => format!(
                "OmniSprint: {}",
                if self.wurst.settings.autosprint_omni { "ON" } else { "OFF" }
            ),
            UiSettingId::AutoSprintHungry => format!(
                "HungrySprint: {}",
                if self.wurst.settings.autosprint_hungry { "ON" } else { "OFF" }
            ),
            UiSettingId::NoFallMinFall => {
                format!("Min Fall: {:.1}m", self.wurst.settings.nofall_min_fall_distance)
            }
            UiSettingId::KillauraRange => {
                format!("Range: {:.2}", self.wurst.settings.killaura_range)
            }
            UiSettingId::KillauraCps => {
                format!("Speed: {:.0} CPS", self.wurst.settings.killaura_cps)
            }
            UiSettingId::FastBreakSpeed => {
                format!("Break Speed: {:.2}x", self.wurst.settings.fastbreak_speed)
            }
            UiSettingId::ReachDistance => {
                format!("Reach: {:.2}m", self.wurst.settings.reach_distance)
            }
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
            UiSettingId::AutoSprintOmni => self.wurst.settings.autosprint_omni,
            UiSettingId::AutoSprintHungry => self.wurst.settings.autosprint_hungry,
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
            UiSettingId::KeybindManager => {
                let actions = self.wurst.run_command(".binds list");
                for a in actions {
                    if let CommandAction::Message(m) = a {
                        self.system_message(Text::literal(m).with_color(0x55_FF55));
                    }
                }
            }
            UiSettingId::WurstOptions => {
                self.wurst.menu_open = false;
                self.wurst.wurst_options_from_pause = false;
                self.wurst.wurst_options_open = true;
                self.capture(false);
            }
            UiSettingId::FlightHSpeed => {
                let v = self.wurst.settings.flight_horizontal_speed + delta * 0.25;
                self.wurst.settings.flight_horizontal_speed =
                    if v > 10.0 { 0.25 } else { v.clamp(0.05, 10.0) };
            }
            UiSettingId::FlightVSpeed => {
                let v = self.wurst.settings.flight_vertical_speed + delta * 0.25;
                self.wurst.settings.flight_vertical_speed =
                    if v > 5.0 { 0.25 } else { v.clamp(0.05, 5.0) };
            }
            UiSettingId::FlightAntiKick => {
                self.wurst.settings.flight_anti_kick = !self.wurst.settings.flight_anti_kick;
            }
            UiSettingId::AutoSprintOmni => {
                self.wurst.settings.autosprint_omni = !self.wurst.settings.autosprint_omni;
            }
            UiSettingId::AutoSprintHungry => {
                self.wurst.settings.autosprint_hungry = !self.wurst.settings.autosprint_hungry;
            }
            UiSettingId::NoFallMinFall => {
                let v = self.wurst.settings.nofall_min_fall_distance + delta * 0.5;
                self.wurst.settings.nofall_min_fall_distance =
                    if v > 10.0 { 0.0 } else { v.clamp(0.0, 10.0) };
            }
            UiSettingId::KillauraRange => {
                let v = self.wurst.settings.killaura_range + delta * 0.25;
                self.wurst.settings.killaura_range =
                    if v > 7.0 { 3.0 } else { v.clamp(3.0, 7.0) };
            }
            UiSettingId::KillauraCps => {
                let v = self.wurst.settings.killaura_cps + delta * 1.0;
                self.wurst.settings.killaura_cps =
                    if v > 20.0 { 1.0 } else { v.clamp(1.0, 20.0) };
            }
            UiSettingId::FastBreakSpeed => {
                let v = self.wurst.settings.fastbreak_speed + delta * 0.25;
                self.wurst.settings.fastbreak_speed =
                    if v > 5.0 { 1.0 } else { v.clamp(1.0, 5.0) };
            }
            UiSettingId::ReachDistance => {
                let v = self.wurst.settings.reach_distance + delta * 0.25;
                self.wurst.settings.reach_distance =
                    if v > 8.0 { 3.0 } else { v.clamp(3.0, 8.0) };
            }
            UiSettingId::SpeedMultiplier => {
                let v = self.wurst.settings.speed_multiplier + delta * 0.25;
                self.wurst.settings.speed_multiplier =
                    if v > 5.0 { 1.0 } else { v.clamp(1.0, 5.0) };
            }
            UiSettingId::HighJumpHeight => {
                let v = self.wurst.settings.highjump_height + delta * 1.0;
                self.wurst.settings.highjump_height =
                    if v > 20.0 { 1.0 } else { v.clamp(1.0, 100.0) };
            }
            UiSettingId::StepHeight => {
                let v = self.wurst.settings.step_height + delta * 0.5;
                self.wurst.settings.step_height =
                    if v > 3.0 { 0.5 } else { v.clamp(0.5, 10.0) };
            }
            UiSettingId::TimerSpeed => {
                let v = self.wurst.settings.timer_speed + delta * 0.25;
                self.wurst.settings.timer_speed =
                    if v > 5.0 { 0.5 } else { v.clamp(0.1, 10.0) };
            }
            UiSettingId::GlideFallSpeed => {
                let v = self.wurst.settings.glide_fall_speed + delta * 0.025;
                self.wurst.settings.glide_fall_speed =
                    if v > 0.25 { 0.025 } else { v.clamp(0.005, 0.25) };
            }
        }
        self.wurst.save();
    }

    pub(super) fn handle_wurst_mouse(&mut self, button: MouseButton, pressed: bool) -> bool {
        if !self.wurst.active {
            return false;
        }
        if self.wurst.wurst_options_open {
            if pressed && button == MouseButton::Left {
                self.click_wurst_options();
            }
            return true;
        }
        if self.wurst.navigator_open {
            if pressed && button == MouseButton::Left {
                self.click_navigator();
            }
            return true;
        }
        if !self.wurst.menu_open {
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
        let row_step = 13.0_f32;

        // Check SettingsWindow popup first (top-most)
        if let Some((sid, sx, sy)) = self.wurst.settings_popup {
            let sw = 140.0_f32;
            let items = Self::hack_settings(sid);
            let sh = title_h + 2.0 + (items.len().max(1) as f32) * row_step;
            if mx >= sx && mx <= sx + sw && my >= sy && my <= sy + sh {
                if my <= sy + title_h {
                    // Red X close button at [sx + sw - 11 .. sx + sw - 2]
                    if button == MouseButton::Left && mx >= sx + sw - 11.0 {
                        self.wurst.settings_popup = None;
                        self.audio.play_button_click();
                    }
                    return true;
                }
                let rel_y = my - (sy + title_h + 2.0);
                if rel_y >= 0.0 {
                    let row = (rel_y / row_step) as usize;
                    if let Some(&ui_id) = items.get(row) {
                        let delta = if button == MouseButton::Right { -1.0 } else { 1.0 };
                        self.step_ui_setting(ui_id, delta);
                        self.audio.play_button_click();
                    }
                }
                return true;
            }
        }

        for idx in (0..self.wurst.gui_windows.len()).rev() {
            let win = self.wurst.gui_windows[idx].clone();
            if mx < win.x || mx > win.x + win.width || my < win.y {
                continue;
            }
            if my <= win.y + title_h {
                if button == MouseButton::Left {
                    // Minimize box: [x2 - 11 .. x2 - 2]
                    if mx >= win.x + win.width - 11.0 {
                        self.wurst.gui_windows[idx].minimized = !win.minimized;
                    } else if mx >= win.x + win.width - 22.0 {
                        // Pin box: [x2 - 22 .. x2 - 13]
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
            let body_y = win.y + title_h;
            if let Some(cat) = win.category {
                let hacks = Self::window_hacks(cat);
                let inner_h = 2.0 + (hacks.len() as f32) * row_step;
                let outer_h = inner_h.min(MAX_INNER_HEIGHT);
                if my <= body_y + outer_h {
                    let rel_y = my - body_y - win.scroll - 2.0;
                    if rel_y >= 0.0 {
                        let row = (rel_y / row_step) as usize;
                        let in_row_y = rel_y - (row as f32) * row_step;
                        if in_row_y <= 11.0
                            && let Some(&id) = hacks.get(row)
                            && button == MouseButton::Left
                        {
                            if id.has_settings() && mx > win.x + win.width - 15.0 {
                                if self.wurst.settings_popup.is_some_and(|(cur, _, _)| cur == id) {
                                    self.wurst.settings_popup = None;
                                } else {
                                    let (sw, _) = self.gui_size();
                                    let mut px = win.x + win.width + 5.0;
                                    if px + 140.0 > sw {
                                        px = (win.x - 145.0).max(4.0);
                                    }
                                    let py = (win.y + 12.0 + (row as f32) * row_step + win.scroll).max(4.0);
                                    self.wurst.settings_popup = Some((id, px, py));
                                }
                            } else {
                                self.wurst.toggle(id);
                            }
                            self.audio.play_button_click();
                        }
                    }
                    return true;
                }
            } else {
                let inner_h = 2.0 + (UiSettingId::ALL.len() as f32) * row_step;
                let outer_h = inner_h.min(MAX_INNER_HEIGHT);
                if my <= body_y + outer_h {
                    let rel_y = my - body_y - win.scroll - 2.0;
                    if rel_y >= 0.0 {
                        let row = (rel_y / row_step) as usize;
                        let in_row_y = rel_y - (row as f32) * row_step;
                        if in_row_y <= 11.0
                            && let Some(&s) = UiSettingId::ALL.get(row)
                        {
                            let delta = if button == MouseButton::Right { -1.0 } else { 1.0 };
                            self.step_ui_setting(s, delta);
                            self.audio.play_button_click();
                        }
                    }
                    return true;
                }
            }
        }
        true
    }

    /// ClickGuiIcons.drawMinimizeArrow(x1, y1, x2, y2, hovering, minimized)
    fn draw_minimize_arrow(
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        hovering: bool,
        minimized: bool,
    ) {
        let xa1 = x1 + 1.0;
        let xa2 = (x1 + x2) * 0.5;
        let xa3 = x2 - 1.0;
        if minimized {
            let ya1 = y1 + 3.0;
            let ya2 = y2 - 2.5;
            let c = if hovering {
                rgba(0, 255, 0, 255)
            } else {
                rgba(0, 217, 0, 255)
            };
            batch.fill_triangle(atlas, [[xa1, ya1], [xa2, ya2], [xa3, ya1]], c);
        } else {
            let ya1 = y2 - 3.0;
            let ya2 = y1 + 2.5;
            let c = if hovering {
                rgba(255, 0, 0, 255)
            } else {
                rgba(217, 0, 0, 255)
            };
            batch.fill_triangle(atlas, [[xa1, ya1], [xa3, ya1], [xa2, ya2]], c);
        }
    }

    /// ClickGuiIcons.drawPin(x1, y1, x2, y2, hovering, pinned)
    fn draw_pin_icon(
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        hovering: bool,
        pinned: bool,
    ) {
        let needle = if hovering {
            rgba(255, 255, 255, 255)
        } else {
            rgba(217, 217, 217, 255)
        };
        if pinned {
            let knob = if hovering {
                rgba(255, 0, 0, 255)
            } else {
                rgba(217, 0, 0, 255)
            };
            batch.fill(atlas, x1 + 2.0, y1 + 2.0, (x2 - x1) - 4.0, (y2 - y1) - 4.0, knob);
            batch.fill(atlas, x1 + 1.0, y2 - 2.5, (x2 - x1) - 2.0, 1.5, knob);
            batch.fill(atlas, x1 + 3.5, y2 - 1.5, (x2 - x1) - 7.0, 1.5, needle);
        } else {
            let knob = if hovering {
                rgba(0, 255, 0, 255)
            } else {
                rgba(0, 217, 0, 255)
            };
            batch.fill(atlas, x1 + 2.5, y1 + 1.5, 4.5, 4.5, knob);
            batch.fill_triangle(
                atlas,
                [[x1 + 1.0, y2 - 1.0], [x1 + 4.0, y2 - 3.0], [x1 + 3.0, y2 - 4.0]],
                needle,
            );
        }
    }

    fn draw_border_2d(
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: u32,
    ) {
        let w = (x2 - x1).max(0.0);
        let h = (y2 - y1).max(0.0);
        batch.fill(atlas, x1, y1, w, 1.0, color);
        batch.fill(atlas, x1, y2 - 1.0, w, 1.0, color);
        batch.fill(atlas, x1, y1, 1.0, h, color);
        batch.fill(atlas, x2 - 1.0, y1, 1.0, h, color);
    }

    fn hack_settings(id: HackId) -> Vec<UiSettingId> {
        match id {
            HackId::Flight => vec![
                UiSettingId::FlightHSpeed,
                UiSettingId::FlightVSpeed,
                UiSettingId::FlightAntiKick,
            ],
            HackId::AutoSprint => vec![UiSettingId::AutoSprintOmni, UiSettingId::AutoSprintHungry],
            HackId::NoFall => vec![UiSettingId::NoFallMinFall],
            HackId::Killaura => vec![UiSettingId::KillauraRange, UiSettingId::KillauraCps],
            HackId::FastBreak => vec![UiSettingId::FastBreakSpeed],
            HackId::Reach => vec![UiSettingId::ReachDistance],
            HackId::SpeedHack => vec![UiSettingId::SpeedMultiplier],
            HackId::HighJump => vec![UiSettingId::HighJumpHeight],
            HackId::Step => vec![UiSettingId::StepHeight],
            HackId::Timer => vec![UiSettingId::TimerSpeed],
            HackId::Glide => vec![UiSettingId::GlideFallSpeed],
            _ => vec![],
        }
    }

    fn click_wurst_options(&mut self) {
        let (w, h) = self.gui_size();
        let mx = w * 0.5;
        let y0 = h * 0.25 - 16.0;
        let cur = self.cursor;

        // Column 1: Settings (x = mx - 154, w = 100, h = 20)
        let x1 = mx - 154.0;
        if horizon_client::ui::hovered(cur, x1, y0 + 24.0, 100.0, 20.0) {
            self.wurst.settings.click_friends = !self.wurst.settings.click_friends;
            self.wurst.save();
            self.audio.play_button_click();
            return;
        }
        if horizon_client::ui::hovered(cur, x1, y0 + 48.0, 100.0, 20.0) {
            self.wurst.settings.count_users = !self.wurst.settings.count_users;
            self.wurst.save();
            self.audio.play_button_click();
            return;
        }
        if horizon_client::ui::hovered(cur, x1, y0 + 72.0, 100.0, 20.0) {
            self.wurst.settings.spoof_vanilla = !self.wurst.settings.spoof_vanilla;
            self.wurst.save();
            self.audio.play_button_click();
            return;
        }
        if horizon_client::ui::hovered(cur, x1, y0 + 96.0, 100.0, 20.0) {
            self.wurst.settings.translations = !self.wurst.settings.translations;
            self.wurst.save();
            self.audio.play_button_click();
            return;
        }

        // Column 2: Managers (x = mx - 50, w = 100, h = 20)
        let x2 = mx - 50.0;
        if horizon_client::ui::hovered(cur, x2, y0 + 24.0, 100.0, 20.0) {
            self.audio.play_button_click();
            self.wurst.wurst_options_open = false;
            self.set_screen(Screen::Playing);
            self.wurst.menu_open = true;
            self.capture(false);
            return;
        }
        if horizon_client::ui::hovered(cur, x2, y0 + 48.0, 100.0, 20.0) {
            self.audio.play_button_click();
            self.wurst.toggle(HackId::XRay);
            return;
        }
        if horizon_client::ui::hovered(cur, x2, y0 + 72.0, 100.0, 20.0) {
            self.audio.play_button_click();
            self.wurst.settings.show_tabgui = !self.wurst.settings.show_tabgui;
            self.wurst.save();
            return;
        }

        // Column 3: Links (x = mx + 54, w = 100, h = 20)
        let x3 = mx + 54.0;
        let links = [
            (24.0, "https://www.wurstclient.net/options-website/"),
            (48.0, "https://www.wurstclient.net/options-wiki/"),
            (72.0, "https://www.wurstclient.net/options-forum/"),
            (96.0, "https://www.wurstclient.net/options-twitter/"),
            (120.0, "https://www.wurstclient.net/options-donate/"),
        ];
        for (dy, url) in links {
            if horizon_client::ui::hovered(cur, x3, y0 + dy, 100.0, 20.0) {
                self.audio.play_button_click();
                self.click_url(url.to_owned());
                return;
            }
        }

        // Bottom Back button: (mx - 100, h / 4 + 128, 200, 20)
        if horizon_client::ui::hovered(cur, mx - 100.0, h * 0.25 + 128.0, 200.0, 20.0) {
            self.audio.play_button_click();
            self.wurst.wurst_options_open = false;
            if self.wurst.wurst_options_from_pause {
                self.set_screen(Screen::Paused);
            } else {
                self.capture(true);
            }
        }
    }

    fn draw_wurst_options_screen(&self, batch: &mut HudBatch) {
        let atlas = &self.gui.atlas;
        let (w, h) = (batch.width, batch.height);
        let mx = w * 0.5;
        let y0 = h * 0.25 - 16.0;
        let cur = self.cursor;

        horizon_client::ui::menu_background(batch, atlas, 0.0, 0.0, w, h, true);

        let title = "Wurst Options";
        let tw = HudBatch::text_width(atlas, title);
        batch.text_styled(
            atlas,
            &Text::literal(title),
            mx - tw * 0.5,
            40.0,
            rgba(255, 255, 255, 255),
            true,
        );

        let headers = [
            (mx - 104.0, "Settings"),
            (mx, "Managers"),
            (mx + 104.0, "Links"),
        ];
        for (cx, label) in headers {
            let lw = HudBatch::text_width(atlas, label);
            batch.text_styled(
                atlas,
                &Text::literal(label),
                cx - lw * 0.5,
                h * 0.25 - 4.0,
                rgba(240, 240, 240, 255),
                true,
            );
        }

        let draw_btn = |batch: &mut HudBatch, x: f32, y: f32, bw: f32, label: &str| {
            let hov = horizon_client::ui::hovered(cur, x, y, bw, 20.0);
            horizon_client::ui::button(
                batch,
                atlas,
                x,
                y,
                bw,
                20.0,
                &Text::literal(label),
                true,
                hov,
            );
        };

        let x1 = mx - 154.0;
        draw_btn(
            batch,
            x1,
            y0 + 24.0,
            100.0,
            &format!(
                "Click Friends: {}",
                if self.wurst.settings.click_friends { "ON" } else { "OFF" }
            ),
        );
        draw_btn(
            batch,
            x1,
            y0 + 48.0,
            100.0,
            &format!(
                "Count Users: {}",
                if self.wurst.settings.count_users { "ON" } else { "OFF" }
            ),
        );
        draw_btn(
            batch,
            x1,
            y0 + 72.0,
            100.0,
            &format!(
                "Spoof Vanilla: {}",
                if self.wurst.settings.spoof_vanilla { "ON" } else { "OFF" }
            ),
        );
        draw_btn(
            batch,
            x1,
            y0 + 96.0,
            100.0,
            &format!(
                "Translations: {}",
                if self.wurst.settings.translations { "ON" } else { "OFF" }
            ),
        );

        let x2 = mx - 50.0;
        draw_btn(batch, x2, y0 + 24.0, 100.0, "ClickGUI / Binds");
        draw_btn(
            batch,
            x2,
            y0 + 48.0,
            100.0,
            &format!(
                "X-Ray: {}",
                if self.wurst.is_enabled(HackId::XRay) { "ON" } else { "OFF" }
            ),
        );
        draw_btn(
            batch,
            x2,
            y0 + 72.0,
            100.0,
            &format!(
                "TabGUI: {}",
                if self.wurst.settings.show_tabgui { "ON" } else { "OFF" }
            ),
        );

        let x3 = mx + 54.0;
        draw_btn(batch, x3, y0 + 24.0, 100.0, "Official Website");
        draw_btn(batch, x3, y0 + 48.0, 100.0, "Wurst Wiki");
        draw_btn(batch, x3, y0 + 72.0, 100.0, "WurstForum");
        draw_btn(batch, x3, y0 + 96.0, 100.0, "Twitter");
        draw_btn(batch, x3, y0 + 120.0, 100.0, "Donate");

        draw_btn(batch, mx - 100.0, h * 0.25 + 128.0, 200.0, "Back");
    }

    fn navigator_filtered(&self) -> Vec<HackId> {
        let q = self.wurst.navigator_query.trim().to_ascii_lowercase();
        HackId::ALL
            .iter()
            .copied()
            .filter(|id| {
                q.is_empty()
                    || id.name().to_ascii_lowercase().contains(&q)
                    || id.category().name().to_ascii_lowercase().contains(&q)
                    || id.description().to_ascii_lowercase().contains(&q)
            })
            .collect()
    }

    fn click_navigator(&mut self) {
        let (w, h) = self.gui_size();
        let cur = self.cursor;
        let mx = w * 0.5;

        if let Some(id) = self.wurst.navigator_feature {
            // Primary Enable/Disable button at (mx - 151, h - 65, 302, 18)
            if horizon_client::ui::hovered(cur, mx - 151.0, h - 65.0, 302.0, 18.0) {
                self.wurst.toggle(id);
                self.audio.play_button_click();
                return;
            }
            // Settings rows inside NavigatorFeatureScreen
            let items = Self::hack_settings(id);
            let sy = 140.0_f32;
            for (i, &ui_id) in items.iter().enumerate() {
                let ry = sy + (i as f32) * 14.0;
                if horizon_client::ui::hovered(cur, mx - 152.0, ry, 304.0, 12.0) {
                    self.step_ui_setting(ui_id, 1.0);
                    self.audio.play_button_click();
                    return;
                }
            }
            return;
        }

        let list_x = mx - 154.0;
        let items = self.navigator_filtered();
        let content_h = ((items.len() / 3) as f32) * 20.0;
        let max_scroll_rows = ((content_h - (h - 120.0)).max(0.0) / 20.0).ceil() as usize;
        let scroll_px = -((self.wurst.navigator_scroll.min(max_scroll_rows) as f32) * 20.0);

        if cur.1 < 59.0 || cur.1 > h - 42.0 {
            return;
        }
        for (i, &id) in items.iter().enumerate() {
            let fx = list_x + 104.0 * ((i % 3) as f32);
            let fy = 60.0 + ((i / 3) as f32) * 20.0 + scroll_px;
            if fy < 40.0 {
                continue;
            }
            if fy > h - 40.0 {
                break;
            }
            if horizon_client::ui::hovered(cur, fx, fy, 100.0, 16.0) {
                // Clicking the right 16x16 arrow box opens NavigatorFeatureScreen
                if cur.0 >= fx + 84.0 {
                    self.wurst.navigator_feature = Some(id);
                } else {
                    self.wurst.toggle(id);
                }
                self.audio.play_button_click();
                return;
            }
        }
    }

    fn draw_navigator_screen(&self, batch: &mut HudBatch) {
        let atlas = &self.gui.atlas;
        let (w, h) = (batch.width, batch.height);
        let cur = self.cursor;
        let mx = w * 0.5;

        let bg_color = rgba(64, 64, 64, 128);
        let bg_hover = rgba(64, 64, 64, 191);
        let outline_color = rgba(16, 16, 16, 128);
        let txt_color = rgba(240, 240, 240, 255);
        let green_on = rgba(0, 255, 0, 128);
        let green_hover = rgba(0, 255, 0, 191);

        if let Some(id) = self.wurst.navigator_feature {
            let bgx1 = mx - 154.0;
            let bgx2 = mx + 154.0;
            let bgy1 = 60.0;
            let bgy2 = h - 43.0;
            let nw = HudBatch::text_width(atlas, id.name());
            batch.text_styled(atlas, &Text::literal(id.name()), mx - nw * 0.5, 32.0, txt_color, true);

            batch.fill(atlas, bgx1, bgy1, 308.0, (bgy2 - bgy1).max(40.0), bg_color);
            Self::draw_border_2d(batch, atlas, bgx1, bgy1, bgx2, bgy2, outline_color);

            batch.text_styled(
                atlas,
                &Text::literal(format!("Type: Hack, Category: {}", id.category().name())),
                bgx1 + 4.0,
                bgy1 + 4.0,
                txt_color,
                false,
            );
            batch.text_styled(
                atlas,
                &Text::literal("Description:"),
                bgx1 + 4.0,
                bgy1 + 22.0,
                txt_color,
                false,
            );
            batch.text_styled(
                atlas,
                &Text::literal(id.description()),
                bgx1 + 4.0,
                bgy1 + 32.0,
                txt_color,
                false,
            );

            let items = Self::hack_settings(id);
            if !items.is_empty() {
                batch.text_styled(
                    atlas,
                    &Text::literal("Settings: (Click to change, Esc/Backspace = Back)"),
                    bgx1 + 4.0,
                    128.0,
                    txt_color,
                    false,
                );
                for (i, &ui_id) in items.iter().enumerate() {
                    let ry = 140.0 + (i as f32) * 14.0;
                    let hov = horizon_client::ui::hovered(cur, mx - 152.0, ry, 304.0, 12.0);
                    let on = self.ui_setting_active(ui_id);
                    let rbg = match (on, hov) {
                        (true, true) => green_hover,
                        (true, false) => green_on,
                        (false, true) => bg_hover,
                        (false, false) => bg_color,
                    };
                    batch.fill(atlas, mx - 152.0, ry, 304.0, 12.0, rbg);
                    Self::draw_border_2d(batch, atlas, mx - 152.0, ry, mx + 152.0, ry + 12.0, outline_color);
                    batch.text_styled(
                        atlas,
                        &Text::literal(self.ui_setting_label(ui_id)),
                        mx - 148.0,
                        ry + 2.0,
                        txt_color,
                        false,
                    );
                }
            } else {
                batch.text_styled(
                    atlas,
                    &Text::literal("Press Esc or Backspace to go back."),
                    bgx1 + 4.0,
                    128.0,
                    rgba(180, 180, 180, 255),
                    false,
                );
            }

            let on = self.wurst.is_enabled(id);
            let p_hov = horizon_client::ui::hovered(cur, mx - 151.0, h - 65.0, 302.0, 18.0);
            let p_bg = match (on, p_hov) {
                (true, true) => rgba(0, 255, 0, 64),
                (true, false) => rgba(0, 224, 0, 64),
                (false, true) => rgba(96, 96, 96, 64),
                (false, false) => rgba(64, 64, 64, 64),
            };
            batch.fill(atlas, mx - 151.0, h - 65.0, 302.0, 18.0, p_bg);
            Self::draw_border_2d(batch, atlas, mx - 151.0, h - 65.0, mx + 151.0, h - 47.0, outline_color);
            let plbl = if on { "Disable" } else { "Enable" };
            let plw = HudBatch::text_width(atlas, plbl);
            batch.text_styled(
                atlas,
                &Text::literal(plbl),
                mx - plw * 0.5,
                h - 60.0,
                txt_color,
                false,
            );
            return;
        }

        if let Some(id) = self.wurst.navigator_feature {
            let bgx1 = mx - 154.0;
            let bgx2 = mx + 154.0;
            let bgy1 = 60.0;
            let bgy2 = h - 43.0;
            let nw = HudBatch::text_width(atlas, id.name());
            batch.text_styled(atlas, &Text::literal(id.name()), mx - nw * 0.5, 32.0, txt_color, true);

            batch.fill(atlas, bgx1, bgy1, 308.0, (bgy2 - bgy1).max(40.0), bg_color);
            Self::draw_border_2d(batch, atlas, bgx1, bgy1, bgx2, bgy2, outline_color);

            batch.text_styled(
                atlas,
                &Text::literal(format!("Type: Hack, Category: {}", id.category().name())),
                bgx1 + 4.0,
                bgy1 + 4.0,
                txt_color,
                false,
            );
            batch.text_styled(
                atlas,
                &Text::literal("Description:"),
                bgx1 + 4.0,
                bgy1 + 22.0,
                txt_color,
                false,
            );
            batch.text_styled(
                atlas,
                &Text::literal(id.description()),
                bgx1 + 4.0,
                bgy1 + 32.0,
                txt_color,
                false,
            );

            let items = Self::hack_settings(id);
            if !items.is_empty() {
                batch.text_styled(
                    atlas,
                    &Text::literal("Settings: (Click to change, Esc/Backspace = Back)"),
                    bgx1 + 4.0,
                    128.0,
                    txt_color,
                    false,
                );
                for (i, &ui_id) in items.iter().enumerate() {
                    let ry = 140.0 + (i as f32) * 14.0;
                    let hov = horizon_client::ui::hovered(cur, mx - 152.0, ry, 304.0, 12.0);
                    let on = self.ui_setting_active(ui_id);
                    let rbg = match (on, hov) {
                        (true, true) => green_hover,
                        (true, false) => green_on,
                        (false, true) => bg_hover,
                        (false, false) => bg_color,
                    };
                    batch.fill(atlas, mx - 152.0, ry, 304.0, 12.0, rbg);
                    Self::draw_border_2d(batch, atlas, mx - 152.0, ry, mx + 152.0, ry + 12.0, outline_color);
                    batch.text_styled(
                        atlas,
                        &Text::literal(self.ui_setting_label(ui_id)),
                        mx - 148.0,
                        ry + 2.0,
                        txt_color,
                        false,
                    );
                }
            } else {
                batch.text_styled(
                    atlas,
                    &Text::literal("Press Esc or Backspace to go back."),
                    bgx1 + 4.0,
                    128.0,
                    rgba(180, 180, 180, 255),
                    false,
                );
            }

            // Primary Enable/Disable button at (mx - 151, h - 65, 302, 18)
            let on = self.wurst.is_enabled(id);
            let p_hov = horizon_client::ui::hovered(cur, mx - 151.0, h - 65.0, 302.0, 18.0);
            let p_bg = match (on, p_hov) {
                (true, true) => rgba(0, 255, 0, 64),
                (true, false) => rgba(0, 224, 0, 64),
                (false, true) => rgba(96, 96, 96, 64),
                (false, false) => rgba(64, 64, 64, 64),
            };
            batch.fill(atlas, mx - 151.0, h - 65.0, 302.0, 18.0, p_bg);
            Self::draw_border_2d(batch, atlas, mx - 151.0, h - 65.0, mx + 151.0, h - 47.0, outline_color);
            let plbl = if on { "Disable" } else { "Enable" };
            let plw = HudBatch::text_width(atlas, plbl);
            batch.text_styled(
                atlas,
                &Text::literal(plbl),
                mx - plw * 0.5,
                h - 60.0,
                txt_color,
                false,
            );
            return;
        }

        // NavigatorMainScreen.java: no dark background/blur; "Search: " at (middleX - 150, 32)
        batch.text_styled(
            atlas,
            &Text::literal("Search: "),
            mx - 150.0,
            32.0,
            txt_color,
            true,
        );
        let search_txt = if self.wurst.navigator_query.is_empty() {
            "_".to_owned()
        } else {
            format!("{}_", self.wurst.navigator_query)
        };
        batch.text_styled(
            atlas,
            &Text::literal(search_txt),
            mx - 100.0,
            32.0,
            txt_color,
            true,
        );

        let list_x = mx - 154.0;
        let items = self.navigator_filtered();
        let content_h = ((items.len() / 3) as f32) * 20.0;
        let max_scroll = (h - content_h - 120.0).min(0.0);
        let max_scroll_rows = ((-max_scroll) / 20.0).ceil() as usize;
        let scroll_px = -((self.wurst.navigator_scroll.min(max_scroll_rows) as f32) * 20.0);

        let clip_start = batch.vertices.len();
        let mut tooltip: Option<&'static str> = None;

        for (i, &id) in items.iter().enumerate() {
            let fx = list_x + 104.0 * ((i % 3) as f32);
            let fy = 60.0 + ((i / 3) as f32) * 20.0 + scroll_px;
            if fy < 40.0 {
                continue;
            }
            if fy > h - 40.0 {
                break;
            }
            let hov = cur.1 >= 59.0
                && cur.1 <= h - 42.0
                && horizon_client::ui::hovered(cur, fx, fy, 100.0, 16.0);
            if hov {
                tooltip = Some(id.description());
            }
            let on = self.wurst.is_enabled(id);
            let bg = match (on, hov) {
                (true, true) => green_hover,
                (true, false) => green_on,
                (false, true) => bg_hover,
                (false, false) => bg_color,
            };
            batch.fill(atlas, fx, fy, 100.0, 16.0, bg);
            Self::draw_border_2d(batch, atlas, fx, fy, fx + 100.0, fy + 16.0, outline_color);

            // Vertical 1px separator at x + 84 (y + 2..y + 14) + downward arrow
            let bx1 = fx + 84.0;
            batch.fill(atlas, bx1, fy + 2.0, 1.0, 12.0, outline_color);
            Self::draw_minimize_arrow(
                batch,
                atlas,
                bx1 + 2.0,
                fy + 2.5,
                fx + 98.0,
                fy + 13.0,
                hov,
                true,
            );

            let txt_clip = batch.vertices.len();
            batch.text_styled(
                atlas,
                &Text::literal(id.name()),
                fx + 4.0,
                fy + 4.0,
                txt_color,
                false,
            );
            batch.clip_since(txt_clip, fx + 2.0, fy, bx1 - 1.0, fy + 16.0);
        }
        batch.clip_since(clip_start, 0.0, 59.0, w, h - 42.0);

        // Scrollbar at (middleX + 170, 60) .. (middleX + 182, height - 43)
        if max_scroll < 0.0 {
            let sb_x = mx + 170.0;
            let sb_y = 60.0;
            let sb_h = (h - 103.0).max(28.0);
            batch.fill(atlas, sb_x, sb_y, 12.0, sb_h, bg_color);
            Self::draw_border_2d(batch, atlas, sb_x, sb_y, sb_x + 12.0, sb_y + sb_h, outline_color);

            let pct = (scroll_px / max_scroll).clamp(0.0, 1.0);
            let knob_y = sb_y + 2.0 + (sb_h - 28.0).max(0.0) * pct;
            batch.fill(atlas, sb_x + 2.0, knob_y, 8.0, 24.0, bg_color);
            Self::draw_border_2d(batch, atlas, sb_x + 2.0, knob_y, sb_x + 10.0, knob_y + 24.0, outline_color);
        }

        // Tooltip
        if let Some(tip) = tooltip {
            let tw = HudBatch::text_width(atlas, tip) + 4.0;
            let tx = if cur.0 + tw + 11.0 <= w {
                cur.0 + 8.0
            } else {
                (cur.0 - tw - 8.0).max(4.0)
            };
            let ty = if cur.1 + 12.0 <= h {
                cur.1 - 4.0
            } else {
                cur.1 - 16.0
            };
            batch.fill(atlas, tx, ty, tw, 13.0, rgba(64, 64, 64, 191));
            Self::draw_border_2d(batch, atlas, tx, ty, tx + tw, ty + 13.0, outline_color);
            batch.text_styled(atlas, &Text::literal(tip), tx + 2.0, ty + 2.0, txt_color, false);
        }
    }

    /// Draws Wurst7's `WurstLogo`, `TabGui`, `HackListHUD`, `ClickGui`, `Navigator`, and `WurstOptionsScreen`.
    pub(super) fn draw_wurst_hud(&self, batch: &mut HudBatch) {
        if !self.wurst.active {
            return;
        }
        if self.wurst.wurst_options_open {
            self.draw_wurst_options_screen(batch);
            return;
        }
        if self.wurst.navigator_open {
            self.draw_navigator_screen(batch);
            return;
        }

        let atlas = &self.gui.atlas;
        let (w, _h) = (batch.width, batch.height);

        let bg_color = rgba(64, 64, 64, 128);
        let bg_hover = rgba(64, 64, 64, 191);
        let ac_color = rgba(16, 16, 16, 128);
        let ac_hover = rgba(16, 16, 16, 191);
        let outline_color = rgba(16, 16, 16, 128);
        let txt_color = rgba(240, 240, 240, 255);
        let green_on = rgba(0, 255, 0, 128);
        let green_hover = rgba(0, 255, 0, 191);

        // 1. WurstLogo.java (exact coordinates & wurst_128.png RLE graphic)
        if self.wurst.settings.show_watermark {
            let ver = format!(
                "v{} MC{}",
                horizon_client::wurst::VERSION,
                horizon_client::wurst::MC_VERSION
            );
            let ver_w = HudBatch::text_width(atlas, &ver);
            // extractor.fill(0, 6, tr.width(version) + 76, 17, bgColor) -> y=6, h=11
            batch.fill(atlas, 0.0, 6.0, ver_w + 76.0, 11.0, rgba(255, 255, 255, 128));
            batch.text_styled(
                atlas,
                &Text::literal(ver),
                74.0,
                8.0,
                rgba(0, 0, 0, 255),
                false,
            );
            Self::draw_wurst_logo(batch, atlas, 0.0, 3.0, 1.0, 1.0);
        }

        // 2. TabGui.java: (2, 23), off by default, hidden while ClickGUI is open
        if self.wurst.settings.show_tabgui && !self.wurst.menu_open {
            let tx = 2.0_f32;
            let ty = 23.0_f32;
            let tw = 64.0_f32;
            let cats = Category::ALL;
            let th = (cats.len() as f32) * 10.0;
            batch.fill(atlas, tx, ty, tw, th, bg_color);
            Self::draw_border_2d(batch, atlas, tx, ty, tx + tw, ty + th, outline_color);
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
                    txt_color,
                    false,
                );
            }
            if self.wurst.tab_opened {
                let hacks = Self::window_hacks(cats[self.wurst.tab_selected]);
                if !hacks.is_empty() {
                    let sx = tx + tw + 2.0;
                    let sw = hacks
                        .iter()
                        .map(|h| HudBatch::text_width(atlas, h.name()) + 14.0)
                        .fold(64.0_f32, f32::max);
                    let max_rows = 15usize;
                    let total = hacks.len();
                    let start = if total > max_rows {
                        self.wurst
                            .tab_feature_selected
                            .saturating_sub(max_rows / 2)
                            .min(total - max_rows)
                    } else {
                        0
                    };
                    let visible = (total - start).min(max_rows);
                    let sh = (visible as f32) * 10.0;
                    batch.fill(atlas, sx, ty, sw, sh, bg_color);
                    Self::draw_border_2d(batch, atlas, sx, ty, sx + sw, ty + sh, outline_color);
                    for (vi, &id) in hacks[start..start + visible].iter().enumerate() {
                        let i = start + vi;
                        let prefix = if i == self.wurst.tab_feature_selected { ">" } else { "" };
                        let label = format!("{prefix}{}", id.name());
                        let color = if self.wurst.is_enabled(id) {
                            rgba(85, 255, 85, 255)
                        } else {
                            txt_color
                        };
                        batch.text_styled(
                            atlas,
                            &Text::literal(label),
                            sx + 2.0,
                            ty + 1.0 + (vi as f32) * 10.0,
                            color,
                            false,
                        );
                    }
                }
            }
        }

        // 3. HackListHUD.java: top-left below WurstLogo (hidden while ClickGUI is open to avoid overlap)
        if self.wurst.settings.show_hacklist && !self.wurst.menu_open {
            let active_hacks = self.wurst.enabled_render_names();
            let mut y = if self.wurst.settings.show_tabgui { 106.0 } else { 23.0 };
            for name in active_hacks {
                batch.text_styled(atlas, &Text::literal(name), 2.0, y, rgba(255, 255, 255, 255), true);
                y += 9.0;
            }
        }

        // 4. ClickGui.java + Window.java + ClickGuiIcons.java + FeatureButton.java
        let [mx, my] = self.wurst.gui_cursor;
        let title_h = 13.0_f32;
        let row_step = 13.0_f32;
        let mut tooltip: Option<&'static str> = None;

        for win in &self.wurst.gui_windows {
            if !self.wurst.menu_open && !win.pinned {
                continue;
            }
            let x1 = win.x;
            let y1 = win.y;
            let x2 = x1 + win.width;
            let y3 = y1 + title_h;

            // Title bar background (`toIntColor(acColor, opacity)`)
            batch.fill(atlas, x1, y1, win.width, title_h, ac_color);
            batch.text_styled(
                atlas,
                &Text::literal(win.title),
                x1 + 2.0,
                y1 + 3.0,
                txt_color,
                false,
            );

            // Pin button: 9x9 at [x2 - 22, y1 + 2]..[x2 - 13, y1 + 11]
            let px1 = x2 - 22.0;
            let px2 = x2 - 13.0;
            let py1 = y1 + 2.0;
            let py2 = y1 + 11.0;
            let h_pin = self.wurst.menu_open && mx >= px1 && mx <= px2 && my >= py1 && my <= py2;
            batch.fill(atlas, px1, py1, 9.0, 9.0, if h_pin { bg_hover } else { bg_color });
            Self::draw_pin_icon(batch, atlas, px1, py1, px2, py2, h_pin, win.pinned);
            Self::draw_border_2d(batch, atlas, px1, py1, px2, py2, outline_color);

            // Minimize button: 9x9 at [x2 - 11, y1 + 2]..[x2 - 2, y1 + 11]
            let mx1 = x2 - 11.0;
            let mx2 = x2 - 2.0;
            let h_min = self.wurst.menu_open && mx >= mx1 && mx <= mx2 && my >= py1 && my <= py2;
            batch.fill(atlas, mx1, py1, 9.0, 9.0, if h_min { bg_hover } else { bg_color });
            Self::draw_minimize_arrow(batch, atlas, mx1, py1, mx2, py2, h_min, win.minimized);
            Self::draw_border_2d(batch, atlas, mx1, py1, mx2, py2, outline_color);

            if win.minimized {
                Self::draw_border_2d(batch, atlas, x1, y1, x2, y3, outline_color);
                continue;
            }

            if let Some(cat) = win.category {
                let hacks = Self::window_hacks(cat);
                let inner_h = 2.0 + (hacks.len() as f32) * row_step;
                let scrolling = inner_h > MAX_INNER_HEIGHT;
                let outer_h = inner_h.min(MAX_INNER_HEIGHT);
                let y2 = y3 + outer_h;

                // Window body background
                batch.fill(atlas, x1, y3, win.width, outer_h, bg_color);

                if scrolling {
                    let xs1 = x2 - 3.0;
                    let thumb_y = y3 + outer_h * (-win.scroll / inner_h) + 1.0;
                    let thumb_h = ((outer_h - 2.0) * outer_h / inner_h).max(8.0);
                    batch.fill(atlas, xs1, thumb_y, 2.0, thumb_h, ac_hover);
                    Self::draw_border_2d(batch, atlas, xs1, thumb_y, xs1 + 2.0, thumb_y + thumb_h, outline_color);
                }

                let clip_start = batch.vertices.len();
                let c_width = if scrolling { win.width - 7.0 } else { win.width - 4.0 };

                for (i, &id) in hacks.iter().enumerate() {
                    let bx1 = x1 + 2.0;
                    let by1 = y3 + 2.0 + win.scroll + (i as f32) * row_step;
                    let bx2 = bx1 + c_width;
                    let by2 = by1 + 11.0;
                    if by2 < y3 || by1 > y2 {
                        continue;
                    }
                    let has_settings = id.has_settings();
                    let bx3 = if has_settings { bx2 - 11.0 } else { bx2 };

                    let in_body = my >= y3 && my <= y2;
                    let h_feat = self.wurst.menu_open
                        && in_body
                        && mx >= bx1
                        && mx <= bx3
                        && my >= by1
                        && my <= by2;
                    let h_set = self.wurst.menu_open
                        && has_settings
                        && in_body
                        && mx > bx3
                        && mx <= bx2
                        && my >= by1
                        && my <= by2;
                    let on = self.wurst.is_enabled(id);
                    if h_feat {
                        tooltip = Some(id.description());
                    }

                    let btn_bg = match (on, h_feat) {
                        (true, true) => green_hover,
                        (true, false) => green_on,
                        (false, true) => bg_hover,
                        (false, false) => bg_color,
                    };
                    batch.fill(atlas, bx1, by1, bx3 - bx1, 11.0, btn_bg);
                    if has_settings {
                        batch.fill(
                            atlas,
                            bx3,
                            by1,
                            11.0,
                            11.0,
                            if h_set { bg_hover } else { bg_color },
                        );
                        batch.fill(atlas, bx3, by1, 1.0, 11.0, outline_color);
                        Self::draw_minimize_arrow(
                            batch,
                            atlas,
                            bx3 + 1.0,
                            by1 + 0.5,
                            bx2 - 1.0,
                            by2 - 0.5,
                            h_set,
                            true,
                        );
                    }
                    Self::draw_border_2d(batch, atlas, bx1, by1, bx2, by2, outline_color);

                    let nw = HudBatch::text_width(atlas, id.name());
                    let tx = bx1 + ((bx3 - bx1) - nw) * 0.5;
                    batch.text_styled(
                        atlas,
                        &Text::literal(id.name()),
                        tx,
                        by1 + 2.0,
                        txt_color,
                        false,
                    );
                }

                batch.clip_since(clip_start, x1, y3, x2, y2);
                batch.fill(atlas, x1, y3, win.width, 1.0, outline_color);
                Self::draw_border_2d(batch, atlas, x1, y1, x2, y2, outline_color);
            } else {
                let inner_h = 2.0 + (UiSettingId::ALL.len() as f32) * row_step;
                let outer_h = inner_h.min(MAX_INNER_HEIGHT);
                let y2 = y3 + outer_h;

                batch.fill(atlas, x1, y3, win.width, outer_h, bg_color);
                let clip_start = batch.vertices.len();
                let c_width = win.width - 4.0;

                for (i, &s) in UiSettingId::ALL.iter().enumerate() {
                    let bx1 = x1 + 2.0;
                    let by1 = y3 + 2.0 + win.scroll + (i as f32) * row_step;
                    let bx2 = bx1 + c_width;
                    let by2 = by1 + 11.0;
                    let hov = self.wurst.menu_open
                        && mx >= bx1
                        && mx <= bx2
                        && my >= by1
                        && my <= by2;
                    let on = self.ui_setting_active(s);
                    let btn_bg = match (on, hov) {
                        (true, true) => green_hover,
                        (true, false) => green_on,
                        (false, true) => bg_hover,
                        (false, false) => bg_color,
                    };
                    batch.fill(atlas, bx1, by1, c_width, 11.0, btn_bg);
                    Self::draw_border_2d(batch, atlas, bx1, by1, bx2, by2, outline_color);

                    let label = self.ui_setting_label(s);
                    let nw = HudBatch::text_width(atlas, &label);
                    let tx = bx1 + (c_width - nw) * 0.5;
                    batch.text_styled(
                        atlas,
                        &Text::literal(label),
                        tx,
                        by1 + 2.0,
                        txt_color,
                        false,
                    );
                }

                batch.clip_since(clip_start, x1, y3, x2, y2);
                batch.fill(atlas, x1, y3, win.width, 1.0, outline_color);
                Self::draw_border_2d(batch, atlas, x1, y1, x2, y2, outline_color);
            }
        }

        // SettingsWindow popup (spawned by clicking the 11x11 settings arrow)
        if self.wurst.menu_open
            && let Some((sid, sx, sy)) = self.wurst.settings_popup
        {
            let sw = 140.0_f32;
            let items = Self::hack_settings(sid);
            let inner_h = 2.0 + (items.len().max(1) as f32) * row_step;
            let sy2 = sy + title_h + inner_h;
            batch.fill(atlas, sx, sy, sw, title_h, ac_color);
            batch.text_styled(
                atlas,
                &Text::literal(format!("{} Settings", sid.name())),
                sx + 2.0,
                sy + 3.0,
                txt_color,
                false,
            );
            // Red X close button
            let cx1 = sx + sw - 11.0;
            let cy1 = sy + 2.0;
            let h_close = mx >= cx1 && mx <= cx1 + 9.0 && my >= cy1 && my <= cy1 + 9.0;
            batch.fill(atlas, cx1, cy1, 9.0, 9.0, if h_close { bg_hover } else { bg_color });
            batch.text_styled(
                atlas,
                &Text::literal("x"),
                cx1 + 2.0,
                cy1 + 0.5,
                if h_close { rgba(255, 0, 0, 255) } else { rgba(217, 0, 0, 255) },
                false,
            );
            Self::draw_border_2d(batch, atlas, cx1, cy1, cx1 + 9.0, cy1 + 9.0, outline_color);

            batch.fill(atlas, sx, sy + title_h, sw, inner_h, bg_color);
            for (i, &ui_id) in items.iter().enumerate() {
                let bx1 = sx + 2.0;
                let by1 = sy + title_h + 2.0 + (i as f32) * row_step;
                let bw = sw - 4.0;
                let hov = mx >= bx1 && mx <= bx1 + bw && my >= by1 && my <= by1 + 11.0;
                let on = self.ui_setting_active(ui_id);
                let btn_bg = match (on, hov) {
                    (true, true) => green_hover,
                    (true, false) => green_on,
                    (false, true) => bg_hover,
                    (false, false) => bg_color,
                };
                batch.fill(atlas, bx1, by1, bw, 11.0, btn_bg);
                Self::draw_border_2d(batch, atlas, bx1, by1, bx1 + bw, by1 + 11.0, outline_color);
                let lbl = self.ui_setting_label(ui_id);
                let lw = HudBatch::text_width(atlas, &lbl);
                batch.text_styled(
                    atlas,
                    &Text::literal(lbl),
                    bx1 + (bw - lw) * 0.5,
                    by1 + 2.0,
                    txt_color,
                    false,
                );
            }
            batch.fill(atlas, sx, sy + title_h, sw, 1.0, outline_color);
            Self::draw_border_2d(batch, atlas, sx, sy, sx + sw, sy2, outline_color);
        }

        // Hover tooltip box (ClickGui.renderTooltip)
        if self.wurst.menu_open
            && let Some(tip) = tooltip
        {
            let tw = HudBatch::text_width(atlas, tip) + 6.0;
            let tx = (mx + 8.0).min((w - tw - 4.0).max(4.0));
            let ty = my - 14.0;
            let tt_bg = rgba(64, 64, 64, 191);
            batch.fill(atlas, tx, ty, tw, 13.0, tt_bg);
            Self::draw_border_2d(batch, atlas, tx, ty, tx + tw, ty + 13.0, outline_color);
            batch.text_styled(
                atlas,
                &Text::literal(tip),
                tx + 3.0,
                ty + 2.5,
                txt_color,
                false,
            );
        }
    }

    fn tick_wurst_combat(&mut self) {
        if self.wurst.is_enabled(HackId::Panic) {
            self.wurst.enabled.clear();
            return;
        }
        if self.wurst.is_enabled(HackId::AutoRespawn) && self.screen == super::Screen::Dead {
            let _ = self.respawn_player();
        }
        if self.wurst.is_enabled(HackId::AntiAFK) && (self.potions.effects.tick_count % 100 == 0) {
            self.yaw = (self.yaw + 0.5) % 360.0;
        }
        if self.wurst.is_enabled(HackId::AutoArmor) {
            let suffixes = ["_helmet", "_chestplate", "_leggings", "_boots"];
            for (armor_idx, suffix) in suffixes.iter().enumerate() {
                if self.armor.get(armor_idx).is_some_and(Option::is_none) {
                    if let Some(slot) = self.inventory.iter().take(9).position(|s| {
                        s.as_ref().is_some_and(|st| st.item.ends_with(suffix))
                    }) {
                        self.armor[armor_idx] = self.inventory[slot].take();
                        break;
                    }
                }
            }
        }
        if self.wurst.is_enabled(HackId::AutoTotem) {
            let has_totem = self
                .offhand
                .as_ref()
                .is_some_and(|s| s.item.contains("totem_of_undying"));
            if !has_totem {
                if let Some(slot) = self.inventory.iter().take(9).position(|s| {
                    s.as_ref()
                        .is_some_and(|st| st.item.contains("totem_of_undying"))
                }) {
                    if self.remote.is_some() {
                        self.selected = slot;
                        self.remote_swap_hands();
                    } else {
                        std::mem::swap(&mut self.offhand, &mut self.inventory[slot]);
                    }
                }
            }
        }
        if self.wurst.is_enabled(HackId::TriggerBot)
            && !self.wurst.is_enabled(HackId::Killaura)
            && let Some((idx, dist, _)) = self.mob_hit_within(self.entity_reach)
        {
            let id = self.mobs[idx].id;
            if self.wurst.is_enabled(HackId::Criticals)
                && let Some(p) = self.player.as_mut()
                && p.ground.on_ground
            {
                p.velocity.y = 0.1;
            }
            if self.remote.is_some() {
                if self.remote_attack_entity(id as i32, dist) {
                    self.remote_swing_packet();
                }
            } else {
                self.attack_mob(idx);
                self.swing_arm();
            }
        }
        if self.wurst.is_enabled(HackId::BowAimbot)
            && self
                .held()
                .is_some_and(|s| s.item.contains("bow") || s.item.contains("crossbow"))
            && let Some(player) = self.player.as_ref()
        {
            let eye = glam::DVec3::new(player.position.x, player.position.y + 1.62, player.position.z);
            let mut best: Option<(glam::DVec3, f64)> = None;
            for m in self.mobs.iter() {
                if m.removed || !m.alive() || Some(m.id) == self.remote_camera_id() {
                    continue;
                }
                let target = glam::DVec3::new(
                    m.position.x,
                    m.position.y + f64::from(m.height()) * 0.5,
                    m.position.z,
                );
                let d_sq = target.distance_squared(eye);
                if d_sq <= 48.0 * 48.0 && best.is_none_or(|(_, bd)| d_sq < bd) {
                    best = Some((target, d_sq));
                }
            }
            if let Some((target, _)) = best {
                let d = target - eye;
                let horiz = (d.x * d.x + d.z * d.z).sqrt();
                self.yaw = ((-d.x).atan2(d.z).to_degrees()) as f32;
                self.pitch = ((-d.y).atan2(horiz).to_degrees()) as f32;
            }
        }
        if self.wurst.is_enabled(HackId::Killaura)
            && let Some(player) = self.player.as_ref()
            && !player.health.dead_or_dying()
        {
            let range = self.wurst.settings.killaura_range;
            let range_sq = range * range;
            let eye = horizon_core::geometry::Vec3::new(
                player.position.x,
                player.position.y + 1.62,
                player.position.z,
            );
            let mut best: Option<(usize, u32, f64)> = None;
            for (idx, m) in self.mobs.iter().enumerate() {
                if m.removed || !m.alive() || Some(m.id) == self.remote_camera_id() {
                    continue;
                }
                let d_sq = m.position.distance_squared(eye);
                if d_sq <= range_sq && best.is_none_or(|(_, _, bd)| d_sq < bd) {
                    best = Some((idx, m.id, d_sq));
                }
            }
            if let Some((idx, id, d_sq)) = best {
                if self.wurst.is_enabled(HackId::Criticals)
                    && let Some(p) = self.player.as_mut()
                    && p.ground.on_ground
                {
                    p.velocity.y = 0.1;
                }
                let dist = d_sq.sqrt();
                if self.remote.is_some() {
                    if self.remote_attack_entity(id as i32, dist) {
                        self.remote_swing_packet();
                    }
                } else {
                    self.attack_mob(idx);
                    self.swing_arm();
                }
            }
        }
    }

    pub(crate) fn wurst_esp_gizmos(
        &self,
        alpha: f32,
        camera: [f64; 3],
        g: &mut horizon_client_lib::gizmos::Gizmos,
    ) {
        let player_esp = self.wurst.is_enabled(HackId::PlayerESP);
        let mob_esp = self.wurst.is_enabled(HackId::MobESP);
        let item_esp = self.wurst.is_enabled(HackId::ItemESP);
        let chest_esp = self.wurst.is_enabled(HackId::ChestESP);
        let base_finder = self.wurst.is_enabled(HackId::BaseFinder);
        let xray = self.wurst.is_enabled(HackId::XRay);
        if !player_esp && !mob_esp && !item_esp && !chest_esp && !base_finder && !xray {
            return;
        }
        let lerp = |a: f64, b: f64| a + f64::from(alpha) * (b - a);
        let mut push_box = |min: [f64; 3], max: [f64; 3], color: u32| {
            let edges = [
                ([min[0], min[1], min[2]], [max[0], min[1], min[2]]),
                ([max[0], min[1], min[2]], [max[0], min[1], max[2]]),
                ([max[0], min[1], max[2]], [min[0], min[1], max[2]]),
                ([min[0], min[1], max[2]], [min[0], min[1], min[2]]),
                ([min[0], max[1], min[2]], [max[0], max[1], min[2]]),
                ([max[0], max[1], min[2]], [max[0], max[1], max[2]]),
                ([max[0], max[1], max[2]], [min[0], max[1], max[2]]),
                ([min[0], max[1], max[2]], [min[0], max[1], min[2]]),
                ([min[0], min[1], min[2]], [min[0], max[1], min[2]]),
                ([max[0], min[1], min[2]], [max[0], max[1], min[2]]),
                ([max[0], min[1], max[2]], [max[0], max[1], max[2]]),
                ([min[0], min[1], max[2]], [min[0], max[1], max[2]]),
            ];
            for (a, b) in edges {
                g.on_top.push(horizon_client_lib::gizmos::Line {
                    start: a,
                    end: b,
                    color,
                    width: 2.0,
                });
            }
        };

        for m in self.mobs.iter() {
            if m.removed || !m.alive() || Some(m.id) == self.remote_camera_id() {
                continue;
            }
            let is_player = m.kind.id == "minecraft:player";
            if (is_player && !player_esp) || (!is_player && !mob_esp) {
                continue;
            }
            let x = lerp(m.previous.x, m.position.x);
            let y = lerp(m.previous.y, m.position.y);
            let z = lerp(m.previous.z, m.position.z);
            let hw = f64::from(m.width()) * 0.5;
            let h = f64::from(m.height());
            let color = if is_player { 0xFF33FF33 } else { 0xFFFF4444 };
            push_box([x - hw, y, z - hw], [x + hw, y + h, z + hw], color);
        }
        if item_esp {
            for item in &self.dropped {
                let x = lerp(item.previous.x, item.position.x);
                let y = lerp(item.previous.y, item.position.y);
                let z = lerp(item.previous.z, item.position.z);
                push_box(
                    [x - 0.15, y, z - 0.15],
                    [x + 0.15, y + 0.3, z + 0.15],
                    0xFFFFFF33,
                );
            }
        }
        if (chest_esp || base_finder || xray)
            && let Some(p) = self.player.as_ref()
        {
            let px = p.position.x.floor() as i32;
            let py = p.position.y.floor() as i32;
            let pz = p.position.z.floor() as i32;
            for bx in (px - 12)..=(px + 12) {
                for by in (py - 8)..=(py + 8) {
                    for bz in (pz - 12)..=(pz + 12) {
                        let state = self.world.block(bx, by, bz);
                        let name = &self.res.facts().get(state).name;
                        let is_chest = chest_esp
                            && (name.contains("chest")
                                || name.contains("shulker_box")
                                || name.contains("barrel"));
                        let is_base = base_finder
                            && (name.contains("beacon")
                                || name.contains("spawner")
                                || name.contains("enchanting_table")
                                || name.contains("anvil"));
                        let is_ore = xray
                            && (name.ends_with("_ore") || name.contains("ancient_debris"));
                        if is_chest || is_base || is_ore {
                            let x = f64::from(bx);
                            let y = f64::from(by);
                            let z = f64::from(bz);
                            let color = if is_chest {
                                0xFFFFA51A
                            } else if is_ore {
                                0xFF00E5FF
                            } else {
                                0xFFCC33FF
                            };
                            push_box(
                                [x + 0.05, y + 0.05, z + 0.05],
                                [x + 0.95, y + 0.95, z + 0.95],
                                color,
                            );
                        }
                    }
                }
            }
        }
    }
}
