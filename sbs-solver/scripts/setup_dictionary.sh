#!/bin/bash
# setup_dictionary.sh
# Downloads a standard English word list for the solver

DATA_DIR="data"
DICT_FILE="$DATA_DIR/dictionary.txt"
# Using words_alpha.txt (only letters, no numbers/symbols)
URL="https://raw.githubusercontent.com/dwyl/english-words/master/words_alpha.txt"

mkdir -p "$DATA_DIR"

if [ -f "$DICT_FILE" ]; then
    echo "Dictionary already exists at $DICT_FILE"
else
    echo "Downloading dictionary from $URL..."
    if command -v curl >/dev/null 2>&1; then
        curl -L -o "$DICT_FILE" "$URL"
    elif command -v wget >/dev/null 2>&1; then
        wget -O "$DICT_FILE" "$URL"
    else
        echo "Error: Neither curl nor wget found. Please download $URL to $DICT_FILE manually."
        exit 1
    fi
    echo "Dictionary downloaded successfully."
fi
