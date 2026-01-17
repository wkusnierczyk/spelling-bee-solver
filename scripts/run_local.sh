#!/bin/bash
set -e

echo "--- SBS Local Launcher ---"

# 1. Check Pre-requisites
if ! command -v npm &> /dev/null; then
    echo "[Error] 'npm' is not installed. Please install Node.js."
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "[Error] 'cargo' is not installed. Please install Rust."
    exit 1
fi

echo "[1/2] Installing Frontend Dependencies..."
cd sbs-gui
npm install
cd ..

echo ""
echo "--- READY TO START ---"
echo "I will now help you start the Backend and Frontend."
echo "Since these need to run at the same time, please open TWO terminal windows."
echo ""
echo "TERMINAL 1 (Backend):"
echo "  cargo run"
echo ""
echo "TERMINAL 2 (Frontend):"
echo "  cd sbs-gui && npm run dev"
echo ""
echo "Once both are running, open your browser at: http://localhost:5173"
