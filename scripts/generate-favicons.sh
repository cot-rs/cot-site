#!/bin/bash

set -e

IMAGES_DIR=static/images
inkscape --export-type=png --export-png-compression=9 -w 32 "$IMAGES_DIR/favicon.svg" -o "$IMAGES_DIR/favicon-32.png"
inkscape --export-type=png --export-png-compression=9 -w 180 "$IMAGES_DIR/favicon.svg" -o "$IMAGES_DIR/favicon-180.png"
inkscape --export-type=png --export-png-compression=9 -w 192 "$IMAGES_DIR/favicon.svg" -o "$IMAGES_DIR/favicon-192.png"
inkscape --export-type=png --export-png-compression=9 -w 512 "$IMAGES_DIR/favicon.svg" -o "$IMAGES_DIR/favicon-512.png"
magick static/images/favicon-512.png -define icon:auto-resize=16,32,48 assets/favicon.ico
