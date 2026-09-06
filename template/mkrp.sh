#!/bin/bash
# ==============================================================================
# mkrp - PortMaster Official Universal Launcher Script
# Lightweight Ren'Py Runner for ARM64 Handhelds (Knulli, ROCKNIX, ArkOS)
# Version: 0.1.0
# ==============================================================================

# Default game folder name
GAME_CODE="mkrp"

SCRIPT_NAME="$(basename "${BASH_SOURCE[0]}" .sh)"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CODE_PREFIX="${SCRIPT_NAME%% - *}"

# 1. Automatic folder name detection
if [ -d "/$directory/ports/$SCRIPT_NAME" ]; then
  GAME_CODE="$SCRIPT_NAME"
elif [ -d "$SCRIPT_DIR/$SCRIPT_NAME" ]; then
  GAME_CODE="$SCRIPT_NAME"
elif [ -d "/$directory/ports/$CODE_PREFIX" ]; then
  GAME_CODE="$CODE_PREFIX"
elif [ -d "$SCRIPT_DIR/$CODE_PREFIX" ]; then
  GAME_CODE="$CODE_PREFIX"
fi

XDG_DATA_HOME=${XDG_DATA_HOME:-$HOME/.local/share}

# PortMaster header
if [ -d "/opt/system/Tools/PortMaster/" ]; then
  controlfolder="/opt/system/Tools/PortMaster"
elif [ -d "/opt/tools/PortMaster/" ]; then
  controlfolder="/opt/tools/PortMaster"
elif [ -d "$XDG_DATA_HOME/PortMaster/" ]; then
  controlfolder="$XDG_DATA_HOME/PortMaster"
else
  controlfolder="/roms/ports/PortMaster"
fi

source $controlfolder/control.txt
[ -f "${controlfolder}/mod_${CFW_NAME}.txt" ] && source "${controlfolder}/mod_${CFW_NAME}.txt"
get_controls

# Directory setup
if [ -f "$SCRIPT_DIR/mkrp" ]; then
  GAME_ROOT="$SCRIPT_DIR"
elif [ -d "/$directory/ports/$GAME_CODE" ]; then
  GAME_ROOT="/$directory/ports/$GAME_CODE"
elif [ -d "$SCRIPT_DIR/$GAME_CODE" ]; then
  GAME_ROOT="$SCRIPT_DIR/$GAME_CODE"
else
  GAME_ROOT="$(pwd)"
fi
export GAME_ROOT

cleanup() {
  echo "Cleaning up mkrp runtime environment..."
  echo "=== KERNEL DMESG (OOM / CRASH CHECK) ==="
  dmesg | tail -n 50 2>/dev/null
  $ESUDO kill -9 $(pidof gptokeyb) 2>/dev/null
  pm_finish
}
trap cleanup EXIT INT TERM

CONF_DIR="$GAME_ROOT/conf"
SAVE_DIR="$GAME_ROOT/saves"
mkdir -p "$CONF_DIR" "$SAVE_DIR"

# Enable logging
> "$GAME_ROOT/log.txt" && exec > >(tee "$GAME_ROOT/log.txt") 2>&1

echo "================================================="
echo "Starting $GAME_CODE on PortMaster ($CFW_NAME)"
echo "Directory: $GAME_ROOT"
echo "Date: $(date)"
echo "Target Platform: aarch64"
echo "================================================="

cd "$GAME_ROOT"
$ESUDO chmod +x "$GAME_ROOT/mkrp" 2>/dev/null

# Gamepad keymapping daemon (fallback / support)
if [ -f "$controlfolder/gptokeyb" ] && [ -f "$GAME_ROOT/keymap.gptk" ]; then
  $GPTOKEYB "mkrp" -c "$GAME_ROOT/keymap.gptk" &
  sleep 0.5
fi

# Set library paths
export LD_LIBRARY_PATH="$GAME_ROOT/lib:$LD_LIBRARY_PATH"

# Launch mkrp
echo "Executing mkrp..."
"$GAME_ROOT/mkrp" --game-dir "$GAME_ROOT/game" --save-dir "$SAVE_DIR"

echo "mkrp finished with exit code $?"
