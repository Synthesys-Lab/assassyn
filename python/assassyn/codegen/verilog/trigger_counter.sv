// The purpose of a FIFO is different from the purpose of a counter.
// A FIFO can only be pushed or popped once per cycle, while a counter
// can increase multiple event counters in a single cycle.
//
// This is tyically useful for an arbiter, where an arbiter can have multiple
// instances pushed to it in a single same cycle, but it can only pop one
// instance per cycle.
module trigger_counter #(
    parameter WIDTH = 8
    // parameter NAME = "fifo" // TODO(@were): Open this later
) (
  input logic clk,
  input logic rst_n,

  input  logic [WIDTH-1:0] delta,
  output logic             delta_ready,

  input  logic             pop_ready,
  output logic             pop_valid
);

logic [WIDTH-1:0] count;
logic             pop_fire;
logic [WIDTH:0]   sum;
logic [WIDTH:0]   next;
logic [WIDTH-1:0] depth;

assign pop_valid = (count != '0);
assign pop_fire = pop_ready && pop_valid;

// The trigger counter is used as a "credit counter" for FIFO-like pipelines where
// the enqueue depth is a power-of-two FIFO. For DEPTH_LOG2=N, the FIFO depth is
// (1<<N) and the counter width is sized as (N+1) bits so it can represent the
// full state (count == depth).
//
// Recover the corresponding FIFO depth from WIDTH: depth = 1 << (WIDTH-1).
assign depth = {1'b1, {WIDTH-1{1'b0}}};

// Ready unless full (count == depth), or if we're going to pop this cycle
// (freeing one slot).
assign delta_ready = (count != depth) || pop_fire;

assign sum = {1'b0, count} + {1'b0, delta};
assign next = sum - {{WIDTH{1'b0}}, pop_fire};

always_ff @(posedge clk or negedge rst_n) begin
  if (!rst_n) begin
    count <= '0;
  end else begin
    count <= next[WIDTH-1:0];
  end
end

endmodule
