"""Python wrapper for Ramulator2 memory simulator.

This module provides a Python interface to the Ramulator2 memory simulation
library through the CRamualator2Wrapper C++ wrapper.
"""
import os
import sys
import ctypes
from ctypes import c_void_p, c_char_p, c_float, c_bool, c_int64, CFUNCTYPE

def get_library_paths():
    """Get the paths to the wrapper and ramulator2 shared libraries.

    Constructs paths directly from ASSASSYN_HOME environment variable.

    Returns:
        tuple: (wrapper_lib_path, ramulator2_lib_path) both without extensions.

    Raises:
        FileNotFoundError: If ASSASSYN_HOME environment variable is not set.
    """
    assassyn_home = os.environ.get('ASSASSYN_HOME')
    if not assassyn_home:
        raise FileNotFoundError("ASSASSYN_HOME environment variable not set")

    wrapper_lib_path = os.path.join(assassyn_home, 'tools', 'c-ramulator2-wrapper',
                                    'build', 'lib', 'libwrapper')
    ramulator2_lib_path = os.path.join(assassyn_home, '3rd-party', 'ramulator2',
                                       'libramulator')

    return wrapper_lib_path, ramulator2_lib_path

def load_shared_library(lib_path):
    """Load a shared library with fallback for different extensions.

    Uses RTLD_GLOBAL mode to handle recursive shared object dependencies
    as per simulator.md documentation for macOS compatibility.
    """
    # Check if the path already has an extension
    if os.path.exists(lib_path):
        # Use RTLD_GLOBAL mode for macOS compatibility
        if sys.platform.startswith('darwin'):
            return ctypes.CDLL(lib_path, mode=ctypes.RTLD_GLOBAL)
        return ctypes.CDLL(lib_path)
    # Try different extensions in order of preference
    extensions = ['.dylib', '.so', '.dll']  # macOS, Linux, Windows

    for ext in extensions:
        full_path = lib_path + ext
        if os.path.exists(full_path):
            # Use RTLD_GLOBAL mode for macOS compatibility
            if sys.platform.startswith('darwin'):
                return ctypes.CDLL(full_path, mode=ctypes.RTLD_GLOBAL)
            return ctypes.CDLL(full_path)

    # If no library found, raise an error
    raise FileNotFoundError(f"Could not find shared library at {lib_path} "
                            f"with any supported extension")

# Load libraries using the simplified path construction
wrapper_path, ramulator2_path = get_library_paths()
wrapper = load_shared_library(wrapper_path)
ramulator = load_shared_library(ramulator2_path)


# --- Define Request struct (partial mirror) --- pylint: disable=too-few-public-methods
class Request(ctypes.Structure):
    """Memory request structure mirroring the C++ Request class."""
    _fields_ = [
        ("addr", c_int64),               # Addr_t
        ("addr_vec_placeholder", ctypes.c_byte * 24),  # std::vector dummy (GCC/libstdc++ x86_64)
        ("type_id", ctypes.c_int),
        ("source_id", ctypes.c_int),
        ("command", ctypes.c_int),
        ("final_command", ctypes.c_int),
        ("is_stat_updated", c_bool),
        ("_padding", ctypes.c_byte * 7),   # align to 8 bytes
        ("arrive", c_int64),             # Clk_t
        ("depart", c_int64),             # Clk_t
        ("scratchpad", ctypes.c_int * 4),
        ("callback_placeholder", ctypes.c_byte * 32),  # std::function dummy
        ("m_payload", c_void_p),
    ]

# Define callback type
CALLBACK = CFUNCTYPE(None, c_void_p, c_void_p)
# CRamualator2Wrapper* opaque type
CRamualator2WrapperPtr = c_void_p
# Bind functions
wrapper.dram_new.argtypes = []
wrapper.dram_new.restype = CRamualator2WrapperPtr

wrapper.dram_delete.argtypes = [CRamualator2WrapperPtr]
wrapper.dram_delete.restype = None

wrapper.dram_init.argtypes = [CRamualator2WrapperPtr, c_char_p]
wrapper.dram_init.restype = None

wrapper.get_memory_tCK.argtypes = [CRamualator2WrapperPtr]
wrapper.get_memory_tCK.restype = c_float

wrapper.send_request.argtypes = [CRamualator2WrapperPtr, c_int64, c_bool, CALLBACK, c_void_p]
wrapper.send_request.restype = c_bool

wrapper.finish.argtypes = [CRamualator2WrapperPtr]
wrapper.finish.restype = None

wrapper.frontend_tick.argtypes = [CRamualator2WrapperPtr]
wrapper.frontend_tick.restype = None

wrapper.memory_system_tick.argtypes = [CRamualator2WrapperPtr]
wrapper.memory_system_tick.restype = None


class PyRamulator:
    """Python wrapper for Ramulator2 memory simulator.

    This class provides a high-level interface to interact with the Ramulator2
    memory simulator through the CRamualator2Wrapper C++ wrapper.
    """

    def __init__(self, config_path: str):
        """Initialize PyRamulator with configuration file.

        Args:
            config_path: Path to the YAML configuration file.

        Raises:
            RuntimeError: If the CRamualator2Wrapper instance cannot be created.
        """
        self.obj = wrapper.dram_new()
        if not self.obj:
            raise RuntimeError("Failed to create CRamualator2Wrapper instance")
        wrapper.dram_init(self.obj, config_path.encode('utf-8'))
        self.call_backs = []  # to keep references to callbacks
        self.ctxs = {}  # to keep references to ctx objects

    def __del__(self):
        """Clean up the underlying C++ wrapper instance."""
        if self.obj:
            wrapper.dram_delete(self.obj)
            self.obj = None
    # pylint: disable=invalid-name
    def get_memory_tCK(self) -> float:
        """Get memory clock period (tCK) in nanoseconds.

        Returns:
            Memory clock period in nanoseconds.
        """
        return wrapper.get_memory_tCK(self.obj)

    def finish(self):
        """Finalize the simulation and collect statistics."""
        wrapper.finish(self.obj)

    def frontend_tick(self):
        """Advance the frontend simulation by one clock cycle."""
        wrapper.frontend_tick(self.obj)

    def memory_system_tick(self):
        """Advance the memory system simulation by one clock cycle."""
        wrapper.memory_system_tick(self.obj)

    def send_request(self, addr: int, is_write: bool, callback, ctx) -> bool:
        """Send a memory request to the simulated memory system.

        Args:
            addr: Memory address for the request.
            is_write: True for write request, False for read request.
            callback: Python function to call when request completes.
            ctx: Context object passed to the callback function.

        Returns:
            True if request was successfully enqueued, False otherwise.

        Raises:
            ValueError: If callback is None.
        """
        if callback is None:
            raise ValueError("Callback must not be None")
        # Wrap Python ctx object → store it → get its pointer
        py_obj = ctypes.py_object(ctx)
        ctx_ptr = ctypes.cast(ctypes.pointer(py_obj), c_void_p)
        self.ctxs[ctx_ptr.value] = py_obj

        # C callback wrapper
        def _c_callback(req_ptr, ctx_ptr):
            req = ctypes.cast(req_ptr, ctypes.POINTER(Request)).contents
            # unwrap Python object
            py_obj = self.ctxs.get(ctx_ptr, None)
            ctx_val = py_obj.value
            callback(req, ctx_val)

        c_cb = CALLBACK(_c_callback)
        if c_cb not in self.call_backs:
            self.call_backs.append(c_cb)

        return wrapper.send_request(self.obj, addr, is_write, c_cb, ctx_ptr)
