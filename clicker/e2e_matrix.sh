#!/usr/bin/env bash
# e2e transport/mdns matrix: 4 cells x 2 suites, sequential.
# Usage: e2e_matrix.sh [rooms_rejoin|mainnet_local|all]  (default: all)
# Each cell exports CLICKER_TRANSPORT (+CLICKER_MDNS for lan-final),
# runs nextest into a fresh .local-run dir, records the outcome, and
# appends one markdown table. A Telegram text summary closes the run.

set -u
cd "$(dirname "$0")"

STAMP=$(date +%Y%m%d-%H%M%S)
REPORT=".local-run/matrix-${STAMP}.md"
SUITES="${1:-all}"

{
    echo "# e2e matrix ${STAMP}"
    echo ""
    echo "| cell | transport | mdns | suite | result | secs |"
    echo "|------|-----------|------|-------|--------|------|"
} > "$REPORT"

run_cell() {
    local cell="$1" transport="$2" mdns="$3" suite="$4"
    export CLICKER_TRANSPORT="$transport"
    if [ "$mdns" = "on" ]; then
        export CLICKER_MDNS="on"
    else
        unset CLICKER_MDNS
    fi
    local start
    start=$(date +%s)
    local out
    out=$(RUST_LOG="info,rodio=off,cpal=off,alsa=off" cargo nextest run --test "$suite" --all-features --run-ignored all -- --nocapture 2>&1)
    local code=$?
    local end
    end=$(date +%s)
    local result="FAIL"
    if [ "$code" -eq 0 ]; then
        result="PASS"
    fi
    echo "$out" | tail -5
    echo "| $cell | $transport | $mdns | $suite | $result | $((end - start)) |" >> "$REPORT"
}

run_suite() {
    local suite="$1"
    run_cell "tcp-only" "tcp" "off" "$suite"
    run_cell "quic-only" "quic" "off" "$suite"
    run_cell "both" "both" "off" "$suite"
    run_cell "lan-final" "both" "on" "$suite"
}

if [ "$SUITES" = "all" ] || [ "$SUITES" = "rooms_rejoin" ]; then
    run_suite "rooms_rejoin"
fi
if [ "$SUITES" = "all" ] || [ "$SUITES" = "mainnet_local" ]; then
    run_suite "mainnet_local"
fi

TABLE=$(cat "$REPORT")
cargo run --quiet --all-features --example send_text_msg -- "clicker e2e matrix ${STAMP}" "$TABLE" 2>&1 | tail -2
echo "matrix report: $REPORT"
