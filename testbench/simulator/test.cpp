#include "MyWrapper.h"
#include <iostream>
#include <vector>
#include <algorithm>
#include <random> 

// This file should output the same result as the assassyn/python/unit-tests/test_dram.py
int main() {
    MyWrapper wrapper;

    std::string config_path = "../../configs/example_config.yaml";;  // Adjust to your config path
    wrapper.init(config_path);
    bool is_write = false;
    int v = 0; //counter
    std::cout << std::boolalpha;
    for (int i = 0; i < 200; i++) {   
        int plused = v + 1;
        int we = v & 1;
        int re = !we;
        int64_t raddr = v & 0xFF;
        int64_t waddr = plused & 0xFF;
        int64_t addr = is_write ? waddr : raddr; 
        bool ok = wrapper.send_request(addr, is_write, [i](Ramulator::Request& req) {
            std::cout << "Cycle " << i + 3 + (req.depart - req.arrive) << ": Request completed: " << req.addr << " the data is: " << (req.addr - 1) << std::endl;
        });
        if (is_write) {      
            std::cout << "Cycle " << i + 2 << ": Write request sent for address " << addr << ", success or not (true or false)" << ok << std::endl;          
        }
        is_write = !is_write;
        wrapper.frontend_tick();
        wrapper.memory_system_tick();
        v = plused;
    }

    wrapper.finish();
    return 0;
}