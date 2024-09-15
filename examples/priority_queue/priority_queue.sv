`timescale 1ns/1ps

module pq #(
    parameter L = 3, // layers of the tree
    parameter W = 32 // value width
) (
    input logic clk,
    input logic rst_n,

    input  logic           enq_valid,
    input  logic [W - 1:0] enq_value,
    output logic           enq_ready,

    input  logic           deq_req,
    output logic [W - 1:0] deq_value,
    output logic           deq_valid
);

typedef enum logic [1:0] {
    OP_NOP = 2'b00,
    OP_ENQ = 2'b01,
    OP_DEQ = 2'b10
    // OP_EDQ = 2'd3
} op_t;

generate
    for (genvar l = 0; l < L; l++) begin: pq_tokens
        op_t operation;
        logic [l == 0 ? 0 : (l - 1):0] position;
        logic [W - 1:0] value;
    end
    for (genvar l = 0; l < L; l++) begin: pq_tree
        localparam OW = L - l;
        logic [1 + OW + W - 1:0] nodes[0:(1 << l) - 1];
    end
    for (genvar l = 0; l < L; l++) begin: pq_level
        localparam OW = L - l;
        logic [1 + OW + W - 1:0] target_node;
        if (l == 0) begin
            assign target_node = pq_tree[l].nodes[pq_tokens[l].position];
        end else begin
            assign target_node = pq_tokens[l - 1].operation == OP_NOP ?
                pq_tree[l].nodes[pq_tokens[l].position] :
                pq_tree[l].nodes[{pq_tokens[l - 1].position, 1'b0}];
        end

        logic target_node_active;
        assign target_node_active = target_node[1 + OW + W - 1];
        logic [OW - 1:0] target_node_occupied;
        assign target_node_occupied = target_node[OW + W - 1:W];
        logic [W - 1:0] target_node_value;
        assign target_node_value = target_node[W - 1:0];

        always_ff @(posedge clk or negedge rst_n) begin
            if (!rst_n) begin
                pq_tree[l].nodes <= '{default: '0};
                pq_tokens[l].operation <= OP_NOP;
            end else begin
                unique case (pq_tokens[l].operation)
                    OP_ENQ: begin
                        if (!target_node_active || pq_tokens[l].value > target_node_value) begin
                            pq_tree[l].nodes[pq_tokens[l].position] <= {1'b1, target_node_occupied + 1'b1, pq_tokens[l].value};
                        end else begin
                            pq_tree[l].nodes[pq_tokens[l].position] <= {1'b1, target_node_occupied + 1'b1, target_node_value};
                        end
                        pq_tokens[l].operation <= OP_NOP;
                    end
                    OP_DEQ: begin
                        if (l == L - 1) begin // leaves
                            pq_tree[l].nodes[pq_tokens[l].position] <= '0;
                        end else begin
                            `define left_child pq_tree[l == L - 1 ? l : (l + 1)].nodes[{pq_tokens[l].position, 1'b0}]
                            `define right_child pq_tree[l == L - 1 ? l : (l + 1)].nodes[{pq_tokens[l].position, 1'b1}]
                            unique case ({`left_child[1 + OW - 1 + W - 1], `right_child[1 + OW - 1 + W - 1]})
                                2'b00: begin // both child inactive
                                    pq_tree[l].nodes[pq_tokens[l].position] <= {1'b0, target_node_occupied - 1'b1, {W{1'b0}}};
                                end
                                2'b01: begin // right child active
                                    pq_tree[l].nodes[pq_tokens[l].position] <= {1'b1, target_node_occupied - 1'b1, `right_child[W - 1:0]};
                                end
                                2'b10: begin // left child active
                                    pq_tree[l].nodes[pq_tokens[l].position] <= {1'b1, target_node_occupied - 1'b1, `left_child[W - 1:0]};
                                end
                                2'b11: begin // both child active
                                    if (`left_child[W - 1:0] > `right_child[W - 1:0]) begin
                                        pq_tree[l].nodes[pq_tokens[l].position] <= {1'b1, target_node_occupied - 1'b1, `left_child[W - 1:0]};
                                    end else begin
                                        pq_tree[l].nodes[pq_tokens[l].position] <= {1'b1, target_node_occupied - 1'b1, `right_child[W - 1:0]};
                                    end
                                end
                            endcase
                            `undef left_child
                            `undef right_child
                        end
                        pq_tokens[l].operation <= OP_NOP;
                    end
                    OP_NOP: begin
                        if (l == 0) begin
                            pq_tokens[l].operation <= {deq_req, enq_valid};
                            pq_tokens[l].position <= '0;
                            pq_tokens[l].value <= enq_value;
                        end else begin
                            `define safe_l_minus_1 l == 0 ? 0 : (l - 1)
                            unique case (pq_tokens[`safe_l_minus_1].operation)
                                OP_ENQ: begin
                                    `define prev_target_node pq_tree[`safe_l_minus_1].nodes[pq_tokens[`safe_l_minus_1].position]
                                    if (`prev_target_node[1 + OW + 1 + W - 1]) begin // prev level target active
                                        pq_tokens[l].operation <= OP_ENQ;
                                        pq_tokens[l].value <= pq_tokens[`safe_l_minus_1].value > `prev_target_node[W - 1:0] ? `prev_target_node[W - 1:0] : pq_tokens[`safe_l_minus_1].value;
                                        pq_tokens[l].position <= !(&target_node_occupied) ? (l > 1 ? {pq_tokens[`safe_l_minus_1].position, 1'b0} : 1'b0) : (l > 1 ? {pq_tokens[`safe_l_minus_1].position, 1'b1} : 1'b1);
                                    end
                                    `undef prev_target_node
                                end
                                OP_DEQ: begin
                                    `define left_child pq_tree[l].nodes[{pq_tokens[`safe_l_minus_1].position, 1'b0}]
                                    `define right_child pq_tree[l].nodes[{pq_tokens[`safe_l_minus_1].position, 1'b1}]
                                    unique case ({`left_child[1 + OW + W - 1], `right_child[1 + OW + W - 1]})
                                        2'b00: begin // both child inactive
                                            pq_tokens[l].operation <= OP_NOP;
                                            pq_tokens[l].position <= '0;
                                        end
                                        2'b01: begin // right child active
                                            pq_tokens[l].operation <= OP_DEQ;
                                            pq_tokens[l].position <= l > 1 ? {pq_tokens[`safe_l_minus_1].position, 1'b1} : 1'b1;
                                        end
                                        2'b10: begin // left child active
                                            pq_tokens[l].operation <= OP_DEQ;
                                            pq_tokens[l].position <= l > 1 ? {pq_tokens[`safe_l_minus_1].position, 1'b0} : 1'b0;
                                        end
                                        2'b11: begin // both child active
                                            pq_tokens[l].operation <= OP_DEQ;
                                            if (`left_child[W - 1:0] > `right_child[W - 1:0]) begin
                                                pq_tokens[l].position <= l > 1 ? {pq_tokens[`safe_l_minus_1].position, 1'b0} : 1'b0;
                                            end else begin
                                                pq_tokens[l].position <= l > 1 ? {pq_tokens[`safe_l_minus_1].position, 1'b1} : 1'b1;
                                            end
                                        end
                                    endcase
                                    `undef left_child
                                    `undef right_child
                                    pq_tokens[l].value <= '0;
                                end
                                OP_NOP: ;
                            endcase
                            `undef safe_l_minus_1
                        end
                    end
                endcase
            end
        end
    end
    assign deq_valid = pq_tokens[0].operation == OP_DEQ && pq_tree[0].nodes[0][1 + L + W - 1];
    assign deq_value = pq_tree[0].nodes[0][W - 1:0];
    assign enq_ready = !(&pq_tree[0].nodes[0][L + W - 1:W]) && pq_tokens[0].operation == OP_NOP;
endgenerate

endmodule

module tb();

logic clk;
logic rst_n;

logic        enq_valid;
logic [31:0] enq_value;
logic        enq_ready;

logic        deq_req;
logic [31:0] deq_value;
logic        deq_valid;

pq #(.L(3), .W(32)) pq_i (
    .clk(clk),
    .rst_n(rst_n),

    .enq_valid(enq_valid),
    .enq_value(enq_value),
    .enq_ready(enq_ready),

    .deq_req(deq_req),
    .deq_value(deq_value),
    .deq_valid(deq_valid)
);

always #0.5 clk <= !clk;

initial begin
    $fsdbDumpfile("wave.fsdb");
    $fsdbDumpvars();
    $fsdbDumpMDA();
    clk = '0;
    enq_valid = '0;
    enq_value = '0;
    deq_req = '0;
    rst_n = '0;
    #5;
    @(negedge clk);
    rst_n = '1;
    #5;
    enq_valid = '1;
    enq_value = 32'h4;
    @(posedge clk iff enq_ready);
    enq_value = 32'h44;
    @(posedge clk iff enq_ready);
    enq_value = 32'h444;
    @(posedge clk iff enq_ready);
    enq_value = 32'h4444;
    @(posedge clk iff enq_ready);
    enq_value = 32'h44444;
    @(posedge clk iff enq_ready);
    enq_value = 32'h444444;
    @(posedge clk iff enq_ready);
    enq_value = 32'h4444444;
    @(posedge clk iff enq_ready);
    enq_valid = '0;
    enq_value = '0;
    #10;
    deq_req = '1;
    @(posedge clk iff deq_valid);
    $display("%t DEQ: %h", $time, deq_value);
    @(posedge clk iff deq_valid);
    $display("%t DEQ: %h", $time, deq_value);
    @(posedge clk iff deq_valid);
    $display("%t DEQ: %h", $time, deq_value);
    @(posedge clk iff deq_valid);
    $display("%t DEQ: %h", $time, deq_value);
    @(posedge clk iff deq_valid);
    $display("%t DEQ: %h", $time, deq_value);
    @(posedge clk iff deq_valid);
    $display("%t DEQ: %h", $time, deq_value);
    @(posedge clk iff deq_valid);
    $display("%t DEQ: %h", $time, deq_value);
    deq_req = '0;
    #100;
    $finish();
end

endmodule