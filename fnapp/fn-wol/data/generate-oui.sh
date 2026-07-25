#!/bin/bash
# OUI 厂商数据库生成脚本（Linux / macOS）
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUTPUT="$SCRIPT_DIR/oui.csv"
URL="https://standards-oui.ieee.org/oui/oui.csv"

echo "正在下载 IEEE OUI 列表..."
curl -fL "$URL" -o /tmp/oui_raw.csv

echo "正在解析..."
awk -F',' '
  NR > 1 && length($2) == 6 {
    gsub(/^"|"$/, "", $3)
    print $2 "," $3
  }
' /tmp/oui_raw.csv > "$OUTPUT"

TOTAL=$(wc -l < "$OUTPUT")
echo "完成！共 $TOTAL 条记录，输出到 $OUTPUT"
