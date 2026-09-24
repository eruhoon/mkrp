# mkrp (Make Ren'Py Portable) 🎮

`mkrp` is a universal launcher template and handheld optimization toolkit designed to run **Ren'Py visual novel games** smoothly and reliably on Linux ARM64-based retro handheld consoles (e.g., RG VITA PRO, RG40XXH, RG DS running ROCKNIX, Knulli, ArkOS) within the **PortMaster** ecosystem.

It comes pre-packaged with a universal drop-in patch (`zz_handheld_patch.rpy`) that resolves Python 2 to 3 migration issues, adapts memory usage to device specs, and eliminates MicroSD card save/load freezes out of the box.

---

## 🌟 Key Features

### 1. ⚙️ Dynamic Hardware RAM Scaling
- **Automatic Memory Detection**: Reads `/proc/meminfo` at startup to automatically scale resource allocation based on physical RAM:
  - **1GB Devices** (e.g., RG35XX H, RG40XX H): 160MB cache, 16 prediction statements, 96 font glyph cache to prevent Out-Of-Memory (OOM) crashes.
  - **2GB Devices** (e.g., RK3566 2GB, RG405M): 256MB cache, 32 prediction statements, 160 font glyph cache.
  - **4GB+ Devices** (e.g., RG VITA Pro, RG Cube, RG556, Odin): 512MB cache, 48 prediction statements, 256 font glyph cache for smooth CJK text rendering.
- **Rollback Memory Management**: Restricts rollback history buffer (`rollback_length = 20`, `hard_rollback_limit = 40`) to prevent progressive memory bloat during long reading sessions.

### 2. ⚡ MicroSD Storage & Save / Load Acceleration
- **Sequential Read-Ahead Optimization**: Scales kernel block read-ahead queue from default 128KB to 1024KB (1MB) on startup for 30~50% faster sequential asset streaming, and automatically restores original settings upon game exit.
- **In-Memory Slot Metadata Caching**: Hooks into `renpy.loadsave.slot_json` and `slot_mtime` with granular slot-level eviction to eliminate repeated random I/O storms across slow MicroSD cards.
- **Auto-Fit Thumbnail Screenshots**: Wraps screenshot displayables to seamlessly fit native UI frame dimensions with quality optimization.
- **Instant Menu Transitions**: Disables sluggish full-screen FBO Dissolve transitions on confirmation dialogs and menus (`enter_yesno_transition = None`), replacing them with crisp, snappy transitions (0.15s).

### 3. 🛡️ Ren'Py 8 & Python 3 Compatibility Layer (Compat Polyfills)
- **Builtin `cmp` Polyfill**: Restores Python 2 comparison logic for older visual novels (Ren'Py 7).
- **Class `None` Comparison Defense**: Automatically patches custom classes to safely handle `< None` and `> None` comparisons, preventing fatal `TypeError` crashes on legacy stat and affection checks.

### 4. 🎮 Handheld Controls & PortMaster Standards
- **PortMaster Runtime Mounting**: Integrates with official PortMaster SquashFS runtimes (`renpy_8.3.4` / `renpy_8.1.3`).
- **Pre-Configured Gamepad Mapping**: Built-in `gptokeyb` profile providing virtual mouse cursor control via analog stick and standard handheld layout mappings (A: Advance/Click, B: Rollback/Right-Click, Y: Menu, X: Hide UI, Triggers: Auto/Skip/QuickSave).

---

## 📁 Project Structure

### 1. Source Repository

```text
mkrp/
├── template/
│   ├── mkrp.sh            # PortMaster device launcher script (CPU affinity, shader cache & launch)
│   ├── port.json          # PortMaster metadata and runtime configuration
│   ├── keymap.gptk        # gptokeyb gamepad/virtual mouse mapping
│   └── game/
│       └── zz_handheld_patch.rpy  # ⭐️ Pure universal performance & compatibility patch
├── patches/               # 🔒 Local game-specific overlay patches (.gitignored, never committed)
├── scripts/
│   ├── build.mjs          # PortMaster zip packaging script (supports --patch <name>)
│   ├── clean.mjs          # Clean dist build directory
│   └── optimize-assets.mjs# Asset downscaling and optimization tool (sharp)
├── HOW_TO_USE.md          # Hardware installation and device setup guide
├── package.json           # Node.js project manifest
└── README.md              # Project documentation
```

### 2. Distribution Package Structure (PortMaster `ports/`)

When you run `pnpm run build`, `dist/mkrp-v*.zip` is generated. Extracting it produces the standard PortMaster layout:

```text
roms/ports/ (or storage/roms/ports/)
├── mkrp.sh                # Game launcher script (placed directly in ports/)
└── mkrp/                  # Main game directory
    ├── port.json          # Port metadata
    ├── keymap.gptk        # Gamepad control mapping
    ├── conf/              # Persistent configs and Mesa shader disk cache (.cache)
    ├── game/
    │   ├── zz_handheld_patch.rpy  # ⭐️ Pre-bundled universal handheld patch (auto-loaded)
    │   └── [User Game Assets]    # Place your game's *.rpa, *.rpyc, and audio here
    └── saves/             # Save data directory
```

---

## 🚀 Quick Start

### 1. Build the PortMaster Package

```bash
pnpm install

# 1) Build pure universal PortMaster package:
pnpm run build

# 2) (Optional) Build with a local game-specific patch (from patches/<game_name>/):
pnpm run build --patch <game_name>
```

### 2. (Optional) Optimize PC High-Resolution Game Assets

If porting a 1080p/1440p PC game to a 720p/480p handheld device, downscale the assets to save massive amounts of RAM and eliminate SD card loading stutters:

```bash
pnpm run optimize --input /path/to/extracted/images --max-width 1280 --max-height 720
```

### 3. Install on Device

1. Extract the `.zip` archive into the `ports/` directory on your handheld console's SD card.
2. Copy the contents of your Ren'Py game's `game/` folder (such as `*.rpa`, `*.rpyc`, fonts, images) into `mkrp/game/`.
   > [!NOTE]
   > Do **not** overwrite or delete `mkrp/game/zz_handheld_patch.rpy`—it automatically applies the hardware scaling, save speedups, and Python 3 polyfills.
3. Launch the game from the **Ports** section in your handheld's frontend (EmulationStation).

> [!TIP]
> For control button layouts, troubleshooting, and asset conversion tips, refer to [HOW_TO_USE.md](HOW_TO_USE.md).

---

## 📄 License

MIT License
