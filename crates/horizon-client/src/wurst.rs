//! Native Rust port of Wurst Client (v7.57-MC26.3, GPLv3 — https://github.com/Wurst-Imperium/Wurst7)
//! for Horizon's Rust Minecraft client (`horizon-client`).
//!
//! Activated when `<game_dir>/horizon-native-mods/wurst.enabled` exists (toggled from the
//! Borizon launcher's Mods -> Horizon tab) or when `HORIZON_WURST=1` is set.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const VERSION: &str = "7.57";
pub const MC_VERSION: &str = "26.3";
pub const MARKER_REL_PATH: &str = "horizon-native-mods/wurst.enabled";
pub const CONFIG_FILE: &str = "wurst-config.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Category {
    Movement,
    Render,
    Combat,
    Player,
    Blocks,
    Chat,
    Fun,
    Other,
}

impl Category {
    pub const ALL: [Category; 8] = [
        Category::Movement,
        Category::Render,
        Category::Combat,
        Category::Player,
        Category::Blocks,
        Category::Chat,
        Category::Fun,
        Category::Other,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Category::Movement => "Movement",
            Category::Render => "Render",
            Category::Combat => "Combat",
            Category::Player => "Player",
            Category::Blocks => "Blocks",
            Category::Chat => "Chat",
            Category::Fun => "Fun",
            Category::Other => "Other",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HackId {
    AirJump,
    AntiBlind,
    AutoSprint,
    BunnyHop,
    CreativeFlight,
    Dolphin,
    FastPlace,
    Flight,
    Fullbright,
    Glide,
    HighJump,
    Jesus,
    Jetpack,
    NoClip,
    NoFall,
    NoSlowdown,
    Parkour,
    SafeWalk,
    SpeedHack,
    Spider,
    Step,
    Timer,
}

impl HackId {
    pub const ALL: [HackId; 22] = [
        HackId::AirJump,
        HackId::AntiBlind,
        HackId::AutoSprint,
        HackId::BunnyHop,
        HackId::CreativeFlight,
        HackId::Dolphin,
        HackId::FastPlace,
        HackId::Flight,
        HackId::Fullbright,
        HackId::Glide,
        HackId::HighJump,
        HackId::Jesus,
        HackId::Jetpack,
        HackId::NoClip,
        HackId::NoFall,
        HackId::NoSlowdown,
        HackId::Parkour,
        HackId::SafeWalk,
        HackId::SpeedHack,
        HackId::Spider,
        HackId::Step,
        HackId::Timer,
    ];

    pub fn name(self) -> &'static str {
        match self {
            HackId::AirJump => "AirJump",
            HackId::AntiBlind => "AntiBlind",
            HackId::AutoSprint => "AutoSprint",
            HackId::BunnyHop => "BunnyHop",
            HackId::CreativeFlight => "CreativeFlight",
            HackId::Dolphin => "Dolphin",
            HackId::FastPlace => "FastPlace",
            HackId::Flight => "Flight",
            HackId::Fullbright => "Fullbright",
            HackId::Glide => "Glide",
            HackId::HighJump => "HighJump",
            HackId::Jesus => "Jesus",
            HackId::Jetpack => "Jetpack",
            HackId::NoClip => "NoClip",
            HackId::NoFall => "NoFall",
            HackId::NoSlowdown => "NoSlowdown",
            HackId::Parkour => "Parkour",
            HackId::SafeWalk => "SafeWalk",
            HackId::SpeedHack => "SpeedHack",
            HackId::Spider => "Spider",
            HackId::Step => "Step",
            HackId::Timer => "Timer",
        }
    }

    pub fn category(self) -> Category {
        match self {
            HackId::AirJump
            | HackId::AutoSprint
            | HackId::BunnyHop
            | HackId::CreativeFlight
            | HackId::Dolphin
            | HackId::Flight
            | HackId::Glide
            | HackId::HighJump
            | HackId::Jesus
            | HackId::Jetpack
            | HackId::NoClip
            | HackId::NoFall
            | HackId::NoSlowdown
            | HackId::Parkour
            | HackId::SafeWalk
            | HackId::SpeedHack
            | HackId::Spider
            | HackId::Step => Category::Movement,
            HackId::AntiBlind | HackId::Fullbright => Category::Render,
            HackId::FastPlace => Category::Blocks,
            HackId::Timer => Category::Other,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            HackId::AirJump => "Allows you to jump in mid-air.",
            HackId::AntiBlind => "Prevents blindness and darkness from restricting movement.",
            HackId::AutoSprint => "Makes you sprint automatically.",
            HackId::BunnyHop => "Makes you jump automatically while moving.",
            HackId::CreativeFlight => "Allows you to fly like in Creative Mode.",
            HackId::Dolphin => "Makes you bob up in water automatically.",
            HackId::FastPlace => "Allows you to place blocks faster.",
            HackId::Flight => "Allows you to fly (FlightHack from Wurst7).",
            HackId::Fullbright => "Allows you to see in the dark.",
            HackId::Glide => "Makes you fall like if you had a hang glider.",
            HackId::HighJump => "Allows you to jump higher.",
            HackId::Jesus => "Allows you to walk on water.",
            HackId::Jetpack => "Allows you to fly as if you had a jetpack.",
            HackId::NoClip => "Allows you to freely move through blocks.",
            HackId::NoFall => "Protects you from fall damage.",
            HackId::NoSlowdown => "Cancels slowness effects caused by items, cobwebs, and honey.",
            HackId::Parkour => "Makes you jump automatically when reaching the edge of a block.",
            HackId::SafeWalk => "Prevents you from falling off edges.",
            HackId::SpeedHack => "Allows you to run roughly 1.5x-5x faster than normal.",
            HackId::Spider => "Allows you to climb up walls like a spider.",
            HackId::Step => "Allows you to step up full blocks.",
            HackId::Timer => "Changes the speed of almost everything.",
        }
    }

    pub fn parse(query: &str) -> Option<HackId> {
        let q = query.trim().to_ascii_lowercase().replace(['-', '_', ' '], "");
        match q.as_str() {
            "airjump" => Some(HackId::AirJump),
            "antiblind" => Some(HackId::AntiBlind),
            "autosprint" | "sprint" => Some(HackId::AutoSprint),
            "bunnyhop" | "bhop" => Some(HackId::BunnyHop),
            "creativeflight" | "cfly" => Some(HackId::CreativeFlight),
            "dolphin" => Some(HackId::Dolphin),
            "fastplace" => Some(HackId::FastPlace),
            "flight" | "fly" | "flyhack" => Some(HackId::Flight),
            "fullbright" | "fb" | "nightvision" => Some(HackId::Fullbright),
            "glide" => Some(HackId::Glide),
            "highjump" => Some(HackId::HighJump),
            "jesus" => Some(HackId::Jesus),
            "jetpack" => Some(HackId::Jetpack),
            "noclip" => Some(HackId::NoClip),
            "nofall" => Some(HackId::NoFall),
            "noslowdown" | "noslow" => Some(HackId::NoSlowdown),
            "parkour" => Some(HackId::Parkour),
            "safewalk" => Some(HackId::SafeWalk),
            "speedhack" | "speed" => Some(HackId::SpeedHack),
            "spider" => Some(HackId::Spider),
            "step" => Some(HackId::Step),
            "timer" => Some(HackId::Timer),
            _ => None,
        }
    }
}

/// Ported settings from Wurst7's `FlightHack`, `AutoSprintHack`, `NoFallHack`, `HighJumpHack`,
/// `GlideHack`, `SpeedHackHack`, `StepHack`, `TimerHack`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct WurstSettings {
    // FlightHack.java
    pub flight_horizontal_speed: f64,
    pub flight_vertical_speed: f64,
    pub flight_allow_unsafe_vertical: bool,
    pub flight_show_speed: bool,
    pub flight_anti_kick: bool,
    pub flight_anti_kick_interval: u32,
    pub flight_anti_kick_distance: f64,

    // AutoSprintHack.java
    pub autosprint_omni: bool,
    pub autosprint_hungry: bool,

    // NoFallHack.java
    pub nofall_min_fall_distance: f64,

    // HighJumpHack.java
    pub highjump_height: f64,

    // GlideHack.java
    pub glide_fall_speed: f64,
    pub glide_move_speed: f64,
    pub glide_pause_on_sneak: bool,

    // SpeedHackHack.java
    pub speed_multiplier: f64,

    // StepHack.java
    pub step_height: f64,

    // TimerHack.java
    pub timer_speed: f64,

    // HUD
    pub show_watermark: bool,
    pub show_hacklist: bool,
}

impl Default for WurstSettings {
    fn default() -> Self {
        Self {
            flight_horizontal_speed: 1.0,
            flight_vertical_speed: 1.0,
            flight_allow_unsafe_vertical: false,
            flight_show_speed: true,
            flight_anti_kick: false,
            flight_anti_kick_interval: 70,
            flight_anti_kick_distance: 0.035,

            autosprint_omni: false,
            autosprint_hungry: false,

            nofall_min_fall_distance: 1.0,

            highjump_height: 6.0,

            glide_fall_speed: 0.125,
            glide_move_speed: 1.2,
            glide_pause_on_sneak: true,

            speed_multiplier: 1.5,

            step_height: 1.0,

            timer_speed: 2.0,

            show_watermark: true,
            show_hacklist: true,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct SavedConfig {
    enabled: Vec<HackId>,
    settings: WurstSettings,
    /// Uppercase key name (e.g. "G", "C", "R") -> HackId
    binds: BTreeMap<String, HackId>,
}

/// Actions returned by `.command` execution for `Game` to apply.
#[derive(Clone, Debug)]
pub enum CommandAction {
    Message(String),
    Error(String),
    VClip(f64),
    Teleport(f64, f64, f64),
    Say(String),
    ClearChat,
    GameMode(u8),
}

pub struct WurstClient {
    pub active: bool,
    pub menu_open: bool,
    pub menu_selected: usize,
    pub enabled: HashMap<HackId, bool>,
    pub settings: WurstSettings,
    /// Uppercase key name (e.g. "G", "C", "R") -> HackId
    pub binds: BTreeMap<String, HackId>,
    /// Internal tick counter for `FlightHack.doAntiKick()`.
    pub flight_tick_counter: u32,
    /// Tracks whether `NoClip` set `player.spectator` so disabling restores it cleanly.
    pub noclip_was_spectator: Option<bool>,
    /// Tracks whether `CreativeFlight` set `player.may_fly` so disabling restores it cleanly.
    pub cfly_was_may_fly: Option<bool>,
    /// Tracks whether jump key was held last tick (for AirJump / HighJump edge detection).
    pub was_jump_held: bool,
    config_path: PathBuf,
}

impl WurstClient {
    pub fn default_binds() -> BTreeMap<String, HackId> {
        let mut binds = BTreeMap::new();
        binds.insert("G".into(), HackId::Flight);
        binds.insert("C".into(), HackId::Fullbright);
        binds.insert("R".into(), HackId::SpeedHack);
        binds.insert("J".into(), HackId::Jesus);
        binds.insert("B".into(), HackId::BunnyHop);
        binds.insert("K".into(), HackId::NoFall);
        binds.insert("H".into(), HackId::HighJump);
        binds
    }

    /// Loads Wurst state for `game_dir`. Active only when `<game_dir>/horizon-native-mods/wurst.enabled`
    /// exists or `HORIZON_WURST=1` is set in the environment.
    pub fn load(game_dir: &Path) -> Self {
        let active = game_dir.join(MARKER_REL_PATH).exists()
            || std::env::var("HORIZON_WURST").as_deref() == Ok("1");
        let config_path = game_dir.join(CONFIG_FILE);

        let mut client = Self {
            active,
            menu_open: false,
            menu_selected: 0,
            enabled: HashMap::new(),
            settings: WurstSettings::default(),
            binds: Self::default_binds(),
            flight_tick_counter: 0,
            noclip_was_spectator: None,
            cfly_was_may_fly: None,
            was_jump_held: false,
            config_path,
        };

        if active
            && let Ok(bytes) = fs::read(&client.config_path)
            && let Ok(saved) = serde_json::from_slice::<SavedConfig>(&bytes)
        {
            for id in saved.enabled {
                client.enabled.insert(id, true);
            }
            client.settings = saved.settings;
            if !saved.binds.is_empty() {
                client.binds = saved.binds;
            }
        }

        client
    }

    pub fn save(&self) {
        if !self.active {
            return;
        }
        let mut enabled: Vec<HackId> = self
            .enabled
            .iter()
            .filter_map(|(&id, &on)| on.then_some(id))
            .collect();
        enabled.sort();
        let saved = SavedConfig {
            enabled,
            settings: self.settings.clone(),
            binds: self.binds.clone(),
        };
        if let Ok(text) = serde_json::to_string_pretty(&saved) {
            let _ = fs::write(&self.config_path, text);
        }
    }

    pub fn is_enabled(&self, id: HackId) -> bool {
        self.active && self.enabled.get(&id).copied().unwrap_or(false)
    }

    pub fn set_enabled(&mut self, id: HackId, on: bool) {
        if on {
            // Mutual exclusions ported from Wurst7 (FlightHack.java, JetpackHack.java, CreativeFlightHack.java)
            match id {
                HackId::Flight => {
                    self.enabled.insert(HackId::CreativeFlight, false);
                    self.enabled.insert(HackId::Jetpack, false);
                    self.flight_tick_counter = 0;
                }
                HackId::CreativeFlight => {
                    self.enabled.insert(HackId::Flight, false);
                    self.enabled.insert(HackId::Jetpack, false);
                }
                HackId::Jetpack => {
                    self.enabled.insert(HackId::Flight, false);
                    self.enabled.insert(HackId::CreativeFlight, false);
                }
                _ => {}
            }
        }
        self.enabled.insert(id, on);
        self.save();
    }

    pub fn toggle(&mut self, id: HackId) -> bool {
        let next = !self.is_enabled(id);
        self.set_enabled(id, next);
        next
    }

    /// Ported directly from `FlightHack.getActualVerticalSpeed()`.
    pub fn flight_actual_vertical_speed(&self, invulnerable: bool) -> f64 {
        let limit = !self.settings.flight_allow_unsafe_vertical && !invulnerable;
        let max_v = if limit { 3.95 } else { 10.0 };
        (self.settings.flight_horizontal_speed * self.settings.flight_vertical_speed)
            .clamp(0.05, max_v)
    }

    /// Ported directly from `HighJumpHack.getAddedJumpPower()` (32-iteration binary search
    /// using Minecraft's per-tick gravity `-0.08` and drag `* 0.98`).
    pub fn highjump_added_power(&self) -> f64 {
        if !self.is_enabled(HackId::HighJump) {
            return 0.0;
        }
        let target_height = self.settings.highjump_height.clamp(1.0, 100.0);
        let mut lower = 0.0_f64;
        let mut upper = target_height;
        for _ in 0..32 {
            let motion = (lower + upper) * 0.5;
            let mut jump_height = 0.0;
            let mut velocity = motion;
            while velocity >= 0.003 {
                jump_height += velocity;
                velocity = (velocity - 0.08) * 0.98;
            }
            if jump_height < target_height {
                lower = motion;
            } else {
                upper = motion;
            }
        }
        ((upper as f32).next_up() as f64) - 0.42
    }

    /// Display string for HackList HUD, matching Wurst7's `Hack.getRenderName()`.
    pub fn render_name(&self, id: HackId) -> String {
        match id {
            HackId::Flight if self.settings.flight_show_speed => format!(
                "Flight [{:.2}, {:.2}]",
                self.settings.flight_horizontal_speed, self.settings.flight_vertical_speed
            ),
            HackId::HighJump => format!("HighJump [{:.0}]", self.settings.highjump_height),
            HackId::SpeedHack => format!("SpeedHack [{:.2}x]", self.settings.speed_multiplier),
            HackId::Step => format!("Step [{:.1}]", self.settings.step_height),
            HackId::Timer => format!("Timer [{:.2}x]", self.settings.timer_speed),
            _ => id.name().to_owned(),
        }
    }

    /// Returns all currently enabled hacks in alphabetical order (matching Wurst7's HackListHUD).
    pub fn enabled_render_names(&self) -> Vec<String> {
        let mut out: Vec<String> = HackId::ALL
            .iter()
            .copied()
            .filter(|&id| self.is_enabled(id))
            .map(|id| self.render_name(id))
            .collect();
        out.sort();
        out
    }

    /// Parses and runs a `.`-prefixed Wurst chat command.
    pub fn run_command(&mut self, raw: &str) -> Vec<CommandAction> {
        let body = raw.trim().strip_prefix('.').unwrap_or(raw).trim();
        if body.is_empty() {
            return vec![CommandAction::Message(
                "[Wurst] Type .help for a list of commands.".into(),
            )];
        }
        let mut parts = body.split_whitespace();
        let cmd = parts.next().unwrap_or("").to_ascii_lowercase();
        let args: Vec<&str> = parts.collect();

        match cmd.as_str() {
            "help" | "h" => vec![
                CommandAction::Message(format!("[Wurst v{VERSION}] Commands:")),
                CommandAction::Message(
                    "  .t <hack> [on|off] - Toggle a hack (or press Right Shift for menu)".into(),
                ),
                CommandAction::Message(
                    "  .hacks [category]  - List all 22 ported hacks and status".into(),
                ),
                CommandAction::Message(
                    "  .set <hack> <opt> <val> - Change a hack setting".into(),
                ),
                CommandAction::Message(
                    "  .binds [list|add <key> <hack>|remove <key>|reset]".into(),
                ),
                CommandAction::Message(
                    "  .vclip <blocks> | .tp <x> <y> <z> | .gm <0-3> | .say <msg> | .clear".into(),
                ),
            ],
            "t" | "toggle" => {
                let Some(name) = args.first() else {
                    return vec![CommandAction::Error("Usage: .t <hack> [on|off]".into())];
                };
                let Some(id) = HackId::parse(name) else {
                    return vec![CommandAction::Error(format!(
                        "[Wurst] Unknown hack '{name}'. Type .hacks to see all hacks."
                    ))];
                };
                let on = match args.get(1).map(|s| s.to_ascii_lowercase()).as_deref() {
                    Some("on" | "true" | "1") => {
                        self.set_enabled(id, true);
                        true
                    }
                    Some("off" | "false" | "0") => {
                        self.set_enabled(id, false);
                        false
                    }
                    _ => self.toggle(id),
                };
                vec![CommandAction::Message(format!(
                    "[Wurst] {} is now {}.",
                    id.name(),
                    if on { "ON" } else { "OFF" }
                ))]
            }
            "hacks" | "list" => {
                let filter = args.first().map(|s| s.to_ascii_lowercase());
                let items: Vec<String> = HackId::ALL
                    .iter()
                    .copied()
                    .filter(|id| {
                        filter
                            .as_deref()
                            .is_none_or(|f| id.category().name().to_ascii_lowercase() == f)
                    })
                    .map(|id| {
                        if self.is_enabled(id) {
                            format!("[ON] {}", id.name())
                        } else {
                            id.name().to_owned()
                        }
                    })
                    .collect();
                vec![CommandAction::Message(format!(
                    "[Wurst] Hacks ({}): {}",
                    items.len(),
                    items.join(", ")
                ))]
            }
            "set" => self.cmd_set(&args),
            "binds" | "bind" => self.cmd_binds(&args),
            "vclip" => {
                let Some(h) = args.first().and_then(|s| s.parse::<f64>().ok()) else {
                    return vec![CommandAction::Error("Usage: .vclip <height>".into())];
                };
                vec![
                    CommandAction::VClip(h),
                    CommandAction::Message(format!("[Wurst] Vertically clipped by {h} blocks.")),
                ]
            }
            "tp" => {
                if args.len() != 3 {
                    return vec![CommandAction::Error("Usage: .tp <x> <y> <z>".into())];
                }
                let (Ok(x), Ok(y), Ok(z)) = (
                    args[0].parse::<f64>(),
                    args[1].parse::<f64>(),
                    args[2].parse::<f64>(),
                ) else {
                    return vec![CommandAction::Error("Usage: .tp <x> <y> <z>".into())];
                };
                vec![
                    CommandAction::Teleport(x, y, z),
                    CommandAction::Message(format!("[Wurst] Teleported to {x:.1}, {y:.1}, {z:.1}.")),
                ]
            }
            "say" => {
                let msg = body
                    .split_once(char::is_whitespace)
                    .map(|(_, rest)| rest.trim())
                    .unwrap_or("");
                if msg.is_empty() {
                    return vec![CommandAction::Error("Usage: .say <message>".into())];
                }
                vec![CommandAction::Say(msg.to_owned())]
            }
            "clear" => vec![CommandAction::ClearChat],
            "gm" => {
                let Some(mode) = args.first().and_then(|s| match s.to_ascii_lowercase().as_str() {
                    "0" | "s" | "survival" => Some(0),
                    "1" | "c" | "creative" => Some(1),
                    "2" | "a" | "adventure" => Some(2),
                    "3" | "sp" | "spectator" => Some(3),
                    _ => None,
                }) else {
                    return vec![CommandAction::Error(
                        "Usage: .gm <0|1|2|3|survival|creative|adventure|spectator>".into(),
                    )];
                };
                vec![CommandAction::GameMode(mode)]
            }
            _ => vec![CommandAction::Error(format!(
                "[Wurst] Unknown command '.{cmd}'. Type .help for help."
            ))],
        }
    }

    fn cmd_set(&mut self, args: &[&str]) -> Vec<CommandAction> {
        if args.len() < 3 {
            return vec![CommandAction::Error(
                "Usage: .set <hack> <setting> <value> (e.g. .set flight speed 2.0)".into(),
            )];
        }
        let Some(id) = HackId::parse(args[0]) else {
            return vec![CommandAction::Error(format!(
                "[Wurst] Unknown hack '{}'.",
                args[0]
            ))];
        };
        let key = args[1].to_ascii_lowercase();
        let val = args[2];
        let parse_f = || val.parse::<f64>().ok();
        let parse_b = || match val.to_ascii_lowercase().as_str() {
            "true" | "on" | "1" => Some(true),
            "false" | "off" | "0" => Some(false),
            _ => None,
        };

        let ok = match (id, key.as_str()) {
            (HackId::Flight, "speed" | "hspeed" | "horizontal") => {
                if let Some(v) = parse_f() {
                    self.settings.flight_horizontal_speed = v.clamp(0.05, 10.0);
                    true
                } else {
                    false
                }
            }
            (HackId::Flight, "vspeed" | "vertical") => {
                if let Some(v) = parse_f() {
                    self.settings.flight_vertical_speed = v.clamp(0.05, 5.0);
                    true
                } else {
                    false
                }
            }
            (HackId::Flight, "antikick" | "anti-kick") => {
                if let Some(b) = parse_b() {
                    self.settings.flight_anti_kick = b;
                    true
                } else {
                    false
                }
            }
            (HackId::AutoSprint, "omni" | "alldirections") => {
                if let Some(b) = parse_b() {
                    self.settings.autosprint_omni = b;
                    true
                } else {
                    false
                }
            }
            (HackId::HighJump, "height") => {
                if let Some(v) = parse_f() {
                    self.settings.highjump_height = v.clamp(1.0, 100.0);
                    true
                } else {
                    false
                }
            }
            (HackId::SpeedHack, "speed" | "multiplier") => {
                if let Some(v) = parse_f() {
                    self.settings.speed_multiplier = v.clamp(1.0, 5.0);
                    true
                } else {
                    false
                }
            }
            (HackId::Glide, "fallspeed" | "speed") => {
                if let Some(v) = parse_f() {
                    self.settings.glide_fall_speed = v.clamp(0.005, 0.25);
                    true
                } else {
                    false
                }
            }
            (HackId::Step, "height") => {
                if let Some(v) = parse_f() {
                    self.settings.step_height = v.clamp(0.5, 10.0);
                    true
                } else {
                    false
                }
            }
            (HackId::Timer, "speed") => {
                if let Some(v) = parse_f() {
                    self.settings.timer_speed = v.clamp(0.1, 10.0);
                    true
                } else {
                    false
                }
            }
            (HackId::NoFall, "minfall" | "distance") => {
                if let Some(v) = parse_f() {
                    self.settings.nofall_min_fall_distance = v.clamp(0.0, 10.0);
                    true
                } else {
                    false
                }
            }
            _ => {
                return vec![CommandAction::Error(format!(
                    "[Wurst] Unknown setting '{key}' for {}.",
                    id.name()
                ))];
            }
        };

        if !ok {
            return vec![CommandAction::Error(format!(
                "[Wurst] Invalid value '{val}' for {} {key}.",
                id.name()
            ))];
        }
        self.save();
        vec![CommandAction::Message(format!(
            "[Wurst] Set {} {key} = {val}.",
            id.name()
        ))]
    }

    fn cmd_binds(&mut self, args: &[&str]) -> Vec<CommandAction> {
        match args.first().map(|s| s.to_ascii_lowercase()).as_deref() {
            None | Some("list") => {
                let list: Vec<String> = self
                    .binds
                    .iter()
                    .map(|(k, h)| format!("{k}={}", h.name()))
                    .collect();
                vec![CommandAction::Message(format!(
                    "[Wurst] Keybinds (RightShift=Menu): {}",
                    list.join(", ")
                ))]
            }
            Some("add") if args.len() >= 3 => {
                let key = args[1].to_ascii_uppercase();
                let Some(id) = HackId::parse(args[2]) else {
                    return vec![CommandAction::Error(format!(
                        "[Wurst] Unknown hack '{}'.",
                        args[2]
                    ))];
                };
                self.binds.insert(key.clone(), id);
                self.save();
                vec![CommandAction::Message(format!(
                    "[Wurst] Bound {key} -> {}.",
                    id.name()
                ))]
            }
            Some("remove" | "del") if args.len() >= 2 => {
                let key = args[1].to_ascii_uppercase();
                self.binds.remove(&key);
                self.save();
                vec![CommandAction::Message(format!("[Wurst] Unbound {key}."))]
            }
            Some("reset") => {
                self.binds = Self::default_binds();
                self.save();
                vec![CommandAction::Message(
                    "[Wurst] Keybinds reset to defaults.".into(),
                )]
            }
            _ => vec![CommandAction::Error(
                "Usage: .binds [list | add <key> <hack> | remove <key> | reset]".into(),
            )],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highjump_binary_search_matches_wurst7() {
        let mut w = WurstClient::load(Path::new("."));
        w.active = true;
        w.set_enabled(HackId::HighJump, true);
        w.settings.highjump_height = 6.0;
        let added = w.highjump_added_power();
        // Vanilla jump is 0.42 (~1.25 blocks); 6 blocks requires ~0.95 total -> ~0.53 added.
        assert!(added > 0.4 && added < 0.8, "added={added}");
    }

    #[test]
    fn flight_mutual_exclusion_matches_wurst7() {
        let mut w = WurstClient::load(Path::new("."));
        w.active = true;
        w.set_enabled(HackId::CreativeFlight, true);
        assert!(w.is_enabled(HackId::CreativeFlight));
        w.set_enabled(HackId::Flight, true);
        assert!(w.is_enabled(HackId::Flight));
        assert!(!w.is_enabled(HackId::CreativeFlight));
    }

    #[test]
    fn toggle_command_works() {
        let mut w = WurstClient::load(Path::new("."));
        w.active = true;
        let _ = w.run_command(".t flight on");
        assert!(w.is_enabled(HackId::Flight));
        let _ = w.run_command(".t flight off");
        assert!(!w.is_enabled(HackId::Flight));
    }
}
