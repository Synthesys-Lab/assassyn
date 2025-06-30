#include "./MyWrapper.h"


void MyWrapper::init(const std::string& config_path){
    YAML::Node config = Ramulator::Config::parse_config_file(config_path, {});
    ramulator2_frontend = Ramulator::Factory::create_frontend(config);
    ramulator2_memorysystem = Ramulator::Factory::create_memory_system(config);

    ramulator2_frontend->connect_memory_system(ramulator2_memorysystem);
    ramulator2_memorysystem->connect_frontend(ramulator2_frontend);
}

float MyWrapper::get_memory_tCK() const {
    return ramulator2_memorysystem->get_tCK();
}

//void response(Packet pkt, int64_t latency);

bool MyWrapper::send_request(int64_t addr, bool is_write, std::function<void(Ramulator::Request&)> callback) {
    bool enqueue_success;
    enqueue_success = ramulator2_frontend->receive_external_requests(is_write, addr, 0, callback);
    // if (enqueue_success) {
    //     // only the write success will go inside it.
    //     if (pkt.is_write){
    //         //write the pkt into it
    //         memory[pkt.addr] = pkt;
    //     }
    // }
    return enqueue_success;
}

// void response(Packet pkt, int64_t latency) {
//     std::cout << "The request of: " << pkt.addr << std::endl; 
//     std::cout << "The data is: " << pkt.data << std::endl;
//     std::cout << "The latency is: " << latency << std::endl;
// }

void MyWrapper::finish(){
    ramulator2_frontend->finalize();
    ramulator2_memorysystem->finalize();
}

void MyWrapper::frontend_tick(){
    ramulator2_frontend->tick();
}

void MyWrapper::memory_system_tick(){
    ramulator2_memorysystem->tick();
}

MyWrapper::~MyWrapper() {
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
    MyWrapper* dram_new() {
        return new MyWrapper();
    }
    
    // Destructor: delete a MyWrapper instance
    void dram_delete(MyWrapper* obj) {
        delete obj;
    }
    
    // Wrap init method: pass config path as C string
    void dram_init(MyWrapper* obj, const char* config_path) {
        obj->init(std::string(config_path));
    }
    
    // Wrap get_memory_tCK method
    float get_memory_tCK(MyWrapper* obj) {
        return obj->get_memory_tCK();
    }
    
    // Wrap send_request method
    bool send_request(MyWrapper* obj, int64_t addr, bool is_write, void (*callback)(Ramulator::Request*, void*), void* ctx) {
        return obj->send_request(addr, is_write, 
            [callback, ctx](Ramulator::Request& req) {
                callback(&req, ctx);
            });
    }
    
    // Wrap finish method
    void MyWrapper_finish(MyWrapper* obj) {
        obj->finish();
    }
    
    // Wrap tick method
    void frontend_tick(MyWrapper* obj) {
        obj->frontend_tick();
    }

    void memory_system_tick(MyWrapper* obj) {
        obj->memory_system_tick();
    }
    
}