#include "./CWrapper.h"


void CWrapper::init(const std::string& config_path){
    YAML::Node config = Ramulator::Config::parse_config_file(config_path, {});
    ramulator2_frontend = Ramulator::Factory::create_frontend(config);
    ramulator2_memorysystem = Ramulator::Factory::create_memory_system(config);

    ramulator2_frontend->connect_memory_system(ramulator2_memorysystem);
    ramulator2_memorysystem->connect_frontend(ramulator2_frontend);
}

float CWrapper::get_memory_tCK() const {
    return ramulator2_memorysystem->get_tCK();
}


bool CWrapper::send_request(int64_t addr, bool is_write, std::function<void(Ramulator::Request&)> callback) {
    bool enqueue_success;
    enqueue_success = ramulator2_frontend->receive_external_requests(is_write, addr, 0, callback);
    return enqueue_success;
}

void CWrapper::finish(){
    ramulator2_frontend->finalize();
    ramulator2_memorysystem->finalize();
}

void CWrapper::frontend_tick(){
    ramulator2_frontend->tick();
}

void CWrapper::memory_system_tick(){
    ramulator2_memorysystem->tick();
}

CWrapper::~CWrapper() {
    if(ramulator2_frontend) {
        delete ramulator2_frontend;
        ramulator2_frontend = nullptr;
    }
    if(ramulator2_memorysystem) {
        delete ramulator2_memorysystem;
        ramulator2_memorysystem = nullptr;
    }
}

extern "C" {

    // Factory: create a new MyWrapper instance
    CWrapper* dram_new() {
        return new CWrapper();
    }
    
    // Destructor: delete a MyWrapper instance
    void dram_delete(CWrapper* obj) {
        delete obj;
    }
    
    // Wrap init method: pass config path as C string
    void dram_init(CWrapper* obj, const char* config_path) {
        obj->init(std::string(config_path));
    }
    
    // Wrap get_memory_tCK method
    float get_memory_tCK(CWrapper* obj) {
        return obj->get_memory_tCK();
    }
    
    // Wrap send_request method
    bool send_request(CWrapper* obj, int64_t addr, bool is_write, void (*callback)(Ramulator::Request*, void*), void* ctx) {
        return obj->send_request(addr, is_write, 
            [callback, ctx](Ramulator::Request& req) {
                callback(&req, ctx);
            });
    }
    
    // Wrap finish method
    void finish(CWrapper* obj) {
        obj->finish();
    }
    
    // Wrap tick method
    void frontend_tick(CWrapper* obj) {
        obj->frontend_tick();
    }

    void memory_system_tick(CWrapper* obj) {
        obj->memory_system_tick();
    }
    
}