# Patch Apply Script Documentation

## Overview

The `patch-apply.sh` script is a lightweight patch management tool that replaces the traditional git patch system with a custom format designed for exact line matching and flexible patch operations. This script eliminates commit hash dependencies while maintaining precise control over file modifications.

## Purpose

This script was created to address limitations of the git patch system:

- **Commit Hash Dependencies**: Git patches require specific commit hashes, making them fragile
- **Complex Format**: Git patches have complex headers and metadata
- **Limited Flexibility**: Difficult to modify or adapt patches for different contexts

The lightweight format provides:
- **Exact Line Matching**: Whitespace-sensitive matching for precise control
- **Flexible Replacements**: One original line can be replaced with multiple new lines
- **Reverse Operations**: Patches can be cleanly reversed
- **Simple Format**: Easy to read, write, and maintain

## Patch Format

### Basic Structure

```
path/to/target/file
-original line to be replaced
+replacement line 1
+replacement line 2
+replacement line 3

path/to/another/file
-another original line
+another replacement line
```

### Format Rules

1. **File Path**: First line specifies the target file (relative to working directory)
2. **Original Line**: Lines starting with `-` indicate content to be removed
3. **Replacement Lines**: Lines starting with `+` indicate content to be added
4. **Exact Matching**: All matching is whitespace-sensitive
5. **Single Replacement**: Each patch block contains exactly one replacement operation
6. **Multi-file Support**: Multiple files can be patched in a single patch file

### Example

```patch
frontends/PyCDE/setup.py
-      cmake_args += ["-DESI_RUNTIME=ON"]
+      cmake_args += ["-DESI_RUNTIME=OFF"]

configure.ac
-_MY_CXX_CHECK_SET(CFG_CXXFLAGS_STD_NEWEST,-std=gnu++17)
-_MY_CXX_CHECK_SET(CFG_CXXFLAGS_STD_NEWEST,-std=c++17)
+_MY_CXX_CHECK_SET(CFG_CXXFLAGS_STD_NEWEST,-std=gnu++20)
+_MY_CXX_CHECK_SET(CFG_CXXFLAGS_STD_NEWEST,-std=c++20)
+#_MY_CXX_CHECK_SET(CFG_CXXFLAGS_STD_NEWEST,-std=gnu++17)
+#_MY_CXX_CHECK_SET(CFG_CXXFLAGS_STD_NEWEST,-std=c++17)
```

## Usage

### Command Syntax

```bash
bash scripts/patch-apply.sh <mode> <patch_file>
```

### Modes

#### Apply Mode
```bash
bash scripts/patch-apply.sh apply <patch_file>
```

**Purpose**: Applies patches to target files

**Behavior**:
- Reads the patch file and processes each file block
- Finds the original line (`-` prefixed) in the target file
- Replaces it with all replacement lines (`+` prefixed)
- Creates backup of original file before modification
- Reports success/failure for each file

**Example**:
```bash
cd 3rd-party/circt
bash ../../scripts/patch-apply.sh apply ../../scripts/init/patches/circt.patch
```

#### Reverse Mode
```bash
bash scripts/patch-apply.sh reverse <patch_file>
```

**Purpose**: Reverses previously applied patches

**Behavior**:
- Finds consecutive replacement lines (`+` prefixed) in target files
- Replaces them with the original line (`-` prefixed)
- Restores files to their pre-patch state
- Reports success/failure for each file

**Example**:
```bash
cd 3rd-party/circt
bash ../../scripts/patch-apply.sh reverse ../../scripts/init/patches/circt.patch
```

#### Check Mode
```bash
bash scripts/patch-apply.sh check <patch_file>
```

**Purpose**: Determines if patches are already applied

**Behavior**:
- Checks if replacement lines exist in target files
- Returns exit code 0 if patches are applied
- Returns exit code 1 if patches are not applied
- Used by build systems to skip unnecessary patch operations

**Example**:
```bash
cd 3rd-party/circt
if bash ../../scripts/patch-apply.sh check ../../scripts/init/patches/circt.patch; then
    echo "Patches already applied"
else
    echo "Need to apply patches"
fi
```

## Implementation Details

### File Processing

The script processes patch files by:

1. **Reading Patch File**: Parses the patch file line by line
2. **Identifying Blocks**: Separates file blocks (file path + patch lines)
3. **Processing Each File**: For each file block:
   - Validates target file exists
   - Reads target file content
   - Performs the requested operation (apply/reverse/check)
   - Reports results

### Error Handling

The script includes comprehensive error handling:

- **File Not Found**: Reports missing target files
- **No Match Found**: Reports when original lines cannot be found
- **Multiple Matches**: Detects ambiguous matches and reports errors
- **Invalid Format**: Validates patch file format
- **Backup Creation**: Creates backups before modifications

### File Descriptor Management

The script uses subshells to prevent file descriptor conflicts when processing multiple files in a single patch file.

## Integration with Build System

### Makefile Integration

The script is integrated into the build system through Makefile include files:

```makefile
# Patch application target
3rd-party/circt/.patch-applied:
	@cd 3rd-party/circt && \
		if bash ../../scripts/patch-apply.sh check ../../scripts/init/patches/circt.patch 2>/dev/null; then \
			echo "CIRCT patch already applied — skipping patch step."; \
		else \
			echo "Applying CIRCT patch..."; \
			bash ../../scripts/patch-apply.sh apply ../../scripts/init/patches/circt.patch; \
		fi && \
		touch .patch-applied
```

### Marker Files

The build system uses marker files to track patch and build states:

- **`.patch-applied`**: Indicates patches have been applied
- **`.xxx-built`**: Indicates builds have been completed

These enable incremental builds and prevent unnecessary work.

## Examples

### Simple Single-Line Replacement

**Patch File** (`example.patch`):
```patch
src/config.h
-#define DEBUG_MODE 1
+#define DEBUG_MODE 0
```

**Apply**:
```bash
bash scripts/patch-apply.sh apply example.patch
```

**Reverse**:
```bash
bash scripts/patch-apply.sh reverse example.patch
```

### Multi-Line Replacement

**Patch File** (`complex.patch`):
```patch
src/main.cpp
-    // Old implementation
-    return old_function();
+    // New implementation
+    if (condition) {
+        return new_function();
+    }
+    return fallback_function();
```

### Multi-File Patch

**Patch File** (`multi.patch`):
```patch
src/file1.cpp
-old_code();
+new_code();

src/file2.h
-#define OLD_MACRO
+#define NEW_MACRO
```

## Best Practices

### Writing Patch Files

1. **Use Relative Paths**: Paths should be relative to the working directory where the script is executed
2. **Exact Whitespace**: Match whitespace exactly, including tabs and spaces
3. **Single Operations**: Each patch block should contain exactly one replacement operation
4. **Clear Comments**: Use meaningful comments in replacement lines when helpful

### Using in Build Scripts

1. **Check Before Apply**: Always check if patches are already applied before applying
2. **Error Handling**: Check exit codes and handle errors appropriately
3. **Working Directory**: Execute the script from the correct working directory
4. **Marker Files**: Use marker files to track patch application state

### Testing Patches

1. **Test Apply/Reverse**: Always test both apply and reverse operations
2. **Verify Content**: Check that file contents are correct after operations
3. **Test Edge Cases**: Test with files that don't exist, already patched, etc.

## Troubleshooting

### Common Issues

**"Target file not found"**
- Ensure the patch file path is correct relative to working directory
- Check that the target file exists

**"Original line not found"**
- Verify the original line matches exactly (including whitespace)
- Check if the patch was already applied

**"Multiple matches found"**
- The original line appears multiple times in the file
- Make the original line more specific or unique

**"Replacement lines not found"**
- Trying to reverse a patch that wasn't applied
- Check if the patch was actually applied

### Debug Tips

1. **Verbose Output**: The script provides detailed output for debugging
2. **Check File Contents**: Manually inspect files before and after operations
3. **Test Incrementally**: Test with simple patches before complex ones
4. **Backup Files**: The script creates backups; use them to restore if needed

## Migration from Git Patches

### Converting Git Patches

To convert from git patches to the lightweight format:

1. **Extract File Path**: Get the target file path from git patch headers
2. **Identify Changes**: Find `-` and `+` lines in the git patch
3. **Create New Format**: Write in the lightweight format
4. **Test Thoroughly**: Verify the new patch works correctly

### Example Conversion

**Git Patch**:
```patch
diff --git a/src/config.h b/src/config.h
index 1234567..abcdefg 100644
--- a/src/config.h
+++ b/src/config.h
@@ -1,3 +1,3 @@
 #define VERSION "1.0"
-#define DEBUG 1
+#define DEBUG 0
 #define FEATURE_X 1
```

**Lightweight Format**:
```patch
src/config.h
-#define DEBUG 1
+#define DEBUG 0
```

## See Also

- `scripts/init/README.md` - Build system documentation
- `scripts/init/patches/` - Example patch files
- `scripts/init/*.inc` - Makefile integration examples
