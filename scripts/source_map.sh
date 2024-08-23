#!/bin/bash

set -euo pipefail
cd $TRUNK_STAGING_DIR

wasm_name=$(printf *.wasm)

cargo wasm2map $wasm_name
wasm-tools strip $wasm_name -o $wasm_name

cat > $wasm_name.map.js <<EOF
window.__wasm_source_map__ = '$(cat $wasm_name.map)'
EOF

source_map_script="<script defer src=\"/$wasm_name.map.js\"></script>"

sed -i -e "s%<script type=\"module\"%$source_map_script<script type=\"module\"%g" index.html
