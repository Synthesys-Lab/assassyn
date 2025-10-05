#ifndef CWRAPPER_H
#define CWRAPPER_H

#include "base/base.h"
#include "base/config.h"
#include "base/request.h"
#include "frontend/frontend.h"
#include "memory_system/memory_system.h"
#include <deque>
#include <unordered_map>

class CWrapper {

public:
  CWrapper() = default;
  ~CWrapper();
  void init(const std::string &config_path);
  float get_memory_tCK() const;
  bool send_request(int64_t addr, bool is_write,
                    std::function<void(Ramulator::Request &)> callback);
  void finish();
  void frontend_tick();
  void memory_system_tick();

  std::string config_path;
  Ramulator::IFrontEnd *ramulator2_frontend;
  Ramulator::IMemorySystem *ramulator2_memorysystem;
};

#endif // CWRAPPER_H
