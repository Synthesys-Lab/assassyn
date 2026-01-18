
module fifo #(
    parameter WIDTH = 8,
    parameter DEPTH_LOG2 = 2 // Special case when DEPTH_LOG2 = 0, single element FIFO
    // parameter NAME = "fifo" // TODO(@were): Open this later
) (
    input  logic               clk,
    input  logic               rst_n,

    input  logic               push_valid,
    input  logic [WIDTH - 1:0] push_data,
    output logic               push_ready,

    output logic               pop_valid,
    output logic [WIDTH - 1:0] pop_data,
    input  logic               pop_ready
);

generate
    if (DEPTH_LOG2 == 0) begin : single_element_fifo
        logic              full;
        logic [WIDTH-1:0]  data;
        logic              push_fire;
        logic              pop_fire;

        assign pop_valid = full;
        assign pop_data = data;
        assign pop_fire = pop_ready && pop_valid;

        // Allow push when empty, or when simultaneously popping the only entry.
        assign push_ready = ~full || pop_fire;
        assign push_fire = push_valid && push_ready;

        always_ff @(posedge clk or negedge rst_n) begin
            if (!rst_n) begin
                full <= 1'b0;
                data <= 'x;
            end else begin
                unique case ({push_fire, pop_fire})
                    2'b10: begin
                        data <= push_data;
                        full <= 1'b1;
                    end
                    2'b01: begin
                        full <= 1'b0;
                    end
                    2'b11: begin
                        // Replace the popped element.
                        data <= push_data;
                        full <= 1'b1;
                    end
                    default: begin
                    end
                endcase
            end
        end

    end else begin : multi_element_fifo
        localparam int FIFO_SIZE = (1 << DEPTH_LOG2);
        localparam logic [DEPTH_LOG2:0] FIFO_SIZE_L = FIFO_SIZE[DEPTH_LOG2:0];

        logic [DEPTH_LOG2-1:0] front;
        logic [DEPTH_LOG2-1:0] back;
        logic [DEPTH_LOG2:0]   count;
        logic [WIDTH-1:0]      q[0:FIFO_SIZE-1];

        logic push_fire;
        logic pop_fire;

        assign pop_valid = (count != 0);
        assign pop_data = pop_valid ? q[front] : 'x;
        assign pop_fire = pop_ready && pop_valid;

        // Allow push when not full, or when simultaneously popping (freeing space).
        assign push_ready = (count < FIFO_SIZE_L) || pop_fire;
        assign push_fire = push_valid && push_ready;

        always_ff @(posedge clk or negedge rst_n) begin
            if (!rst_n) begin
                front <= '0;
                back <= '0;
                count <= '0;
            end else begin
                unique case ({push_fire, pop_fire})
                    2'b10: begin
                        q[back] <= push_data;
                        back <= back + 1'b1;
                        count <= count + 1'b1;
                    end
                    2'b01: begin
                        front <= front + 1'b1;
                        count <= count - 1'b1;
                    end
                    2'b11: begin
                        q[back] <= push_data;
                        back <= back + 1'b1;
                        front <= front + 1'b1;
                        // count unchanged
                    end
                    default: begin
                    end
                endcase
            end
        end
    end
endgenerate

endmodule
