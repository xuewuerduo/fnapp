#!/bin/bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUTPUT="$SCRIPT_DIR/oui.csv"
URL="https://standards-oui.ieee.org/oui/oui.csv"

echo "下载 IEEE OUI 列表..."
curl -fL "$URL" -o /tmp/oui_raw.csv

echo "解析中..."
python3 -c "
import csv, sys
with open('/tmp/oui_raw.csv') as f:
    r = csv.reader(f)
    next(r)  # skip header
    for row in r:
        if len(row) >= 3 and len(row[1]) == 6:
            prefix = row[1]
            vendor = row[2].strip().strip('\"').strip()
            if vendor:
                # quote if contains comma
                if ',' in vendor:
                    print(f'{prefix},\"{vendor}\"')
                else:
                    print(f'{prefix},{vendor}')
" > "$OUTPUT"

TOTAL=$(wc -l < "$OUTPUT")
echo "OK! $TOTAL records -> $OUTPUT"
