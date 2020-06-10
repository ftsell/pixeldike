defmodule PixelflutCanvas.Canvas do
  @moduledoc"""
  A module for transparently managing Pixel data on a canvas.

  It uses erlang :array internally so expect debugging to be a tad bit weird
  """

  alias PixelflutCanvas.Canvas

  defstruct width: -1, height: -1, content: nil

  @doc ~S"""
  Generates a new blank canvas Struct in the specified size.
  """
  @spec gen_canvas(number(), number()) :: %Canvas{}
  def gen_canvas(width, height) do
    %Canvas{
      width: width,
      height: height,
      content: :array.new(width * height, [default: <<0, 0, 0>>, fixed: true])
    }
  end

  @spec index(%Canvas{}, number(), number()) :: number()
  defp index(%Canvas{width: width}, x, y) do
    y * width + x
  end

  @doc"""
  Sets a pixel on the canvas to the specified color.
  The color is simply a 3-byte bitstring which gets saved and returned later; it is not
  interpreted by the canvas in any way.

  Coordinates start at zero and shall not be larger than the canvases width or height.
  """
  @spec set_pixel(%Canvas{}, number(), number(), bitstring()) :: %Canvas{}
  def set_pixel(
        canvas = %Canvas{width: width, height: height, content: canvas_content},
        x, y, color = <<_ :: 8, _ :: 8, _ :: 8>>)
      when x >= 0 and x < width
      when y >= 0 and y < height do
    %Canvas{
      width: width,
      height: height,
      content: :array.set(index(canvas, x, y), color, canvas_content)
    }
  end

  @doc"""
  Returns the pixel color at the specified location
  """
  @spec get_pixel(%Canvas{}, number(), number()) :: bitstring()
  def get_pixel(
        canvas = %Canvas{width: width, height: height, content: canvas_content},
        x, y)
      when x >= 0 and x < width
      when y >= 0 and y < height do
    :array.get(index(canvas, x, y), canvas_content)
  end
end
