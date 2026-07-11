#!/usr/bin/env bash
set -euo pipefail

audit="docs/RECOMMENDATION.md"

awk -F'|' '
  /^\| [0-9]+ \|/ {
    value = $2
    gsub(/[[:space:]]/, "", value)
    expected++
    if (value != expected) {
      printf "audit sequence error: expected %d, found %s\n", expected, value > "/dev/stderr"
      exit 1
    }
  }
  END {
    if (expected != 500) {
      printf "audit count error: expected 500, found %d\n", expected > "/dev/stderr"
      exit 1
    }
  }
' "$audit"

areas=$(grep -Ec '^## [0-9]+\.' "$audit")
if [[ "$areas" -ne 20 ]]; then
  echo "audit area error: expected 20, found $areas" >&2
  exit 1
fi

readme_keys=$(awk '
  /^## Configuration/ { active = 1 }
  /^## Tray menu/ { active = 0 }
  active && /^\| `[a-z_]+`/ { count++ }
  END { print count + 0 }
' README.md)
if [[ "$readme_keys" -ne 20 ]]; then
  echo "README config-key error: expected 20, found $readme_keys" >&2
  exit 1
fi

for key in \
  interval_secs jitter_secs transcription_delay translation_delay fade_duration \
  exit_duration rare_word_dwell corner card_opacity corner_radius settle_px \
  accent_color sheen theme speak recall_mode recap_chance font_scale \
  enhanced_contrast reduce_motion
do
  if ! grep -q "| \`$key\`" README.md; then
    echo "README config-key error: missing $key" >&2
    exit 1
  fi
done

grep -q 'docs/ARCHITECTURE.md' architecture.md
grep -q 'docs/RECOMMENDATION.md' recommendation.md

echo "documentation invariants: 500 audit rows, 20 areas, 20 config keys"
