#!/bin/bash
# ==============================================================================
# mkrp - PortMaster Official Universal Ren'Py Launcher Script
# Lightweight Ren'Py Runner for ARM64 Handhelds (Knulli, ROCKNIX, ArkOS)
# Version: 0.1.1.0
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
CACHE_DIR="$CONF_DIR/.cache"
mkdir -p "$CONF_DIR" "$SAVE_DIR" "$CACHE_DIR"

# Enable logging
> "$GAME_ROOT/log.txt" && exec > >(tee "$GAME_ROOT/log.txt") 2>&1

echo "================================================="
echo "Starting $GAME_CODE on PortMaster ($CFW_NAME)"
echo "Directory: $GAME_ROOT"
echo "Date: $(date)"
echo "Target Platform: aarch64"
echo "================================================="

# 2. MicroSD Sequential Read-Ahead Optimization (with auto-restore on exit)
MMC_DEV=""
ORIG_READ_AHEAD=""
if [ -d "/sys/block" ]; then
  MNT_DEV=$(df -P "$GAME_ROOT" 2>/dev/null | awk 'NR==2 {print $1}')
  BASE_DEV=$(basename "$MNT_DEV" 2>/dev/null | sed 's/p[0-9]*$//')
  if [ -n "$BASE_DEV" ] && [ -f "/sys/block/$BASE_DEV/queue/read_ahead_kb" ]; then
    MMC_DEV="$BASE_DEV"
    ORIG_READ_AHEAD=$(cat "/sys/block/$MMC_DEV/queue/read_ahead_kb" 2>/dev/null)
    echo "Optimizing MicroSD read-ahead: /sys/block/$MMC_DEV/queue/read_ahead_kb ($ORIG_READ_AHEAD -> 1024)"
    echo 1024 > "/sys/block/$MMC_DEV/queue/read_ahead_kb" 2>/dev/null
  fi
fi

# 3. Ren'Py Runtime Mounting (PortMaster official renpy_8.3.4 / 8.1.3)
RENPY_RUNTIME="renpy_8.3.4"
if [ ! -f "$controlfolder/libs/${RENPY_RUNTIME}.squashfs" ] && [ -f "$controlfolder/libs/renpy_8.1.3.squashfs" ]; then
  RENPY_RUNTIME="renpy_8.1.3"
fi

RENPY_DIR="/tmp/renpy"

cleanup() {
  echo "Cleaning up Ren'Py runtime environment..."
  $ESUDO kill -9 $(pidof gptokeyb) 2>/dev/null
  $ESUDO kill -9 $(pidof gptokeyb2) 2>/dev/null

  # Restore original MicroSD read-ahead
  if [ -n "$MMC_DEV" ] && [ -n "$ORIG_READ_AHEAD" ]; then
    echo "Restoring MicroSD read-ahead for $MMC_DEV to $ORIG_READ_AHEAD KB..."
    echo "$ORIG_READ_AHEAD" > "/sys/block/$MMC_DEV/queue/read_ahead_kb" 2>/dev/null
  fi

  if [ -d "$RENPY_DIR" ]; then
    echo "Unmounting $RENPY_DIR..."
    if command -v fuser >/dev/null 2>&1; then
      $ESUDO fuser -k -9 "$RENPY_DIR" 2>/dev/null
    fi
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

# 4. Gamepad keymapping daemon (gptokeyb)
KEYMAP_FILE="$GAME_ROOT/keymap.gptk"
if [ ! -f "$KEYMAP_FILE" ] && [ -f "$SCRIPT_DIR/keymap.gptk" ]; then
  KEYMAP_FILE="$SCRIPT_DIR/keymap.gptk"
fi

if [ -f "$controlfolder/gptokeyb" ] && [ -f "$KEYMAP_FILE" ]; then
  echo "Starting gptokeyb for startRENPY with keymap: $KEYMAP_FILE"
  $GPTOKEYB "startRENPY" -c "$KEYMAP_FILE" &
  sleep 0.5
fi

# 5. Exports and Environment
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

# GPU Shader Disk Cache: Prevent on-the-fly shader compile stutters on transitions
export MESA_SHADER_CACHE_DIR="$CACHE_DIR"
export MESA_GLSL_CACHE_DIR="$CACHE_DIR"
export __GL_SHADER_DISK_CACHE_PATH="$CACHE_DIR"

# Mali GPU scheduling & Wayland RT thread priority
export MALI_SCHED_RT_THREAD_PRIORITY=95

# 6. CPU Big.LITTLE Core Affinity Tuning
# Prefer high-performance big cores on 8-core SoCs (e.g. RK3576, RK3588, RK3399)
CPU_AFFINITY_CMD=""
if [ -n "$FAST_CORES" ]; then
  CPU_AFFINITY_CMD="$FAST_CORES"
elif command -v taskset >/dev/null 2>&1; then
  NUM_CPUS=$(nproc 2>/dev/null || grep -c ^processor /proc/cpuinfo 2>/dev/null || echo 4)
  if [ "$NUM_CPUS" -ge 8 ]; then
    CPU_AFFINITY_CMD="taskset -c 4-7"
  fi
fi

if [ -n "$CPU_AFFINITY_CMD" ]; then
  echo "Enabling CPU Big Core Affinity: $CPU_AFFINITY_CMD"
fi

cd "$GAME_ROOT"

# 7. Launch Ren'Py
echo "Executing Ren'Py runtime: $RENPY_DIR/startRENPY \"$GAME_ROOT\"..."
if [ -n "$CPU_AFFINITY_CMD" ]; then
  $CPU_AFFINITY_CMD "$RENPY_DIR/startRENPY" "$GAME_ROOT"
else
  "$RENPY_DIR/startRENPY" "$GAME_ROOT"
fi
EXIT_CODE=$?

echo "Ren'Py finished with exit code $EXIT_CODE"
exit $EXIT_CODE
