# Ramulator2 Interface Wrapper

This module provides a Rust wrapper for the Ramulator2 memory simulator, exposing
a safe interface to interact with the C++ library through dynamic loading.

## Data Structures

### Request

The `Request` struct represents a memory request in the Ramulator2 system:

````rust
#[repr(C)]
pub struct Request {
    pub addr: i64,                    // Memory address
    pub addr_vec: Vec<i32>,           // Address vector for multi-channel access
    pub type_id: i32,                 // Request type identifier
    pub source_id: i32,               // Source identifier
    pub command: i32,                 // Memory command
    pub final_command: i32,           // Final command after translation
    pub is_stat_updated: bool,        // Statistics update flag
    pub arrive: i64,                  // Arrival timestamp
    pub depart: i64,                  // Departure timestamp
    pub scratchpad: [i32; 4],         // Scratchpad for additional data
    pub callback: Option<extern "C" fn(*mut Request)>, // Completion callback
    pub m_payload: *mut c_void,       // Payload pointer
}

#[repr(C)]
pub struct Response {
    valid: bool,    // If it is a valid response
    addr: usize,    // The address of memory request
    data: Vec<u8>,  // The data
    read_succ: bool, // If the last read request in this cycle succeeds
    write_succ: bool, // If the last write request in this cycle succeeds
    is_write: bool, // Is write
}
````

### MemoryInterface

The `MemoryInterface` struct provides the main interface to interact with Ramulator2:

````rust
pub struct MemoryInterface {
    lib: Library,        // Dynamically loaded library handle
    wrapper: CRamulator2Wrapper,  // Opaque pointer to C++ wrapper object
    write_buffer: VecDeque<(usize, Vec<u8>)>, 
}
````

- `write_buffer` holds data to be written to the memory. A queue is adopted to retain memory order for proper callback handling.

## Exposed Interface

### Initialization

````rust
/// Creates a new MemoryInterface instance by loading the Ramulator2 library.
/// Returns an error if the library cannot be loaded or initialized.
pub unsafe fn new(lib: Library) -> Result<Self, Box<dyn Error>>

/// Initializes the memory system with the specified configuration file.
/// The config_path should point to a valid Ramulator2 configuration file.
pub unsafe fn init(&self, config_path: &str)
````

### Simulation Control

````rust
/// Advances the frontend simulation by one tick.
/// This should be called for each simulation cycle.
pub unsafe fn frontend_tick(&self)

/// Advances the memory system simulation by one tick.
/// This should be called for each simulation cycle.
pub unsafe fn memory_tick(&self)

/// Finalizes the simulation and performs cleanup.
/// Should be called when the simulation is complete.
pub unsafe fn finish(&self)
````

### Memory Operations

````rust
/// Sends a memory request to the Ramulator2 system.
///
/// # Arguments
/// * `addr` - Memory address to access
/// * `is_write` - True for write operation, false for read
/// * `callback` - Function to call when request completes
/// * `ctx` - Context pointer passed to callback
///
/// # Returns
/// * `true` if request was accepted, `false` otherwise
pub unsafe fn send_request(
    &self,
    addr: i64,
    is_write: bool,
    callback: RequestCallback,
    ctx: *mut c_void,
) -> bool
````

## Type Definitions

````rust
type CRamulator2Wrapper = *mut c_void;
type RequestCallback = extern "C" fn(*mut Request, *mut c_void);
type ResponseCallback = extern "C" fn(*mut Response, *mut c_void);
````

## Resource Management

The `MemoryInterface` implements `Drop` to ensure proper cleanup of the underlying
C++ wrapper object when the Rust object goes out of scope. This prevents memory
leaks and ensures the Ramulator2 library is properly finalized.

## Safety Considerations

All public methods are marked as `unsafe` because they interact with C++ code
through FFI. Users must ensure:

1. The library handle remains valid throughout the interface lifetime
2. Configuration files exist and are properly formatted
3. Callbacks are properly implemented and don't cause undefined behavior
4. Memory addresses are valid and properly aligned
5. The interface is not used concurrently from multiple threads without proper synchronization

## Multi-platform Support

As discussed in [Rust simulator generation](../../../python/assassyn/codegen/simulator/simulator.md),
Linux and MacOS has different behaviors on dynamic objects that links other dynamic objects,
and MacOS has to manually specify `RTLD_LAZY | RTLD_GLOBAL` flag.
Thus a platform-related macro shall be imposed.