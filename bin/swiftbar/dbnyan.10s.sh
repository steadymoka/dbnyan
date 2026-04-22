#!/usr/bin/env bash
# <xbar.title>dbnyan</xbar.title>
# <xbar.desc>portless server status + start/stop, plus dev server</xbar.desc>
# <swiftbar.hideAbout>true</swiftbar.hideAbout>
# <swiftbar.hideRunInTerminal>true</swiftbar.hideRunInTerminal>
# <swiftbar.hideDisablePlugin>true</swiftbar.hideDisablePlugin>

# Lives at <project>/bin/swiftbar/dbnyan.10s.sh — project root is two levels
# up. `readlink -f` survives the case where someone symlinks this elsewhere.
SELF="$(readlink -f "$0" 2>/dev/null || echo "$0")"
PROJECT="$(cd "$(dirname "$SELF")/../.." && pwd)"

# SwiftBar runs with a minimal PATH. Prepend common dev tool locations so
# `portless`, `node`, `cargo`, `bun` are findable.
export PATH="/opt/homebrew/bin:/usr/local/bin:$HOME/.cargo/bin:$HOME/.bun/bin:$PATH"
for nvmdir in "$HOME"/.nvm/versions/node/*/bin; do
  [ -d "$nvmdir" ] && export PATH="$nvmdir:$PATH"
done

NAME="${PORTLESS_NAME:-dbnyan}"
URL="https://${NAME}.localhost:1355"
LOG="$HOME/Library/Logs/dbnyan-portless.log"

DEV_NAME="${PORTLESS_DEV_NAME:-dbnyan-dev}"
DEV_URL="https://${DEV_NAME}.localhost:1355"
DEV_LOG="$HOME/Library/Logs/dbnyan-dev.log"
DEV_PIDFILE="$HOME/Library/Caches/dbnyan-dev.pid"

case "${1:-}" in
  start)
    mkdir -p "$(dirname "$LOG")"
    cd "$PROJECT" || exit 1
    nohup portless "$NAME" ./bin/start >> "$LOG" 2>&1 &
    disown
    exit 0
    ;;
  stop)
    pkill -f "portless $NAME" 2>/dev/null
    pkill -f "target/release/dbnyan-server" 2>/dev/null
    exit 0
    ;;
  dev-start)
    mkdir -p "$(dirname "$DEV_LOG")" "$(dirname "$DEV_PIDFILE")"
    cd "$PROJECT" || exit 1
    # Truncate log per start so a fresh run isn't buried under old output.
    : > "$DEV_LOG"
    nohup portless "$DEV_NAME" ./bin/dev >> "$DEV_LOG" 2>&1 &
    echo $! > "$DEV_PIDFILE"
    disown
    exit 0
    ;;
  dev-stop)
    pkill -f "portless $DEV_NAME" 2>/dev/null
    pkill -f "target/debug/dbnyan-server" 2>/dev/null
    pkill -f "node_modules/.bin/vite" 2>/dev/null
    if [ -f "$DEV_PIDFILE" ]; then
      kill "$(cat "$DEV_PIDFILE")" 2>/dev/null || true
      rm -f "$DEV_PIDFILE"
    fi
    exit 0
    ;;
esac

# --- prod status ---
# portless's daemon listens for *.localhost regardless of upstream — so
# checking root would always succeed (502 page). Probe /api/health and
# require 2xx so we only see 🐈 when the Rust server itself is alive.
HEALTH="$(curl -sk --max-time 2 -o /dev/null -w '%{http_code}' "$URL/api/health" 2>/dev/null || echo 000)"
if [[ "$HEALTH" =~ ^2 ]]; then
  PROD_STATUS="up"
elif pgrep -f "portless $NAME" >/dev/null; then
  PROD_STATUS="starting"
else
  PROD_STATUS="down"
fi

# --- dev status ---
# Probe /api/health via vite's proxy — proves both the Rust backend AND vite
# are up, not just portless's 502 fallback.
DEV_HEALTH="$(curl -sk --max-time 2 -o /dev/null -w '%{http_code}' "$DEV_URL/api/health" 2>/dev/null || echo 000)"
if [[ "$DEV_HEALTH" =~ ^2 ]]; then
  DEV_STATUS="up"
elif pgrep -f "portless $DEV_NAME" >/dev/null \
  || pgrep -f "target/debug/dbnyan-server" >/dev/null \
  || pgrep -f "node_modules/.bin/vite" >/dev/null; then
  DEV_STATUS="starting"
else
  DEV_STATUS="down"
fi

# --- menu bar icon ---
if [[ "$PROD_STATUS" == "up" ]]; then
  echo "🐈"
elif [[ "$DEV_STATUS" == "up" ]]; then
  echo "🔧"
elif [[ "$PROD_STATUS" == "starting" || "$DEV_STATUS" == "starting" ]]; then
  echo "⌯"
else
  echo "⌯"
fi

echo "---"

# --- production section ---
echo "Production · $PROD_STATUS | disabled=true size=11"
echo "Open $URL | href=$URL"
if [[ "$PROD_STATUS" == "down" ]]; then
  echo "▶︎ Start | shell=$SELF param1=start terminal=false refresh=true"
else
  echo "■ Stop | shell=$SELF param1=stop terminal=false refresh=true"
fi
echo "View logs | shell=/usr/bin/open param1=$LOG terminal=false"

echo "---"

# --- development section ---
echo "Development · $DEV_STATUS | disabled=true size=11"
echo "Open $DEV_URL | href=$DEV_URL"
if [[ "$DEV_STATUS" == "down" ]]; then
  if [[ "$PROD_STATUS" != "down" ]]; then
    # Prod holds port 3939 → cargo dev build would ENOADDR. Disable to warn.
    echo "▶︎ Start (cargo + vite) | disabled=true"
    echo "  (stop production first — both use port 3939) | size=10 color=gray"
  else
    echo "▶︎ Start (cargo + vite) | shell=$SELF param1=dev-start terminal=false refresh=true"
  fi
else
  echo "■ Stop | shell=$SELF param1=dev-stop terminal=false refresh=true"
fi
echo "View logs | shell=/usr/bin/open param1=$DEV_LOG terminal=false"

echo "---"
echo "Refresh | refresh=true"
