defmodule PixelflutCanvas.CanvasClient do
  @moduledoc false

  @doc """
  Sets a pixel on the canvas to the specified color.
  The color is simply a 3-byte bitstring which gets saved and returned later; it is not
  interpreted by the CanvasServer in any way.

  Coordinates start at zero and shall not be larger than the canvases width or height.
  """
  @spec set_pixel(pid(), number(), number(), bitstring()) :: :ok
  def set_pixel(server \\ PixelflutCanvas.CanvasServer, x, y, color) do
    GenServer.cast(server, {:set_pixel, [x: x, y: y, color: color]})
  end

  @doc """
  Returns the pixel color at the specified location
  """
  @spec get_pixel(pid(), number(), number()) :: {:ok, bitstring()} | {:error, :invalid_coordinates}
  def get_pixel(server \\ PixelflutCanvas.CanvasServer, x, y) do
    GenServer.call(server, {:get_pixel, [x: x, y: y]})
  end

  @doc"""
  Returns the canvases size as keyword list with :width and :height keys
  """
  @spec get_size(pid()) :: [width: number(), height: number()]
  def get_size(server \\ PixelflutCanvas.CanvasServer) do
    GenServer.call(server, {:get_size})
  end

  @doc """
  Returns the canvas encoded in the specified format.
  Since encodings are done in background, the result is not always immediate after having called set_pixel.
  """
  @spec get_encoded(pid(), atom()) :: {:ok, bitstring()} | {:error, :invalid_encoding}
  def get_encoded(server \\ PixelflutCanvas.CanvasServer, algorithm) do
    GenServer.call(server, {:get_encoded, algorithm})
  end
end
