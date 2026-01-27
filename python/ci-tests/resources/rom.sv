// -----------------------------------------------------------------
// Parameterized ROM (Read-Only Memory) with file initialization
// This module defines a ROM whose contents are loaded from an
// external hex file via the $readmemh system task. It is
// parameterized for both address width and data width.
//
// In synthesis, many tools (Vivado/Quartus/Yosys) support this
// initialization method and will infer appropriate memory blocks.
// See references about $readmemh usage for memory initialization.  [oai_citation:1‡Project F](https://projectf.io/posts/initialize-memory-in-verilog/?utm_source=chatgpt.com)
// -----------------------------------------------------------------
module rom #(
    parameter ADDR_WIDTH = 8,   // Width of the address bus -> depth = 2^ADDR_WIDTH
    parameter DATA_WIDTH = 32,  // Width of each memory word
    parameter INIT_FILE  = ""   // Optional hex file for memory initialization
) (
    input  logic [ADDR_WIDTH-1:0] addr, // Read address input
    output logic [DATA_WIDTH-1:0] data   // Read data output
);

    // -----------------------------------------------------------------
    // Memory array declaration (unpacked array).
    // Depth = 2^ADDR_WIDTH, each location is DATA_WIDTH bits wide.
    // -----------------------------------------------------------------
    logic [DATA_WIDTH-1:0] mem [0:(1<<ADDR_WIDTH)-1];

    // -----------------------------------------------------------------
    // Load contents from INIT_FILE using $readmemh in the initial block.
    // $readmemh loads hex values from the file into the memory array.
    // The file should have hex values (whitespace separated, per line).
    // This is a common technique to initialize ROM contents.  [oai_citation:2‡Stack Overflow](https://stackoverflow.com/questions/67701135/how-to-make-a-synthesizable-instruction-memory-in-systemverilog?utm_source=chatgpt.com)
    // -----------------------------------------------------------------
    initial begin
        if (INIT_FILE != "") begin
            $readmemh(INIT_FILE, mem);
        end
    end

    // -----------------------------------------------------------------
    // Combinational read logic: simply index the memory with 'addr'
    // NOTE: In some synthesis tools this may infer combinational ROM,
    // or memory blocks (BRAM/LUTRAM) depending on size and style.
    // -----------------------------------------------------------------
    assign data = mem[addr];

endmodule