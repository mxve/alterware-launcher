#!/bin/bash

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <source_directory> <output_directory>"
    echo "Example: $0 ./my_files ./output"
    exit 1
fi

SOURCE_DIR=$(realpath "$1")
OUTPUT_DIR=$(realpath "$2")
INFO_JSON="$OUTPUT_DIR/info.json"

echo "Processing files from: $SOURCE_DIR"
echo "Output directory: $OUTPUT_DIR"
echo ""

mkdir -p "$OUTPUT_DIR"

PROCESSED_HASHES=$(mktemp)
JSON_CONTENT=$(mktemp)
DELETE_LIST=$(mktemp)
PROCESSED_PATHS=$(mktemp)
trap 'rm -f "$PROCESSED_HASHES" "$JSON_CONTENT" "$DELETE_LIST" "$PROCESSED_PATHS"' EXIT

if [ -f "$INFO_JSON" ]; then
    echo "Found existing info.json, using as base..."
    cp "$INFO_JSON" "$JSON_CONTENT"
else
    echo "Creating new info.json..."
    echo '{"delete_list":[],"files":[]}' > "$JSON_CONTENT"
fi

cd "$SOURCE_DIR" || exit 1

echo "Finding and processing files..."
find . -type f ! -path "$OUTPUT_DIR/*" -print0 | \
while IFS= read -r -d $'\0' file; do
    echo "  Processing: $file"
    rel_path="${file#./}"
    echo "$rel_path" >> "$PROCESSED_PATHS"
    current_hash=$(b3sum "$file" | cut -d ' ' -f 1)
    size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file")

    existing_entry=$(jq -r --arg path "$rel_path" '.files[] | select(.name == $path)' "$JSON_CONTENT")

    if [ -n "$existing_entry" ]; then
        old_hash=$(echo "$existing_entry" | jq -r '.hash')
        if [ "$current_hash" = "$old_hash" ] && [ -f "$OUTPUT_DIR/$old_hash" ]; then
            echo "    Unchanged, skipping..."
            echo "$old_hash" >> "$PROCESSED_HASHES"
            continue
        else
            echo "    Changed, updating..."
            echo "$old_hash" >> "$DELETE_LIST"
        fi
    else
        echo "    New file..."
    fi

    cp "$file" "$OUTPUT_DIR/$current_hash"
    echo "$current_hash" >> "$PROCESSED_HASHES"

    jq --arg path "$rel_path" 'del(.files[] | select(.name == $path))' "$JSON_CONTENT" > "$JSON_CONTENT.tmp" && mv "$JSON_CONTENT.tmp" "$JSON_CONTENT"

    jq --arg path "$rel_path" \
       --arg size "$size" \
       --arg hash "$current_hash" \
       '.files += [{"name": $path, "size": ($size|tonumber), "hash": $hash}]' \
       "$JSON_CONTENT" > "$JSON_CONTENT.tmp" && mv "$JSON_CONTENT.tmp" "$JSON_CONTENT"
done

echo "Removing entries for deleted files..."
while IFS= read -r entry; do
    path=$(echo "$entry" | jq -r '.name')
    hash=$(echo "$entry" | jq -r '.hash')
    if ! grep -Fxq "$path" "$PROCESSED_PATHS"; then
        echo "  Removing entry for deleted file: $path"
        jq --arg path "$path" 'del(.files[] | select(.name == $path))' "$JSON_CONTENT" > "$JSON_CONTENT.tmp" && mv "$JSON_CONTENT.tmp" "$JSON_CONTENT"
        if [ -f "$OUTPUT_DIR/$hash" ]; then
            echo "  Marking for deletion: $hash"
            echo "$hash" >> "$DELETE_LIST"
        fi
    fi
done < <(jq -c '.files[]' "$JSON_CONTENT")

echo "Writing info.json..."
cp "$JSON_CONTENT" "$INFO_JSON"

echo "Cleaning up unused files..."
for file in "$OUTPUT_DIR"/*; do
    if [ "$file" = "$INFO_JSON" ]; then
        continue
    fi
    if [ ! -f "$file" ]; then
        continue
    fi
    basename=$(basename "$file")
    if ! grep -q "^${basename}$" "$PROCESSED_HASHES"; then
        echo "  Removing unused file: $basename"
        rm "$file"
    fi
done

echo "Removing old and deleted files..."
sort -u "$DELETE_LIST" | while read -r hash; do
    if [ -f "$OUTPUT_DIR/$hash" ]; then
        echo "  Removing: $hash"
        rm "$OUTPUT_DIR/$hash"
    fi
done

echo ""
echo "Done! Files processed and info.json updated."