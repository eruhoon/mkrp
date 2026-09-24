#!/bin/bash
# ==============================================================================
# mkrp - PortMaster Official Universal Ren'Py Launcher Script
# Lightweight Ren'Py Runner for ARM64 Handhelds (Knulli, ROCKNIX, ArkOS)
# Version: 0.1.0.0
# ==============================================================================

SCRIPT_NAME="$(basename "${BASH_SOURCE[0]}" .sh)"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CODE_PREFIX="${SCRIPT_NAME%% - *}"

# 1. Automatic folder name detection
GAME_CODE="mkrp"
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
if [ -d "/$directory/ports/$GAME_CODE" ]; then
  GAME_ROOT="/$directory/ports/$GAME_CODE"
elif [ -d "$SCRIPT_DIR/$GAME_CODE" ]; then
  GAME_ROOT="$SCRIPT_DIR/$GAME_CODE"
else
  GAME_ROOT="$(pwd)"
fi
export GAME_ROOT

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

# 2. Ren'Py Runtime Mounting (PortMaster official renpy_8.3.4 / 8.1.3)
RENPY_RUNTIME="renpy_8.3.4"
if [ ! -f "$controlfolder/libs/${RENPY_RUNTIME}.squashfs" ] && [ -f "$controlfolder/libs/renpy_8.1.3.squashfs" ]; then
  RENPY_RUNTIME="renpy_8.1.3"
fi

RENPY_DIR="/tmp/renpy"

cleanup() {
  echo "Cleaning up Ren'Py runtime environment..."
  $ESUDO kill -9 $(pidof gptokeyb) 2>/dev/null
  $ESUDO kill -9 $(pidof gptokeyb2) 2>/dev/null
  if [ -d "$RENPY_DIR" ]; then
    echo "Unmounting $RENPY_DIR..."
    $ESUDO umount "$RENPY_DIR" 2>/dev/null
    rmdir "$RENPY_DIR" 2>/dev/null
  fi
  pm_finish
}
trap cleanup EXIT INT TERM

if [ -f "$controlfolder/libs/${RENPY_RUNTIME}.squashfs" ]; then
  echo "Mounting runtime: $controlfolder/libs/${RENPY_RUNTIME}.squashfs -> $RENPY_DIR"
  $ESUDO mkdir -p "$RENPY_DIR"
  $ESUDO umount "$RENPY_DIR" 2>/dev/null
  $ESUDO mount "$controlfolder/libs/${RENPY_RUNTIME}.squashfs" "$RENPY_DIR"
else
  echo "ERROR: Ren'Py runtime not found in $controlfolder/libs/ (${RENPY_RUNTIME}.squashfs)"
  exit 1
fi

if [ ! -f "$RENPY_DIR/startRENPY" ]; then
  echo "ERROR: startRENPY executable not found in $RENPY_DIR"
  exit 1
fi

$ESUDO chmod +x "$RENPY_DIR/startRENPY" 2>/dev/null

# 3. Gamepad keymapping daemon (gptokeyb)
KEYMAP_FILE="$GAME_ROOT/keymap.gptk"
if [ ! -f "$KEYMAP_FILE" ] && [ -f "$SCRIPT_DIR/keymap.gptk" ]; then
  KEYMAP_FILE="$SCRIPT_DIR/keymap.gptk"
fi

if [ -f "$controlfolder/gptokeyb" ] && [ -f "$KEYMAP_FILE" ]; then
  echo "Starting gptokeyb for startRENPY with keymap: $KEYMAP_FILE"
  $GPTOKEYB "startRENPY" -c "$KEYMAP_FILE" &
  sleep 0.5
fi

# 4. Exports and Environment
export PORTMASTER_HOME="$controlfolder"
export SDL_GAMECONTROLLERCONFIG="$sdl_controllerconfig"
export LD_LIBRARY_PATH="$RENPY_DIR:$RENPY_DIR/lib:$LD_LIBRARY_PATH"
export PYTHONPATH="$RENPY_DIR:$RENPY_DIR/lib/python3.12:$PYTHONPATH"

# Ren'Py optimization & redirection envs
export RENPY_PATH_TO_SAVES="$SAVE_DIR"
export RENPY_NO_REDIRECT_STDIO=1
# Smooth rendering and VSync sync on ARM64 Mali GPUs
export RENPY_GL_VSYNC=1
export RENPY_GL_SWAP_INTERVAL=1

cd "$GAME_ROOT"

# 5. Launch Ren'Py
echo "Executing Ren'Py runtime: $RENPY_DIR/startRENPY \"$GAME_ROOT\"..."
"$RENPY_DIR/startRENPY" "$GAME_ROOT"
EXIT_CODE=$?

echo "Ren'Py finished with exit code $EXIT_CODE"
exit $EXIT_CODE
