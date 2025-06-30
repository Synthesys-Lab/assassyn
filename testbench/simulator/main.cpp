#include "MyWrapper.h"
#include <iostream>
#include <vector>
#include <algorithm> // for std::shuffle
#include <random> 

int main() {
    MyWrapper wrapper;

    std::string config_path = "../../configs/example_config.yaml";;  // Adjust to your config path
    wrapper.init(config_path);

    //std::vector<Packet> write_packets, read_packets;
    // for (int64_t i = 1; i <= 10; ++i) {
    //     Packet pkt;
    //     pkt.is_write = true;
    //     pkt.addr = i;
    //     pkt.data = std::to_string(i + 1); // "2" to "11"

    //     write_packets.push_back(pkt);
    // }
    // for (int64_t i = 1; i <= 10; ++i) {
    //     Packet pkt;
    //     pkt.is_write = false;
    //     pkt.addr = i;
    //     pkt.data = ""; // it does not matter

    //     read_packets.push_back(pkt);
    // }
    //finish all writes
    // for (int i = 0;; i++){
    //     if (write_packets.empty()){
    //         //we finished writeing all data.
    //         break;
    //     }
    //     Packet pkt = write_packets.back();
    //     write_packets.pop_back();
    //     bool success;
    //     success = wrapper.send_request(pkt);
    //     if (!success){
    //         write_packets.push_back(pkt);
    //     }
    //     wrapper.tick();
    // }
    // do all reads.
    // for (int i = 0; i < 1000; i++){
    //     if (!read_packets.empty()){
    //         Packet pkt = read_packets.back();
    //         read_packets.pop_back();
    //         bool success;
    //         success = wrapper.send_request(pkt);
    //         if (!success){
    //             read_packets.push_back(pkt);
    //         }
    //         wrapper.tick();
    //     } else {
    //         //finish all reads, but wait for the response
    //         wrapper.tick();
    //     }
    // }

    // for (int i=0;i<1000;i++) { // 1M cycles
    //         bool ok = wrapper.send_request(addr, 1);
    //         if (ok) {
    //             std::cout << "The cycle is: " << i << std::endl;
    //             enq_count ++;
    //             addr += offset;
    //         }
    //     wrapper.tick();
    // }

    std::vector<int64_t> stream_read_addrs, random_read_addrs;
    // stream_read_addrs.reserve(1000); // Reserve space to avoid reallocations
    // random_read_addrs.reserve(1000);

    // for (int64_t i = 1; i <= 1000; ++i) {
    //     stream_read_addrs.push_back(i);
    //     random_read_addrs.push_back(i);
    // }
    // std::random_device rd;
    // std::mt19937 gen(rd());
    // std::shuffle(random_read_addrs.begin(), random_read_addrs.end(), gen);

    // print_vector(stream_read_addrs);
    // print_vector(random_read_addrs);

    //stream read the data
    // for (int i=0;i<1000000;i++) { // 1M cycles
    //     if (!stream_read_addrs.empty()) {
    //         int64_t addr = stream_read_addrs.front();
    //         stream_read_addrs.erase(stream_read_addrs.begin());
    //         bool ok = wrapper.send_request(addr, 0);
    //             if (!ok) {
    //                 stream_read_addrs.insert(stream_read_addrs.begin(), addr);
    //             }
    //     }
    //     wrapper.frontend_tick();
    //     wrapper.memory_system_tick();
    // }

    //random read the data
    // auto callback = [](ramulator::Request& req){
    //     std::cout << "Request completed: " << req.addr << ", Type: " << req.type_id << std::endl;
    // };
    // for (int i=0;i<1000000;i++) { // 1M cycles
    //     if (!random_read_addrs.empty()) {
    //         int64_t addr = random_read_addrs.front();
    //         random_read_addrs.erase(random_read_addrs.begin());
    //         bool ok = wrapper.send_request(addr, 0, callback);
    //             if (!ok) {
    //                 random_read_addrs.insert(random_read_addrs.begin(), addr);
    //             }
    //     }
    //     wrapper.frontend_tick();
    //     wrapper.memory_system_tick();
    // }
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