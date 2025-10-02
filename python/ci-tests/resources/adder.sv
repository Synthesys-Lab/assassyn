// Simple 32-bit adder module
module adder(
    input  logic [31:0] a,
    input  logic [31:0] b,
    output logic [31:0] c
);
    assign c = a + b;
endmodule