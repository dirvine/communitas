# Path Sanitization Security Fix - Critical Issue #3

**Date:** 2025-10-26  
**Component:** Website Manager (`communitas-core/src/website/manager.rs`)  
**Severity:** Critical  
**Status:** ✅ Fixed and Tested

## Problem

The Website Manager's path sanitization was vulnerable to directory traversal attacks. The original implementation used weak string replacement:

```rust
let safe_path = path.replace("..", "").replace("//", "/");
```

This approach could be bypassed with patterns like:
- `../../../etc/passwd` - Direct traversal
- `....//..//test.md` - Obfuscated traversal
- `a..b.md` - Consecutive dots
- Very long paths (256+ characters)

## Solution

### 1. Hardened Path Validation Function

Implemented `validate_page_path()` with comprehensive security checks:

```rust
fn validate_page_path(path: &str) -> WebsiteResult<String> {
    // Length limit (255 characters max)
    if path.len() > 255 {
        return Err(WebsiteError::InvalidPath("Path too long".into()));
    }

    // Reject empty paths
    if path.is_empty() {
        return Err(WebsiteError::InvalidPath("Empty path".into()));
    }

    // Split and validate each component
    let parts: Vec<&str> = path.split('/').filter(|c| !c.is_empty()).collect();

    if parts.is_empty() {
        return Err(WebsiteError::InvalidPath("No valid path components".into()));
    }

    let mut components = Vec::new();

    for comp in parts {
        // Block directory traversal
        if comp == "." || comp == ".." {
            return Err(WebsiteError::InvalidPath("Path traversal attempt".into()));
        }

        // Allow only safe characters: alphanumeric, dash, underscore, dot
        if !comp.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
            return Err(WebsiteError::InvalidPath(
                format!("Invalid characters in: {}", comp)
            ));
        }

        // Reject components that are all dots
        if comp.chars().all(|c| c == '.') {
            return Err(WebsiteError::InvalidPath("Invalid component".into()));
        }

        // Reject consecutive dots (prevents obfuscation)
        if comp.contains("..") {
            return Err(WebsiteError::InvalidPath("Consecutive dots not allowed".into()));
        }

        components.push(comp);
    }

    Ok(components.join("/"))
}
```

### 2. Updated `page_doc_id()` Signature

Changed from:
```rust
fn page_doc_id(four_word_address: &str, path: &str) -> String
```

To:
```rust
fn page_doc_id(four_word_address: &str, path: &str) -> WebsiteResult<String>
```

This ensures validation errors propagate to callers.

### 3. Updated All Call Sites

Modified all methods that call `page_doc_id()` to handle the `Result`:
- `save_page()` - line 87
- `load_page()` - line 119
- `load_page_doc()` - line 157
- `delete_page()` - line 254

## Testing

### Security Tests Added

Added comprehensive test coverage in `communitas-core/tests/website_storage_test.rs`:

#### 1. Path Traversal Attack Prevention (`test_path_traversal_attacks`)
Tests rejection of:
- `../../../etc/passwd`
- `../../sensitive.md`
- `../outside.md`
- `....//..//test.md`

**Result:** ✅ All malicious paths correctly rejected

#### 2. Invalid Component Detection (`test_invalid_path_components`)
Tests rejection of:
- `a..b.md` (consecutive dots)

**Result:** ✅ Correctly rejected

#### 3. Valid Path Acceptance (`test_valid_paths_accepted`)
Tests acceptance of legitimate paths:
- `home.md`
- `blog/post-1.md`
- `docs/api/v1.0.md`
- `my_file.md`
- `test-page_2.md`

**Result:** ✅ All valid paths accepted

#### 4. Path Length Limits (`test_path_length_limit`)
Tests:
- Rejection of 256+ character paths
- Acceptance of paths within limits

**Result:** ✅ Correctly enforced

#### 5. Empty Path Rejection (`test_empty_path_rejection`)
Tests rejection of empty string paths

**Result:** ✅ Correctly rejected

### Test Results

```
running 16 tests
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

All existing tests continue to pass, confirming backward compatibility with legitimate use cases.

## Security Validation Rules

The hardened validation enforces:

1. **Length Limit:** Maximum 255 characters
2. **Non-Empty:** Path must contain at least one valid component
3. **No Traversal:** Rejects `.` and `..` components
4. **Safe Characters Only:** Allows only `a-zA-Z0-9`, `-`, `_`, `.`
5. **No Dot Sequences:** Rejects consecutive dots anywhere in path
6. **Component Validation:** Each path segment must pass all checks

## Files Modified

1. `communitas-core/src/website/manager.rs`
   - Added `validate_page_path()` function
   - Updated `page_doc_id()` signature and implementation
   - Updated 4 call sites to handle validation errors

2. `communitas-core/tests/website_storage_test.rs`
   - Added 5 new security-focused test cases
   - 185 lines of comprehensive test coverage

## Migration Impact

- **API Change:** `page_doc_id()` now returns `Result` instead of `String`
- **Backward Compatibility:** All valid paths continue to work
- **Breaking Change:** Invalid/malicious paths that were previously silently sanitized now return errors
- **User Impact:** None for legitimate users; improved security for all

## Verification Commands

```bash
# Run all website storage tests
cargo test -p communitas-core --test website_storage_test

# Run specific security tests
cargo test -p communitas-core --test website_storage_test test_path_traversal_attacks
cargo test -p communitas-core --test website_storage_test test_invalid_path_components
cargo test -p communitas-core --test website_storage_test test_valid_paths_accepted
```

## Recommendations

1. ✅ **Completed:** Hardened path validation implemented
2. ✅ **Completed:** Comprehensive test coverage added
3. 🔄 **Consider:** Add rate limiting for failed path validation attempts
4. 🔄 **Consider:** Log rejected paths for security monitoring
5. 🔄 **Consider:** Add similar validation to other file path inputs in the codebase

## References

- Original Issue: Critical Issue #3 - Harden Path Sanitization
- OWASP Path Traversal: https://owasp.org/www-community/attacks/Path_Traversal
- CWE-22: Improper Limitation of a Pathname to a Restricted Directory
