#!/usr/bin/env sh
# Rebuild the checked-in example PDFs.
set -eu

script_dir=$(cd "$(dirname "$0")" && pwd)
tome=${TOME:-tome}

if ! command -v "$tome" >/dev/null 2>&1; then
  printf 'tome is not installed; install it or set TOME to the binary path\n' >&2
  exit 1
fi

# Each example writes its rendered PDF next to example.md.
for name in letter-din cv notes recipe flyer; do
  "$tome" "$script_dir/$name/example.md" \
    --template "$script_dir/$name/template.typ" \
    --output "$script_dir/$name/example.pdf" \
    --force
done

printf 'wrote example PDFs in %s/*/example.pdf\n' "$script_dir"
