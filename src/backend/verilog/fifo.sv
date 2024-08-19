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
logic [DEPTH_LOG2-1:0] count;
logic [WIDTH - 1:0] q[0:(1<<DEPTH_LOG2)-1];

logic [DEPTH_LOG2-1:0] new_count;
logic [DEPTH_LOG2-1:0] new_front;

always @(posedge clk or negedge rst_n) begin
  if (!rst_n) begin
    front <= 0;
    back <= 0;
    pop_valid <= 1'b0;
    pop_data <= 'x;
    count <= 0;
    push_ready <= 1'b1;
  end else begin

    // $display("%t\t%s: front=%d back=%d count=%d", $time, NAME, front, back, count);

    assign new_count = count + (push_valid ? 1 : 0) - (pop_ready ? 1 : 0);

    assign new_front = front + (pop_ready && count != 0 ? 1 : 0);

    if (push_valid) begin
      // $display("%t\t%s.push %d", $time, NAME, push_data);
      if (new_count <= (1 << DEPTH_LOG2)) begin
        q[back] <= push_data;
        back <= (back + 1);
        push_ready <= (count + 1 - (pop_ready ? 1 : 0)) != (1 << DEPTH_LOG2);
      end else begin
        push_ready <= 1'b0;
      end
    end

    // if (pop_ready && new_count != 0) begin
    //   $display("%t\t%s.pop %d", $time, NAME, q[new_front]);
    // end

    front <= new_front;
    count <= new_count;
    pop_valid <= new_count != 0;
    // This is the most tricky part of the code:
    // If new_count is 0, we have noting to pop, so we just give pop_valid a 0,
    // and pop_data a 'x. Otherwise, we have to pop something real from the FIFO.
    // Because the array write uses a non-blocking "<=" operator, the result
    // of array write will not be visible until the next cycle. However, we
    // need this result when new_front == back. This indicates the newly
    // pushed data is also the front of the FIFO. Instead of reading it from
    // the array buffer, we directly forward the push_data to pop_data.
    pop_data <= new_count == 0 ? 'x : (new_front == back && push_valid ? push_data : q[new_front]);

  end
end

endmodule
