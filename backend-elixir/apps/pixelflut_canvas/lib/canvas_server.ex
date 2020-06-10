defmodule PixelflutCanvas.CanvasServer do
  @moduledoc false

  use GenServer

  @impl true
  def init(opts) do
    state = PixelflutCanvas.Canvas.gen_canvas(opts[:width], opts[:height])
    {:ok, state}
  end

  def start_link(opts) do
    init_opts = [
      width: Application.get_env(:pixelflut_canvas, :width),
      height: Application.get_env(:pixelflut_canvas, :height),
    ]
    GenServer.start_link(__MODULE__, init_opts, opts)
  end

  @impl GenServer
  def handle_cast({:set_pixel, [{:x, x}, {:y, y}, {:color, color}]}, state) do
    try do
      {:noreply, PixelflutCanvas.Canvas.set_pixel(state, x, y, color)}
    rescue
       ArgumentError -> {:noreply, state}
    end
  end

  @impl GenServer
  def handle_call({:get_pixel, [{:x, x}, {:y, y}]}, _, state) do
    try do
      {:reply, PixelflutCanvas.Canvas.get_pixel(state, x, y), state}
    rescue
      ArgumentError -> {:reply, {:error, :argument_error}, state}
    end
  end

  @impl GenServer
  def handle_call({:get_size}, _, state) do
    {:reply, [width: state.width, height: state.height], state}
  end

  @impl GenServer
  def handle_call({:get_raw_array}, _, state) do
    {:reply, state.content, state}
  end
end
