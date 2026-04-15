#!/bin/bash

# Lightweight patch application script
# Usage: patch-apply.sh [apply|reverse|check] <patch-file>

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PATCH_FILE="$2"
MODE="$1"

if [ $# -ne 2 ]; then
    echo "Usage: $0 [apply|reverse|check] <patch-file>" >&2
    exit 1
fi

if [ ! -f "$PATCH_FILE" ]; then
    echo "Error: Patch file '$PATCH_FILE' not found" >&2
    exit 1
fi

# Parse patch file and apply changes
apply_patch() {
    local current_file=""
    local original_line=""
    local replacement_lines=()
    local in_replacement=false
    local line=""
    
    exec 3< "$PATCH_FILE"
    while IFS= read -r line <&3 || [ -n "$line" ]; do
        line="${line%$'\r'}"
        # Skip empty lines
        if [ -z "$line" ]; then
            continue
        fi
        
        # Check if this is a file path (doesn't start with - or +)
        if [[ ! "$line" =~ ^[+-] ]]; then
            # Process previous file if we have one
            if [ -n "$current_file" ] && [ -n "$original_line" ]; then
                process_file "$current_file" "$original_line" "${replacement_lines[@]}"
            fi
            
            # Start new file
            current_file="$line"
            original_line=""
            replacement_lines=()
            in_replacement=false
            continue
        fi
        
        # Check if this is an original line to replace
        if [[ "$line" =~ ^- ]]; then
            # Process previous file if we have one
            if [ -n "$current_file" ] && [ -n "$original_line" ]; then
                process_file "$current_file" "$original_line" "${replacement_lines[@]}"
            fi
            
            # Start new replacement
            original_line="${line:1}"  # Remove the - prefix
            replacement_lines=()
            in_replacement=true
            continue
        fi
        
        # Check if this is a replacement line
        if [[ "$line" =~ ^\+ ]]; then
            if [ "$in_replacement" = true ]; then
                replacement_lines+=("${line:1}")  # Remove the + prefix
            else
                echo "Error: Found + line without preceding - line" >&2
                exit 1
            fi
        fi
    done
    exec 3<&-
    
    # Process the last file
    if [ -n "$current_file" ] && [ -n "$original_line" ]; then
        process_file "$current_file" "$original_line" "${replacement_lines[@]}"
    fi
}

# Process a single file replacement
process_file() {
    local file_path="$1"
    local original="$2"
    shift 2
    local replacements=("$@")
    local line=""
    
    if [ ! -f "$file_path" ]; then
        echo "Error: Target file '$file_path' not found" >&2
        exit 1
    fi
    
    # Create temporary file
    local temp_file=$(mktemp)
    
    # Find and replace the first matching original line.
    local found=false
    
    while IFS= read -r line || [ -n "$line" ]; do
        line="${line%$'\r'}"
        if [ "$found" = false ] && [ "$line" = "$original" ]; then
            # Found the original line, replace it with all replacement lines.
            found=true
            for replacement in "${replacements[@]}"; do
                echo "$replacement" >> "$temp_file"
            done
        else
            echo "$line" >> "$temp_file"
        fi
    done < "$file_path"
    
    if [ "$found" = false ]; then
        echo "Error: Original line not found in '$file_path': '$original'" >&2
        rm -f "$temp_file"
        exit 1
    fi
    
    # Replace original file with patched version
    mv "$temp_file" "$file_path"
    echo "Applied patch to '$file_path'"
}

# Reverse patch (find + lines and replace with - line)
reverse_patch() {
    local current_file=""
    local original_line=""
    local replacement_lines=()
    local in_replacement=false
    local line=""
    
    exec 3< "$PATCH_FILE"
    while IFS= read -r line <&3 || [ -n "$line" ]; do
        line="${line%$'\r'}"
        # Skip empty lines
        if [ -z "$line" ]; then
            continue
        fi
        
        # Check if this is a file path
        if [[ ! "$line" =~ ^[+-] ]]; then
            # Process previous file if we have one
            if [ -n "$current_file" ] && [ -n "$original_line" ]; then
                reverse_file "$current_file" "$original_line" "${replacement_lines[@]}"
            fi
            
            # Start new file
            current_file="$line"
            original_line=""
            replacement_lines=()
            in_replacement=false
            continue
        fi
        
        # Check if this is an original line
        if [[ "$line" =~ ^- ]]; then
            # Process previous file if we have one
            if [ -n "$current_file" ] && [ -n "$original_line" ]; then
                reverse_file "$current_file" "$original_line" "${replacement_lines[@]}"
            fi
            
            # Start new replacement
            original_line="${line:1}"  # Remove the - prefix
            replacement_lines=()
            in_replacement=true
            continue
        fi
        
        # Check if this is a replacement line
        if [[ "$line" =~ ^\+ ]]; then
            if [ "$in_replacement" = true ]; then
                replacement_lines+=("${line:1}")  # Remove the + prefix
            else
                echo "Error: Found + line without preceding - line" >&2
                exit 1
            fi
        fi
    done
    exec 3<&-
    
    # Process the last file
    if [ -n "$current_file" ] && [ -n "$original_line" ]; then
        reverse_file "$current_file" "$original_line" "${replacement_lines[@]}"
    fi
}

# Reverse a single file replacement
reverse_file() {
    local file_path="$1"
    local original="$2"
    shift 2
    local replacements=("$@")
    
    if [ ! -f "$file_path" ]; then
        echo "Error: Target file '$file_path' not found" >&2
        exit 1
    fi
    
    local temp_file=$(mktemp)
    local found=false
    local lines=()
    local i=0
    local j=0
    local matches=true

    mapfile -t lines < "$file_path"
    for i in "${!lines[@]}"; do
        lines[$i]="${lines[$i]%$'\r'}"
    done

    i=0

    while [ "$i" -lt "${#lines[@]}" ]; do
        matches=true

        if [ "$found" = false ] && [ "${#replacements[@]}" -gt 0 ] && \
           [ $((i + ${#replacements[@]})) -le "${#lines[@]}" ]; then
            j=0
            while [ "$j" -lt "${#replacements[@]}" ]; do
                if [ "${lines[$((i + j))]}" != "${replacements[$j]}" ]; then
                    matches=false
                    break
                fi
                j=$((j + 1))
            done

            if [ "$matches" = true ]; then
                echo "$original" >> "$temp_file"
                found=true
                i=$((i + ${#replacements[@]}))
                continue
            fi
        fi

        echo "${lines[$i]}" >> "$temp_file"
        i=$((i + 1))
    done
    
    if [ "$found" = false ]; then
        echo "Error: Replacement lines not found in '$file_path'" >&2
        rm -f "$temp_file"
        exit 1
    fi
    
    # Replace original file with reversed version
    mv "$temp_file" "$file_path"
    echo "Reversed patch in '$file_path'"
}

# Check if patch is already applied
check_patch() {
    local current_file=""
    local original_line=""
    local replacement_lines=()
    local in_replacement=false
    local all_applied=true
    local line=""
    
    exec 3< "$PATCH_FILE"
    while IFS= read -r line <&3 || [ -n "$line" ]; do
        line="${line%$'\r'}"
        # Skip empty lines
        if [ -z "$line" ]; then
            continue
        fi
        
        # Check if this is a file path (doesn't start with - or +)
        if [[ ! "$line" =~ ^[+-] ]]; then
            # Process previous file if we have one
            if [ -n "$current_file" ] && [ -n "$original_line" ]; then
                if ! check_file "$current_file" "$original_line" "${replacement_lines[@]}"; then
                    all_applied=false
                fi
            fi
            
            # Start new file
            current_file="$line"
            original_line=""
            replacement_lines=()
            in_replacement=false
            continue
        fi
        
        # Check if this is an original line to replace
        if [[ "$line" =~ ^- ]]; then
            # Process previous file if we have one
            if [ -n "$current_file" ] && [ -n "$original_line" ]; then
                if ! check_file "$current_file" "$original_line" "${replacement_lines[@]}"; then
                    all_applied=false
                fi
            fi
            
            # Start new replacement
            original_line="${line:1}"  # Remove the - prefix
            replacement_lines=()
            in_replacement=true
            continue
        fi
        
        # Check if this is a replacement line
        if [[ "$line" =~ ^\+ ]]; then
            if [ "$in_replacement" = true ]; then
                replacement_lines+=("${line:1}")  # Remove the + prefix
            else
                echo "Error: Found + line without preceding - line" >&2
                exit 1
            fi
        fi
    done
    exec 3<&-
    
    # Process the last file
    if [ -n "$current_file" ] && [ -n "$original_line" ]; then
        if ! check_file "$current_file" "$original_line" "${replacement_lines[@]}"; then
            all_applied=false
        fi
    fi
    
    if [ "$all_applied" = true ]; then
        echo "Patch is already applied"
        exit 0
    else
        echo "Patch is not applied"
        exit 1
    fi
}

# Check if a single file has the patch applied
check_file() {
    local file_path="$1"
    local original="$2"
    shift 2
    local replacements=("$@")
    
    
    if [ ! -f "$file_path" ]; then
        echo "Error: Target file '$file_path' not found"
        return 1
    fi
    
    local lines=()
    local i=0
    local j=0
    local matches=true

    mapfile -t lines < "$file_path"
    for i in "${!lines[@]}"; do
        lines[$i]="${lines[$i]%$'\r'}"
    done

    i=0

    while [ "$i" -lt "${#lines[@]}" ]; do
        matches=true

        if [ "${#replacements[@]}" -gt 0 ] && \
           [ $((i + ${#replacements[@]})) -le "${#lines[@]}" ]; then
            j=0
            while [ "$j" -lt "${#replacements[@]}" ]; do
                if [ "${lines[$((i + j))]}" != "${replacements[$j]}" ]; then
                    matches=false
                    break
                fi
                j=$((j + 1))
            done

            if [ "$matches" = true ]; then
                return 0
            fi
        fi

        if [ "${lines[$i]}" = "$original" ]; then
            return 1
        fi

        i=$((i + 1))
    done

    echo "Warning: Neither original nor replacement lines found in '$file_path'" >&2
    return 1
}

# Main execution
case "$MODE" in
    "apply")
        apply_patch
        ;;
    "reverse")
        reverse_patch
        ;;
    "check")
        check_patch
        ;;
    *)
        echo "Error: Invalid mode '$MODE'. Use 'apply', 'reverse', or 'check'" >&2
        exit 1
        ;;
esac
