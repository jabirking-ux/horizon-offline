//! Native Rust port of Wurst Client (v7.57-MC26.3, GPLv3 — https://github.com/Wurst-Imperium/Wurst7)
//! for Horizon's Rust Minecraft client (`horizon-client`).

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

include!("wurst_logo_data.rs");

pub const VERSION: &str = "7.57";
pub const MC_VERSION: &str = "26.3";
pub const MARKER_REL_PATH: &str = "horizon-native-mods/wurst.enabled";
pub const CONFIG_FILE: &str = "wurst-config.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Category {
    Blocks,
    Movement,
    Combat,
    Render,
    Chat,
    Fun,
    Items,
    Other,
}

impl Category {
    pub const ALL: [Category; 8] = [
        Category::Blocks,
        Category::Movement,
        Category::Combat,
        Category::Render,
        Category::Chat,
        Category::Fun,
        Category::Items,
        Category::Other,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Category::Blocks => "Blocks",
            Category::Movement => "Movement",
            Category::Combat => "Combat",
            Category::Render => "Render",
            Category::Chat => "Chat",
            Category::Fun => "Fun",
            Category::Items => "Items",
            Category::Other => "Other",
        }
    }
}

macro_rules! define_hacks {
    ($(($variant:ident, $name:literal, $cat:ident, $desc:literal)),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub enum HackId {
            $($variant,)*
        }

        impl HackId {
            pub const ALL: [HackId; 155] = [
                $(HackId::$variant,)*
            ];

            pub fn name(self) -> &'static str {
                match self {
                    $(HackId::$variant => $name,)*
                }
            }

            pub fn category(self) -> Category {
                match self {
                    $(HackId::$variant => Category::$cat,)*
                }
            }

            pub fn description(self) -> &'static str {
                match self {
                    $(HackId::$variant => $desc,)*
                }
            }
        }
    };
}

define_hacks!(
    // 1. BLOCKS (26)
    (AirPlace, "AirPlace", Blocks, "Allows you to place blocks in mid-air."),
    (AntiCactus, "AntiCactus", Blocks, "Protects you from cactus damage."),
    (AutoBuild, "AutoBuild", Blocks, "Automatically builds the selected template."),
    (AutoFarm, "AutoFarm", Blocks, "Harvests and re-plants crops automatically."),
    (AutoMine, "AutoMine", Blocks, "Automatically mines the block you look at."),
    (AutoSign, "AutoSign", Blocks, "Instantly writes pre-set text on signs."),
    (AutoTool, "AutoTool", Blocks, "Automatically equips the fastest applicable tool."),
    (BonemealAura, "BonemealAura", Blocks, "Automatically uses bone meal on nearby crops."),
    (BuildRandom, "BuildRandom", Blocks, "Randomly places blocks around you."),
    (Excavator, "Excavator", Blocks, "Automatically breaks all blocks in the selected area."),
    (FastBreak, "FastBreak", Blocks, "Allows you to break blocks faster."),
    (FastPlace, "FastPlace", Blocks, "Allows you to place blocks 5 times faster."),
    (HandNoClip, "HandNoClip", Blocks, "Allows you to reach specific blocks through walls."),
    (InstaBuild, "InstaBuild", Blocks, "Builds small structures instantly."),
    (InstantBunker, "InstantBunker", Blocks, "Builds a 5x5x3 bunker around you."),
    (Kaboom, "Kaboom", Blocks, "Breaks blocks around you like an explosion."),
    (Liquids, "Liquids", Blocks, "Allows you to interact with liquid blocks."),
    (Nuker, "Nuker", Blocks, "Automatically breaks blocks around you."),
    (NukerLegit, "NukerLegit", Blocks, "Slower Nuker that bypasses all cheat prevention plugins."),
    (ScaffoldWalk, "ScaffoldWalk", Blocks, "Automatically places blocks below your feet."),
    (SpeedNuker, "SpeedNuker", Blocks, "Faster version of Nuker that cannot bypass NoCheat+."),
    (TemplateTool, "TemplateTool", Blocks, "Allows you to create custom templates for AutoBuild."),
    (Tillaura, "Tillaura", Blocks, "Automatically turns dirt, grass, etc. into farmland."),
    (TreeBot, "TreeBot", Blocks, "Experimental bot that chops down trees around you."),
    (Tunneller, "Tunneller", Blocks, "Automatically digs a tunnel around you."),
    (VeinMiner, "VeinMiner", Blocks, "Automatically mines an entire vein of ore."),

    // 2. MOVEMENT (34 incl. AirJump)
    (AirJump, "AirJump", Movement, "Allows you to jump in mid-air."),
    (AntiEntityPush, "AntiEntityPush", Movement, "Prevents you from getting pushed by other entities."),
    (AntiHunger, "AntiHunger", Movement, "Slows down your hunger when you are walking."),
    (AntiWaterPush, "AntiWaterPush", Movement, "Prevents you from getting pushed by water."),
    (AutoSprint, "AutoSprint", Movement, "Makes you sprint automatically."),
    (AutoSwim, "AutoSwim", Movement, "Triggers the swimming animation automatically."),
    (AutoWalk, "AutoWalk", Movement, "Makes you walk automatically."),
    (Blink, "Blink", Movement, "Suspends all motion updates while enabled."),
    (BoatFly, "BoatFly", Movement, "Allows you to fly with boats and other rideable entities."),
    (BunnyHop, "BunnyHop", Movement, "Makes you jump automatically while moving."),
    (CreativeFlight, "CreativeFlight", Movement, "Allows you to fly like in Creative Mode."),
    (Dolphin, "Dolphin", Movement, "Makes you bob up in water automatically."),
    (ExtraElytra, "ExtraElytra", Movement, "Makes the Elytra easier to use."),
    (FastLadder, "FastLadder", Movement, "Allows you to climb up ladders faster."),
    (Fish, "Fish", Movement, "Disables underwater gravity so you can swim like a fish."),
    (Flight, "Flight", Movement, "Allows you to fly."),
    (Follow, "Follow", Movement, "A bot that follows the closest entity."),
    (Glide, "Glide", Movement, "Makes you fall like if you had a hang glider."),
    (HighJump, "HighJump", Movement, "Allows you to jump higher."),
    (InvWalk, "InvWalk", Movement, "Allows you to walk while the inventory is open."),
    (Jesus, "Jesus", Movement, "Allows you to walk on water."),
    (Jetpack, "Jetpack", Movement, "Allows you to fly as if you had a jetpack."),
    (NoClip, "NoClip", Movement, "Allows you to freely move through blocks."),
    (NoFall, "NoFall", Movement, "Protects you from fall damage."),
    (NoLevitation, "NoLevitation", Movement, "Removes the levitation effect when you get hit by a shulker."),
    (NoSlowdown, "NoSlowdown", Movement, "Cancels slowness effects caused by items, cobwebs, and honey."),
    (NoWeb, "NoWeb", Movement, "Prevents you from getting slowed down by cobwebs."),
    (Parkour, "Parkour", Movement, "Makes you jump automatically when reaching the edge of a block."),
    (SafeWalk, "SafeWalk", Movement, "Prevents you from falling off edges."),
    (Sneak, "Sneak", Movement, "Makes you sneak automatically."),
    (SnowShoe, "SnowShoe", Movement, "Allows you to walk on powder snow."),
    (SpeedHack, "SpeedHack", Movement, "Allows you to run roughly 1.5x-5x faster than normal."),
    (Spider, "Spider", Movement, "Allows you to climb up walls like a spider."),
    (Step, "Step", Movement, "Allows you to step up full blocks."),

    // 3. COMBAT (24)
    (AimAssist, "AimAssist", Combat, "Helps you aim at nearby entities."),
    (AnchorAura, "AnchorAura", Combat, "Automatically places, charges, and detonates respawn anchors."),
    (AntiKnockback, "AntiKnockback", Combat, "Prevents you from getting pushed by players and mobs."),
    (ArrowDMG, "ArrowDMG", Combat, "Massively increases arrow damage, but also consumes a lot of hunger."),
    (AttributeSwap, "AttributeSwap", Combat, "Swaps to a mace or heavy weapon right as you attack."),
    (AutoArmor, "AutoArmor", Combat, "Manages your armor automatically."),
    (AutoLeave, "AutoLeave", Combat, "Automatically leaves the server when your health is low."),
    (AutoPotion, "AutoPotion", Combat, "Automatically throws splash healing potions when your health is low."),
    (AutoRespawn, "AutoRespawn", Combat, "Automatically respawns you whenever you die."),
    (AutoSoup, "AutoSoup", Combat, "Automatically eats soup when your health is low."),
    (AutoSword, "AutoSword", Combat, "Automatically equips the best weapon in your hotbar."),
    (AutoTotem, "AutoTotem", Combat, "Automatically moves Totems of Undying to your off-hand."),
    (BowAimbot, "BowAimbot", Combat, "Automatically aims your bow or crossbow."),
    (ClickAura, "ClickAura", Combat, "Automatically attacks the closest valid entity whenever you click."),
    (Criticals, "Criticals", Combat, "Changes all your hits to critical hits."),
    (CrystalAura, "CrystalAura", Combat, "Automatically places and detonates End Crystals."),
    (FightBot, "FightBot", Combat, "A bot that automatically walks around and kills everything."),
    (Killaura, "Killaura", Combat, "Automatically attacks entities around you."),
    (KillauraLegit, "KillauraLegit", Combat, "Slower Killaura that is harder to detect."),
    (MaceDMG, "MaceDMG", Combat, "Increases the damage of mace smash attacks."),
    (MultiAura, "MultiAura", Combat, "Faster Killaura that attacks multiple entities at once."),
    (Protect, "Protect", Combat, "A bot that follows the closest entity and protects it."),
    (TPAura, "TP-Aura", Combat, "Automatically attacks the closest valid entity while teleporting around it."),
    (TriggerBot, "TriggerBot", Combat, "Automatically attacks the entity you are looking at."),

    // 4. RENDER (37)
    (AntiBlind, "AntiBlind", Render, "Prevents blindness and darkness."),
    (AntiWobble, "AntiWobble", Render, "Disables the wobble effect caused by nausea and portals."),
    (BarrierESP, "BarrierESP", Render, "Allows you to see barrier blocks."),
    (BaseFinder, "BaseFinder", Render, "Finds player bases by searching for man-made blocks."),
    (CameraDistance, "CameraDistance", Render, "Allows you to change the camera distance in 3rd person."),
    (CameraNoClip, "CameraNoClip", Render, "Allows the camera in 3rd person to go through walls."),
    (CaveFinder, "CaveFinder", Render, "Helps you to find caves by highlighting them."),
    (ChestESP, "ChestESP", Render, "Highlights nearby chests and storage blocks."),
    (Freecam, "Freecam", Render, "Allows you to move the camera without moving your character."),
    (Fullbright, "Fullbright", Render, "Allows you to see in the dark."),
    (HealthTags, "HealthTags", Render, "Shows the health of players in their nametags."),
    (ItemESP, "ItemESP", Render, "Highlights nearby items."),
    (MobESP, "MobESP", Render, "Highlights nearby mobs."),
    (MobSpawnESP, "MobSpawnESP", Render, "Highlights areas where mobs can spawn."),
    (NameProtect, "NameProtect", Render, "Hides all player names."),
    (NameTags, "NameTags", Render, "Changes the scale of the nametags so you can always read them."),
    (NewChunks, "NewChunks", Render, "Highlights newly generated chunks around you."),
    (NoBackground, "NoBackground", Render, "Removes the dark background behind inventories."),
    (NoFireOverlay, "NoFireOverlay", Render, "Blocks the fire overlay when you are burning."),
    (NoFog, "NoFog", Render, "Removes all fog effects."),
    (NoHurtcam, "NoHurtcam", Render, "Disables the shaking effect when you get hurt."),
    (NoOverlay, "NoOverlay", Render, "Blocks the overlays of water, lava, and powdered snow."),
    (NoPumpkin, "NoPumpkin", Render, "Blocks the overlay when wearing a carved pumpkin."),
    (NoShieldOverlay, "NoShieldOverlay", Render, "Lowers the shield overlay so it blocks less of your view."),
    (NoVignette, "NoVignette", Render, "Blocks the vignette overlay."),
    (NoWeather, "NoWeather", Render, "Allows you to alter the client-side weather and time."),
    (OpenWaterESP, "OpenWaterESP", Render, "Highlights open water for fishing."),
    (Overlay, "Overlay", Render, "Renders the Nuker animation when you mine a block."),
    (PlayerESP, "PlayerESP", Render, "Highlights nearby players."),
    (PortalESP, "PortalESP", Render, "Highlights nearby portals."),
    (ProphuntESP, "ProphuntESP", Render, "Allows you to see fake blocks in Prophunt."),
    (Radar, "Radar", Render, "Shows the location of nearby entities."),
    (RemoteView, "RemoteView", Render, "Allows you to see the world as someone else."),
    (Search, "Search", Render, "Helps you to find specific blocks by highlighting them."),
    (Trajectories, "Trajectories", Render, "Predicts the flight path of arrows and throwable items."),
    (TrueSight, "TrueSight", Render, "Allows you to see invisible entities."),
    (XRay, "X-Ray", Render, "Allows you to see ores through walls."),

    // 5. CHAT (7)
    (AntiSpam, "AntiSpam", Chat, "Blocks chat spam by replacing repeated messages with a counter."),
    (AutoComplete, "AutoComplete", Chat, "Uses an LLM to complete your chat messages."),
    (ChatTranslator, "ChatTranslator", Chat, "Translates chat messages using Google Translate."),
    (FancyChat, "FancyChat", Chat, "Replaces ASCII characters in sent chat messages with fancier unicode."),
    (ForceOP, "ForceOP", Chat, "Cracks AuthMe passwords to gain OP."),
    (InfiniChat, "InfiniChat", Chat, "Removes the 100-line limit from the chat history."),
    (MassTPA, "MassTPA", Chat, "Sends a TPA request to all players."),

    // 6. FUN (7)
    (Derp, "Derp", Fun, "Randomly moves your head around."),
    (HeadRoll, "HeadRoll", Fun, "Makes you nod all the time."),
    (LSD, "LSD", Fun, "Causes colorful hallucinations."),
    (MileyCyrus, "MileyCyrus", Fun, "Makes you twerk."),
    (RainbowUI, "RainbowUI", Fun, "Makes the ClickGUI backdrop and borders have rainbow colors."),
    (SkinDerp, "SkinDerp", Fun, "Randomly toggles parts of your skin."),
    (Tired, "Tired", Fun, "Makes you look tired."),

    // 7. ITEMS (9)
    (AutoDrop, "AutoDrop", Items, "Automatically drops unwanted items."),
    (AutoEat, "AutoEat", Items, "Automatically eats food when necessary."),
    (AutoSteal, "AutoSteal", Items, "Automatically steals everything from all chests you open."),
    (AutoSwitch, "AutoSwitch", Items, "Switches the item in your hand all the time."),
    (CrashChest, "CrashChest", Items, "Generates a chest that bans people from the server if they copy it."),
    (ItemGenerator, "ItemGenerator", Items, "Generates random items and drops them on the ground (Creative only)."),
    (KillPotion, "KillPotion", Items, "Generates a potion that can kill almost anything (Creative only)."),
    (Restock, "Restock", Items, "Automatically restocks your hotbar from your inventory."),
    (TrollPotion, "TrollPotion", Items, "Generates a potion with many annoying effects on it."),

    // 8. OTHER (11)
    (AntiAFK, "AntiAFK", Other, "Walks around randomly to hide you from AFK detectors."),
    (AutoFish, "AutoFish", Other, "Automatically catches fish using your best fishing rod."),
    (AutoLibrarian, "AutoLibrarian", Other, "Automatically trains villagers to sell specific enchanted books."),
    (AutoReconnect, "AutoReconnect", Other, "Automatically reconnects when you get kicked from the server."),
    (FeedAura, "FeedAura", Other, "Automatically feeds nearby animals."),
    (Panic, "Panic", Other, "Instantly turns off all enabled hacks."),
    (PortalGUI, "PortalGUI", Other, "Allows you to open GUIs in portals."),
    (Reach, "Reach", Other, "Allows you to reach further."),
    (Throw, "Throw", Other, "Uses an item multiple times rapidly."),
    (Timer, "Timer", Other, "Changes the speed of almost everything."),
    (TooManyHax, "TooManyHax", Other, "Blocks features that you don't want to accidentally enable."),
);

impl HackId {
    pub fn has_settings(self) -> bool {
        matches!(
            self,
            HackId::Flight
                | HackId::AutoSprint
                | HackId::NoFall
                | HackId::HighJump
                | HackId::Glide
                | HackId::SpeedHack
                | HackId::Step
                | HackId::Timer
                | HackId::Killaura
                | HackId::FastBreak
                | HackId::Reach
        )
    }

    pub fn parse(query: &str) -> Option<HackId> {
        let q = query.trim().to_ascii_lowercase().replace(['-', '_', ' '], "");
        for &id in &Self::ALL {
            let clean = id.name().to_ascii_lowercase().replace(['-', '_', ' '], "");
            if clean == q {
                return Some(id);
            }
        }
        match q.as_str() {
            "fly" | "flyhack" => Some(HackId::Flight),
            "sprint" => Some(HackId::AutoSprint),
            "bhop" => Some(HackId::BunnyHop),
            "cfly" => Some(HackId::CreativeFlight),
            "fb" | "nightvision" => Some(HackId::Fullbright),
            "noslow" => Some(HackId::NoSlowdown),
            "speed" => Some(HackId::SpeedHack),
            "ka" | "aura" => Some(HackId::Killaura),
            "xray" => Some(HackId::XRay),
            "scaffold" => Some(HackId::ScaffoldWalk),
            _ => None,
        }
    }
}

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

    // HUD & WurstOptionsScreen
    pub show_watermark: bool,
    pub show_hacklist: bool,
    pub show_tabgui: bool,
    pub click_friends: bool,
    pub count_users: bool,
    pub spoof_vanilla: bool,
    pub translations: bool,
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
            // TabGuiOtf.java defaults to Status.DISABLED
            show_tabgui: false,
            click_friends: true,
            count_users: true,
            spoof_vanilla: false,
            translations: true,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct SavedConfig {
    enabled: Vec<HackId>,
    settings: WurstSettings,
    binds: BTreeMap<String, HackId>,
}

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

#[derive(Clone, Debug)]
pub struct GuiWindow {
    pub title: &'static str,
    pub category: Option<Category>,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub minimized: bool,
    pub pinned: bool,
    pub scroll: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiSettingId {
    WurstLogo,
    HackList,
    TabGui,
    KeybindManager,
    WurstOptions,
    FlightHSpeed,
    FlightVSpeed,
    FlightAntiKick,
    SpeedMultiplier,
    HighJumpHeight,
    StepHeight,
    TimerSpeed,
    GlideFallSpeed,
}

impl UiSettingId {
    pub const ALL: [UiSettingId; 13] = [
        UiSettingId::WurstLogo,
        UiSettingId::HackList,
        UiSettingId::TabGui,
        UiSettingId::KeybindManager,
        UiSettingId::WurstOptions,
        UiSettingId::FlightHSpeed,
        UiSettingId::FlightVSpeed,
        UiSettingId::FlightAntiKick,
        UiSettingId::SpeedMultiplier,
        UiSettingId::HighJumpHeight,
        UiSettingId::StepHeight,
        UiSettingId::TimerSpeed,
        UiSettingId::GlideFallSpeed,
    ];
}

pub struct WurstClient {
    pub active: bool,
    pub menu_open: bool,
    pub wurst_options_open: bool,
    pub menu_selected: usize,
    pub tab_selected: usize,
    pub tab_opened: bool,
    pub tab_feature_selected: usize,
    pub gui_windows: Vec<GuiWindow>,
    pub gui_dragging: Option<(usize, f32, f32)>,
    pub settings_popup: Option<(HackId, f32, f32)>,
    pub gui_cursor: [f32; 2],
    pub enabled: HashMap<HackId, bool>,
    pub settings: WurstSettings,
    pub binds: BTreeMap<String, HackId>,
    pub flight_tick_counter: u32,
    pub noclip_was_spectator: Option<bool>,
    pub cfly_was_may_fly: Option<bool>,
    pub was_jump_held: bool,
    config_path: PathBuf,
}

impl WurstClient {
    pub fn default_binds() -> BTreeMap<String, HackId> {
        let mut binds = BTreeMap::new();
        binds.insert("G".into(), HackId::Flight);
        binds.insert("C".into(), HackId::Fullbright);
        binds.insert("R".into(), HackId::Killaura);
        binds.insert("X".into(), HackId::XRay);
        binds.insert("J".into(), HackId::Jesus);
        binds.insert("B".into(), HackId::FastPlace);
        binds.insert("K".into(), HackId::MultiAura);
        binds.insert("H".into(), HackId::HighJump);
        binds.insert("U".into(), HackId::Freecam);
        binds.insert("N".into(), HackId::Nuker);
        binds.insert("Z".into(), HackId::Sneak);
        binds
    }

    pub fn load(game_dir: &Path) -> Self {
        let active = game_dir.join(MARKER_REL_PATH).exists()
            || std::env::current_dir().is_ok_and(|c| c.join(MARKER_REL_PATH).exists())
            || std::env::var("HORIZON_WURST").as_deref() == Ok("1");
        let config_path = game_dir.join(CONFIG_FILE);

        // ClickGui.init() layout: starts at (5, 5), x += width + 5, wraps at scaledWidth
        let mut client = Self {
            active,
            menu_open: false,
            wurst_options_open: false,
            menu_selected: 0,
            tab_selected: 0,
            tab_opened: false,
            tab_feature_selected: 0,
            gui_windows: vec![
                GuiWindow { title: "Blocks", category: Some(Category::Blocks), x: 5.0, y: 23.0, width: 98.0, minimized: false, pinned: false, scroll: 0.0 },
                GuiWindow { title: "Movement", category: Some(Category::Movement), x: 108.0, y: 23.0, width: 102.0, minimized: false, pinned: false, scroll: 0.0 },
                GuiWindow { title: "Combat", category: Some(Category::Combat), x: 215.0, y: 23.0, width: 98.0, minimized: false, pinned: false, scroll: 0.0 },
                GuiWindow { title: "Render", category: Some(Category::Render), x: 318.0, y: 23.0, width: 104.0, minimized: false, pinned: false, scroll: 0.0 },
                GuiWindow { title: "Chat", category: Some(Category::Chat), x: 427.0, y: 23.0, width: 96.0, minimized: true, pinned: false, scroll: 0.0 },
                GuiWindow { title: "Fun", category: Some(Category::Fun), x: 528.0, y: 23.0, width: 88.0, minimized: true, pinned: false, scroll: 0.0 },
                GuiWindow { title: "Items", category: Some(Category::Items), x: 427.0, y: 41.0, width: 96.0, minimized: true, pinned: false, scroll: 0.0 },
                GuiWindow { title: "Other", category: Some(Category::Other), x: 528.0, y: 41.0, width: 96.0, minimized: false, pinned: false, scroll: 0.0 },
                GuiWindow { title: "UI Settings", category: None, x: 427.0, y: 59.0, width: 112.0, minimized: false, pinned: false, scroll: 0.0 },
            ],
            gui_dragging: None,
            settings_popup: None,
            gui_cursor: [0.0, 0.0],
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
        if on && id == HackId::Panic {
            self.enabled.clear();
            self.save();
            return;
        }
        if on {
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

    pub fn flight_actual_vertical_speed(&self, invulnerable: bool) -> f64 {
        let limit = !self.settings.flight_allow_unsafe_vertical && !invulnerable;
        let max_v = if limit { 3.95 } else { 10.0 };
        (self.settings.flight_horizontal_speed * self.settings.flight_vertical_speed)
            .clamp(0.05, max_v)
    }

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
                    "  .t <hack> [on|off] - Toggle any of the 155 Wurst7 hacks".into(),
                ),
                CommandAction::Message(
                    "  .hacks [category]  - List hacks in a category".into(),
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
                if name.eq_ignore_ascii_case("tabgui") {
                    self.settings.show_tabgui = !self.settings.show_tabgui;
                    self.save();
                    return vec![CommandAction::Message(format!(
                        "[Wurst] TabGUI is now {}.",
                        if self.settings.show_tabgui { "ON" } else { "OFF" }
                    ))];
                }
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
                    "[Wurst] Keybinds (RightShift=ClickGUI): {}",
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
