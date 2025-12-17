// Minimal FFI test
import Foundation

// Load the dynamic library directly
let libraryPath = "/Users/davidirvine/Desktop/Devel/projects/communitas/target/aarch64-apple-darwin/debug/libcommunitas_bindings.dylib"

guard let handle = dlopen(libraryPath, RTLD_NOW) else {
    let error = String(cString: dlerror())
    print("Failed to load library: \(error)")
    exit(1)
}

print("Successfully loaded library!")

// Try to find the checksum function (UniFFI always exports this)
if let sym = dlsym(handle, "uniffi_communitas_bindings_checksum_method_communitasclient_get_profile") {
    print("Found symbol: uniffi_communitas_bindings_checksum_method_communitasclient_get_profile")
} else {
    print("Symbol not found")
}

dlclose(handle)
print("Test complete!")
