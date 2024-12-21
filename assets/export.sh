#!/usr/bin/env bash

optimize_svg() {
  local in="${1:-expected input svg}"
  local out="${2:-expected output svg}"

  scour "$in" "$out" \
    --create-groups \
    --strip-xml-prolog \
    --remove-titles \
    --remove-descriptions \
    --remove-descriptive-elements \
    --enable-comment-stripping \
    --enable-viewboxing \
    --indent=none \
    --no-line-breaks \
    --strip-xml-space \
    --enable-id-stripping \
    --shorten-ids
}

optimize_svg logo.inkscape.svg logo.svg
