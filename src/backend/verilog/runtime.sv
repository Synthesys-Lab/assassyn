module fifo #(
    parameter WIDTH = 8,
    parameter DEPTH_LOG2 = 4
    // parameter NAME = "fifo" // TODO(@were): Open this later
) (
  input logic clk,
  input logic rst_n,

  input  logic               push_valid,
  input  logic [WIDTH - 1:0] push_data,
  output logic               push_ready,

  output logic               pop_valid,
  output logic [WIDTH - 1:0] pop_data,
  input  logic               pop_ready
);

logic [DEPTH_LOG2-1:0] front;
logic [DEPTH_LOG2-1:0] back;
logic [DEPTH_LOG2:0] count;
logic [WIDTH - 1:0] q[0:(1<<DEPTH_LOG2)-1];

logic [DEPTH_LOG2:0] new_count;
logic [DEPTH_LOG2-1:0] new_front;
logic temp_pop_valid;

always @(posedge clk or negedge rst_n) begin
  if (!rst_n) begin
    front <= 0;
    back <= 0;
    pop_valid <= 1'b0;
    pop_data <= 'x;
    count <= 0;
    push_ready <= 1'b1;
  end else begin

    // The number of elements in the queue after this cycle.
    assign new_count = count + (push_valid ? 1 : 0) - (pop_ready ? 1 : 0);

    // The new front of the queue after this cycle.
    assign new_front = front + (pop_ready && count != 0 ? 1 : 0);


    if (push_valid && new_count <= (1 << DEPTH_LOG2)) begin
      q[back] <= push_data;
      back <= (back + 1);
    end


    front <= new_front;
    count <= new_count;

    push_ready <= new_count < (1 << DEPTH_LOG2);

    temp_pop_valid = new_count != 0 || push_valid;
    pop_valid <= temp_pop_valid;
    // This is the most tricky part of the code:
    // If new_count is 0, we have noting to pop, so we just give pop_valid a 0,
    // and pop_data a 'x. Otherwise, we have to pop something real from the FIFO.
    // Because the array write uses a non-blocking "<=" operator, the result
    // of array write will not be visible until the next cycle. However, we
    // need this result when new_front == back. This indicates the newly
    // pushed data is also the front of the FIFO. Instead of reading it from
    // the array buffer, we directly forward the push_data to pop_data.
    pop_data <= temp_pop_valid ? (new_front == back && push_valid ? push_data : q[new_front]) : 'x;

  end
end

endmodule

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
logic [WIDTH-1:0] temp;
logic [WIDTH-1:0] new_count;

always @(posedge clk or negedge rst_n) begin
  if (!rst_n) begin
    count <= '0;
  end else begin
    // If pop_ready is high, counter -= 1
    assign temp = count + delta;
    // To avoid overflow minus
    assign new_count = temp >= (pop_ready ? 1 : 0) ? temp - (pop_ready ? 1 : 0) : 0;
    // If the counter is gonna overflow, this counter cannot accept any new
    // deltas.
    delta_ready <= new_count != {WIDTH{1'b1}};
    // Assign the new counter value.
    count <= new_count;
    pop_valid <= (new_count != 0 || delta != 0);
  end
end

endmodule
