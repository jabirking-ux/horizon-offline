//! Connects the native Rust Wurst Client (`horizon_client::wurst`) into `Game`:
//! per-tick hack execution, `.command` chat interception, keybinds, and HUD/ClickGUI rendering.
use super::*;
use horizon_client::text::Text;
use horizon_client::wurst::{CommandAction, HackId};

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

    /// Handles Wurst keybinds and the Right-Shift Quick-Menu when playing.
    /// Returns `true` if the key was consumed by the Wurst menu.
    pub(super) fn handle_wurst_key(&mut self, key: KeyCode) -> bool {
        if !self.wurst.active || self.title.is_some() || self.screen != Screen::Playing {
            return false;
        }
        if key == KeyCode::ShiftRight {
            self.wurst.menu_open = !self.wurst.menu_open;
            return true;
        }
        if self.wurst.menu_open {
            match key {
                KeyCode::Escape => {
                    self.wurst.menu_open = false;
                    return true;
                }
                KeyCode::ArrowUp => {
                    let n = HackId::ALL.len();
                    self.wurst.menu_selected = (self.wurst.menu_selected + n - 1) % n;
                    return true;
                }
                KeyCode::ArrowDown => {
                    self.wurst.menu_selected = (self.wurst.menu_selected + 1) % HackId::ALL.len();
                    return true;
                }
                KeyCode::Enter | KeyCode::Space => {
                    if let Some(&id) = HackId::ALL.get(self.wurst.menu_selected) {
                        let on = self.wurst.toggle(id);
                        self.system_message(
                            Text::literal(format!(
                                "[Wurst] {} {}",
                                id.name(),
                                if on { "ON" } else { "OFF" }
                            ))
                            .with_color(if on { 0x55_FF55 } else { 0xAA_AAAA }),
                        );
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

    /// Draws the Wurst v7.57 watermark, active HackList, and Right-Shift Quick-Menu overlay.
    pub(super) fn draw_wurst_hud(&self, batch: &mut HudBatch) {
        if !self.wurst.active {
            return;
        }
        let atlas = &self.gui.atlas;
        let (w, _h) = (batch.width, batch.height);

        // Top-left Wurst watermark (matching WurstLogo.java)
        if self.wurst.settings.show_watermark {
            let title = format!(
                "Wurst v{} MC{} (Rust)",
                horizon_client::wurst::VERSION,
                horizon_client::wurst::MC_VERSION
            );
            let box_w = (title.len() as f32) * 6.0 + 16.0;
            batch.fill(atlas, 4.0, 4.0, box_w, 16.0, 0x90_000000);
            batch.fill(atlas, 4.0, 4.0, 2.0, 16.0, 0xFF_55FF55);
            batch.text_styled(atlas, &Text::literal(title), 10.0, 8.0, 0xFF_FFFFFF, true);
        }

        // Top-right active HackList (matching HackListHUD.java)
        if self.wurst.settings.show_hacklist {
            let active_hacks = self.wurst.enabled_render_names();
            let mut y = 6.0_f32;
            for name in active_hacks {
                let text_w = (name.len() as f32) * 6.0 + 8.0;
                let x = (w - text_w - 4.0).max(4.0);
                batch.fill(atlas, x, y - 1.0, text_w, 11.0, 0x80_000000);
                batch.fill(atlas, x + text_w - 2.0, y - 1.0, 2.0, 11.0, 0xFF_55FF55);
                batch.text_styled(atlas, &Text::literal(name), x + 3.0, y, 0xFF_FFFFFF, true);
                y += 12.0;
            }
        }

        // Right-Shift interactive Quick-Menu overlay
        if self.wurst.menu_open {
            let panel_w = 240.0_f32;
            let row_h = 11.0_f32;
            let header_h = 26.0_f32;
            let panel_h = header_h + (HackId::ALL.len() as f32) * row_h + 10.0;
            let px = 8.0_f32;
            let py = 26.0_f32;

            batch.fill(atlas, px, py, panel_w, panel_h, 0xCC_101418);
            batch.fill(atlas, px, py, panel_w, 1.0, 0xFF_55FF55);
            batch.text_styled(
                atlas,
                &Text::literal("Wurst Client - Up/Down, Enter=Toggle, RShift=Close"),
                px + 6.0,
                py + 6.0,
                0xFF_55FF55,
                true,
            );
            batch.text_styled(
                atlas,
                &Text::literal("Chat: .help  .t <hack>  .set <hack> <opt> <val>"),
                px + 6.0,
                py + 16.0,
                0xFF_AAAAAA,
                true,
            );

            for (idx, &id) in HackId::ALL.iter().enumerate() {
                let ry = py + header_h + (idx as f32) * row_h;
                let on = self.wurst.is_enabled(id);
                let selected = idx == self.wurst.menu_selected;
                if selected {
                    batch.fill(atlas, px + 2.0, ry - 1.0, panel_w - 4.0, row_h, 0x90_2E4A36);
                }
                let status = if on { "[ON] " } else { "[OFF]" };
                let line = format!("{status} {} ({})", id.name(), id.category().name());
                let color = if on {
                    0xFF_55FF55
                } else if selected {
                    0xFF_FFFFFF
                } else {
                    0xFF_CCCCCC
                };
                batch.text_styled(atlas, &Text::literal(line), px + 6.0, ry, color, true);
            }
        }
    }
}
