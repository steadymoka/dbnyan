#!/usr/bin/env bash
# <xbar.title>dbnyan</xbar.title>
# <xbar.desc>portless server status + start/stop</xbar.desc>
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
esac

# portless's daemon listens for *.localhost regardless of upstream — so
# checking root would always succeed (502 page). Probe /api/health and
# require 2xx so we only see 🐈 when the Rust server itself is alive.
HEALTH="$(curl -sk --max-time 2 -o /dev/null -w '%{http_code}' "$URL/api/health" 2>/dev/null || echo 000)"
if [[ "$HEALTH" =~ ^2 ]]; then
  STATUS="up"
elif pgrep -f "portless $NAME" >/dev/null; then
  STATUS="starting"
else
  STATUS="down"
fi

case "$STATUS" in
  up)       echo "🐈" ;;
  starting) echo "⌯" ;;
  *)        echo "⌯" ;;
esac

echo "---"
echo "Open $URL | href=$URL"
echo "---"

if [[ "$STATUS" == "down" ]]; then
  echo "▶︎ Start server | shell=$SELF param1=start terminal=false refresh=true"
else
  echo "■ Stop server | shell=$SELF param1=stop terminal=false refresh=true"
fi

echo "View logs | shell=/usr/bin/open param1=$LOG terminal=false"
echo "Refresh | refresh=true"
