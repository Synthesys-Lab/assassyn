#include "MyWrapper.h"
#include <iostream>
#include <vector>
#include <algorithm> // for std::shuffle
#include <random> 

//This file is just for test
int main() {
    MyWrapper wrapper;

    std::string config_path = "../../configs/example_config.yaml";;  // Adjust to your config path
    wrapper.init(config_path);
    std::vector<int64_t> stream_read_addrs, random_read_addrs;

    
    for (int i = 0; i < 1000000; i++) { // 1M cycles
        int64_t addr = i % 1000 + 1; // Addresses from 1 to 1000
        bool is_write = false; // Alternate between read and write
        bool ok = wrapper.send_request(addr, is_write, [](Ramulator::Request& req) {
            std::cout << "Request completed: " << req.addr << std::endl;
        });
        wrapper.frontend_tick();
        wrapper.memory_system_tick();
    }

    wrapper.finish();
    return 0;
}