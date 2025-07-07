#ifndef MYWRAPPER_H
#define MYWRAPPER_H

#include "include/config.h"
#include "include/frontend.h"
#include "include/memory_system.h"
#include "include/base.h"
#include "include/request.h"
#include <deque>
#include <unordered_map>

class MyWrapper {

public:
    MyWrapper() = default;
    ~MyWrapper();
    void init(const std::string& config_path);
    float get_memory_tCK() const;
    bool send_request(int64_t addr, bool is_write, std::function<void(Ramulator::Request&)> callback);
    void finish();
    void frontend_tick();
    void memory_system_tick();

    std::string config_path;
    Ramulator::IFrontEnd* ramulator2_frontend;
    Ramulator::IMemorySystem* ramulator2_memorysystem;
};

#endif // MYWRAPPER_H
