#!/usr/bin/env python3
import os
import sys

def remove_trailing_spaces(file_path):
    """
    Remove trailing whitespaces from a given file.
    
    :param file_path: Path to the file to process
    """
    try:
        # Read the file
        with open(file_path, 'r') as file:
            lines = file.readlines()
        
        # Remove trailing whitespaces
        stripped_lines = [line.rstrip() + '\n' for line in lines]
        
        # Write back to the file
        with open(file_path, 'w') as file:
            file.writelines(stripped_lines)
        
        print(f"Processed: {file_path}")
    except Exception as e:
        print(f"Error processing {file_path}: {e}")

def main():
    # Check if files are provided as arguments
    if len(sys.argv) < 2:
        print("Usage: python remove_trailing_spaces.py <file1> [file2 ...]")
        sys.exit(1)
    
    # Process each file
    for file_path in sys.argv[1:]:
        if os.path.isfile(file_path):
            remove_trailing_spaces(file_path)
        else:
            print(f"Warning: {file_path} is not a valid file.")

if __name__ == '__main__':
    main()
