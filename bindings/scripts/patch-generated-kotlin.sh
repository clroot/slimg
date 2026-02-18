#!/usr/bin/env bash
# Patch UniFFI-generated Kotlin code for Kotlin 2.x compatibility.
# The generated SlimgException subclasses declare a `message` constructor
# parameter that conflicts with Throwable.message.
set -euo pipefail

GENERATED_FILE="$1"

if [[ ! -f "$GENERATED_FILE" ]]; then
    echo "Error: File not found: $GENERATED_FILE"
    exit 1
fi

# In SlimgException subclasses, rename constructor parameter `message` to `msg`
# and update the corresponding message override.
sed -i.bak -E '
    /class (Crop|Decode|Encode|Resize|Io|Image)\(/,/\}/ {
        s/val `message`: kotlin\.String/val `msg`: kotlin.String/
        s/get\(\) = "message=\$\{ `message` \}"/get() = "message=${ `msg` }"/
    }
' "$GENERATED_FILE"

# Also update FfiConverterTypeSlimgError references
sed -i.bak -E '
    s/SlimgException\.Crop\(`message`/SlimgException.Crop(`msg`/
    s/SlimgException\.Decode\(`message`/SlimgException.Decode(`msg`/
    s/SlimgException\.Encode\(`message`/SlimgException.Encode(`msg`/
    s/SlimgException\.Resize\(`message`/SlimgException.Resize(`msg`/
    s/SlimgException\.Io\(`message`/SlimgException.Io(`msg`/
    s/SlimgException\.Image\(`message`/SlimgException.Image(`msg`/
' "$GENERATED_FILE"

rm -f "${GENERATED_FILE}.bak"
echo "Patched: $GENERATED_FILE"
