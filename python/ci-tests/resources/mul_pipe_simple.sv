// -----------------------------------------------------------------------------
// Simple Pipelined Multiplier (Unsigned), 4-stage
// -----------------------------------------------------------------------------
module mul_pipe_simple #(
  parameter int W = 32
) (
  input  logic           clk,
  input  logic           rst,       // 高有效复位
  input  logic           in_valid,
  input  logic [W-1:0]   a,
  input  logic [W-1:0]   b,
  output logic           out_valid,
  output logic [2*W-1:0] p
);

  localparam int HALF = W/2;

  // 复位极性转换
  logic rst_n;
  assign rst_n = ~rst;

  // -----------------------------
  // valid 管线
  // -----------------------------
  logic v1, v2, v3, v4;

  // -----------------------------
  // Stage 1: 输入拆分寄存
  // -----------------------------
  logic [HALF-1:0] a_lo_s1, a_hi_s1, b_lo_s1, b_hi_s1;

  // -----------------------------
  // Stage 2: 四个部分积
  // -----------------------------
  logic [2*HALF-1:0] pp0_s2, pp1_s2, pp2_s2, pp3_s2;

  // -----------------------------
  // Stage 3: 对齐与部分求和
  // -----------------------------
  logic [2*W-1:0]    sum1_s3, t2_s3;
  logic [2*W-1:0]    t0_s3, t1_s3;
  logic [2*HALF:0]   mid_s3;

  // ===========================================================================
  // Stage 1
  // ===========================================================================
  always_ff @(posedge clk or negedge rst_n) begin
    if (!rst_n) begin
      a_lo_s1 <= '0;  a_hi_s1 <= '0;
      b_lo_s1 <= '0;  b_hi_s1 <= '0;
      v1      <= 1'b0;
    end else begin
      v1      <= in_valid;
      a_lo_s1 <= a[HALF-1:0];
      a_hi_s1 <= a[W-1:HALF];
      b_lo_s1 <= b[HALF-1:0];
      b_hi_s1 <= b[W-1:HALF];
    end
  end

  // ===========================================================================
  // Stage 2
  // ===========================================================================
  always_ff @(posedge clk or negedge rst_n) begin
    if (!rst_n) begin
      pp0_s2 <= '0; pp1_s2 <= '0; pp2_s2 <= '0; pp3_s2 <= '0;
      v2     <= 1'b0;
    end else begin
      v2     <= v1;
      pp0_s2 <= a_lo_s1 * b_lo_s1;  // Alo * Blo
      pp1_s2 <= a_lo_s1 * b_hi_s1;  // Alo * Bhi
      pp2_s2 <= a_hi_s1 * b_lo_s1;  // Ahi * Blo
      pp3_s2 <= a_hi_s1 * b_hi_s1;  // Ahi * Bhi
    end
  end

  // ===========================================================================
  // Stage 3
  // ===========================================================================
  always_ff @(posedge clk or negedge rst_n) begin
    if (!rst_n) begin
      sum1_s3 <= '0;
      t2_s3   <= '0;
      t0_s3   <= '0;
      t1_s3   <= '0;
      mid_s3  <= '0;
      v3      <= 1'b0;
    end else begin
      v3    <= v2;

      // t0 = zero-extend(PP0)
      t0_s3 <= {{(2*W-2*HALF){1'b0}}, pp0_s2};

      // t1 = ((PP1 + PP2) << HALF)
      mid_s3 <= {1'b0, pp1_s2} + {1'b0, pp2_s2};   // 宽度 2*HALF+1
      t1_s3  <= {{(2*W-(2*HALF+1)){1'b0}}, mid_s3} << HALF;

      // sum1 = t0 + t1
      sum1_s3 <= t0_s3 + t1_s3;

      // t2 = (PP3 << (2*HALF))
      t2_s3   <= {pp3_s2, {2*HALF{1'b0}}};
    end
  end

  // ===========================================================================
  // Stage 4
  // ===========================================================================
  always_ff @(posedge clk or negedge rst_n) begin
    if (!rst_n) begin
      p   <= '0;
      v4  <= 1'b0;
    end else begin
      v4  <= v3;
      p   <= sum1_s3 + t2_s3;
    end
  end

  assign out_valid = v4;

endmodule