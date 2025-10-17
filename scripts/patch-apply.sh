#!/bin/bash

# Lightweight patch application script
# Usage: patch-apply.sh [apply|reverse|check] <patch-file>

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PATCH_FILE="$2"
MODE="$1"

if [ $# -ne 2 ]; then
    echo "Usage: $0 [apply|reverse|check] <patch-file>"
    exit 1
fi

if [ ! -f "$PATCH_FILE" ]; then
    echo "Error: Patch file '$PATCH_FILE' not found"
    exit 1
fi

# Parse patch file and apply changes
apply_patch() {
    local current_file=""
    local original_line=""
    local replacement_lines=()
    local in_replacement=false
    
    while IFS= read -r line; do
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
        if [[ "$line" =~ ^+ ]]; then
            if [ "$in_replacement" = true ]; then
                replacement_lines+=("${line:1}")  # Remove the + prefix
            else
                echo "Error: Found + line without preceding - line"
                exit 1
            fi
        fi
    done < "$PATCH_FILE"
    
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
    
    if [ ! -f "$file_path" ]; then
        echo "Error: Target file '$file_path' not found"
        exit 1
    fi
    
    # Create temporary file
    local temp_file=$(mktemp)
    
    # Find and replace the original line
    local found=false
    local in_replacement=false
    local replacement_count=0
    
    while IFS= read -r line; do
        if [ "$found" = false ] && [ "$line" = "$original" ]; then
            # Found the original line, replace with all replacement lines
            found=true
            for replacement in "${replacements[@]}"; do
                echo "$replacement" >> "$temp_file"
            done
        elif [ "$found" = true ] && [ "$replacement_count" -lt "${#replacements[@]}" ]; then
            # Skip lines that match our replacements (they're already in the file)
            replacement_count=$((replacement_count + 1))
        else
            # Normal line, copy as-is
            echo "$line" >> "$temp_file"
        fi
    done < "$file_path"
    
    if [ "$found" = false ]; then
        echo "Error: Original line not found in '$file_path': '$original'"
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
    
    while IFS= read -r line; do
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
        if [[ "$line" =~ ^+ ]]; then
            if [ "$in_replacement" = true ]; then
                replacement_lines+=("${line:1}")  # Remove the + prefix
            else
                echo "Error: Found + line without preceding - line"
                exit 1
            fi
        fi
    done < "$PATCH_FILE"
    
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
        echo "Error: Target file '$file_path' not found"
        exit 1
    fi
    
    # Create temporary file
    local temp_file=$(mktemp)
    
    # Find consecutive replacement lines and replace with original
    local found=false
    local replacement_count=0
    local skip_count=0
    
    while IFS= read -r line; do
        if [ "$found" = false ] && [ "$replacement_count" -lt "${#replacements[@]}" ]; then
            # Check if this line matches the first replacement
            if [ "$line" = "${replacements[$replacement_count]}" ]; then
                if [ "$replacement_count" -eq 0 ]; then
                    # Found start of replacement block
                    found=true
                    skip_count=1
                    replacement_count=1
                    # Write the original line instead
                    echo "$original" >> "$temp_file"
                else
                    # This shouldn't happen if we're tracking correctly
                    echo "Error: Unexpected replacement line found"
                    rm -f "$temp_file"
                    exit 1
                fi
            else
                # Normal line, copy as-is
                echo "$line" >> "$temp_file"
            fi
        elif [ "$found" = true ] && [ "$skip_count" -lt "${#replacements[@]}" ]; then
            # Skip the remaining replacement lines
            skip_count=$((skip_count + 1))
        else
            # Normal line, copy as-is
            echo "$line" >> "$temp_file"
        fi
    done < "$file_path"
    
    if [ "$found" = false ]; then
        echo "Error: Replacement lines not found in '$file_path'"
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
    
    while IFS= read -r line; do
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
        if [[ "$line" =~ ^+ ]]; then
            if [ "$in_replacement" = true ]; then
                replacement_lines+=("${line:1}")  # Remove the + prefix
            else
                echo "Error: Found + line without preceding - line"
                exit 1
            fi
        fi
    done < "$PATCH_FILE"
    
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
    
    # Look for the replacement lines in sequence
    local replacement_count=0
    local found_original=false
    
    # Use a subshell to avoid file descriptor conflicts
    (
        while IFS= read -r line; do
            if [ "$replacement_count" -lt "${#replacements[@]}" ]; then
                if [ "$line" = "${replacements[$replacement_count]}" ]; then
                    replacement_count=$((replacement_count + 1))
                elif [ "$line" = "$original" ]; then
                    found_original=true
                    break
                fi
            elif [ "$line" = "$original" ]; then
                found_original=true
                break
            fi
        done < "$file_path"
        
        if [ "$replacement_count" -eq "${#replacements[@]}" ]; then
            # All replacement lines found in sequence
            exit 0
        elif [ "$found_original" = true ]; then
            # Original line found, patch not applied
            exit 1
        else
            # Neither found, file might be in unexpected state
            echo "Warning: Neither original nor replacement lines found in '$file_path'"
            exit 1
        fi
    )
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
        echo "Error: Invalid mode '$MODE'. Use 'apply', 'reverse', or 'check'"
        exit 1
        ;;
esac
