import os
import ctypes
from ctypes import c_void_p, c_char_p, c_float, c_bool, c_int64, CFUNCTYPE, POINTER

home = os.getenv('ASSASSYN_HOME', os.getcwd())
wrapper_lib_path = os.path.abspath(f"{home}/testbench/simulator/build/lib/libwrapper.so")
ramulator_lib_path = os.path.abspath(f"{home}/3rd-party/ramulator2/libramulator.so")
wrapper = ctypes.CDLL(wrapper_lib_path)
ramulator = ctypes.CDLL(ramulator_lib_path)


# --- Define Request struct (partial mirror) ---
class Request(ctypes.Structure):
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
# MyWrapper* opaque type
MyWrapperPtr = c_void_p
# Bind functions
wrapper.dram_new.argtypes = []
wrapper.dram_new.restype = MyWrapperPtr

wrapper.dram_delete.argtypes = [MyWrapperPtr]
wrapper.dram_delete.restype = None

wrapper.dram_init.argtypes = [MyWrapperPtr, c_char_p]
wrapper.dram_init.restype = None

wrapper.get_memory_tCK.argtypes = [MyWrapperPtr]
wrapper.get_memory_tCK.restype = c_float

wrapper.send_request.argtypes = [MyWrapperPtr, c_int64, c_bool, CALLBACK, c_void_p]
wrapper.send_request.restype = c_bool

wrapper.MyWrapper_finish.argtypes = [MyWrapperPtr]
wrapper.MyWrapper_finish.restype = None

wrapper.frontend_tick.argtypes = [MyWrapperPtr]
wrapper.frontend_tick.restype = None

wrapper.memory_system_tick.argtypes = [MyWrapperPtr]
wrapper.memory_system_tick.restype = None

class PyRamulator:
    def __init__(self, config_path: str):
        self.obj = wrapper.dram_new()
        if not self.obj:
            raise RuntimeError("Failed to create MyWrapper instance")
        wrapper.dram_init(self.obj, config_path.encode('utf-8'))
        self.call_backs = []  # to keep references to callbacks
        self.ctxs = {}  # to keep references to ctx objects

    
    def __del__(self):
        if self.obj:
            wrapper.dram_delete(self.obj)
            self.obj = None 
    
    def get_memory_tCK(self) -> float:
        return wrapper.get_memory_tCK(self.obj)
    
    def finish(self):
        wrapper.MyWrapper_finish(self.obj)
    
    def frontend_tick(self):
        wrapper.frontend_tick(self.obj)
    
    def memory_system_tick(self):
        wrapper.memory_system_tick(self.obj)

    def send_request(self, addr: int, is_write: bool, callback, ctx) -> bool:
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