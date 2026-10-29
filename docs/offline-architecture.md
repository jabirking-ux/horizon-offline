# Horizon Offline: Architecture & Offline Support

This fork decouples Horizon from Microsoft account infrastructure, Xbox Live authentication, and OS keychain persistence layers (`oo7`, `keyring`).

## Key Principles

1. **Zero Account Requirement**:
   - The launcher does not require Microsoft authentication or device-code authorization to prepare or run Minecraft: Java Edition 26.3.
   - Offline player profiles (`account offline [name]`) are first-class and launch without an upstream "owner gate".

2. **Direct Mojang Asset Pipeline**:
   - Downloads required game files (client jar, server jar, libraries, asset indexes, audio, fonts) directly from Mojang's official endpoints (`piston-meta.mojang.com`, `piston-data.mojang.com`, `resources.download.minecraft.net`).
   - Every file is verified against Mojang's published SHA-1 hashes and pinned `research-lock.json` entries.
   - Resumable: interrupted transfers verify and resume without re-downloading existing valid artifacts.

3. **Lean Dependency Graph**:
   - Removed optional platform keychain dependencies (`oo7`, `futures-lite`, `keyring`).
   - In-memory session and secret storage with zero OS keyring daemon overhead.

## Standard Usage Flow

```bash
# Configure game directory
horizon-launcher settings minecraft-dir ~/.minecraft

# Switch to offline player identity
horizon-launcher account offline Steve

# Prepare 26.3 assets directly from Mojang endpoints
horizon-launcher prepare --minecraft ~/.minecraft

# Launch the game fully offline
horizon-launcher play
```
