defmodule PixelflutCanvas.CanvasServer do
  @moduledoc false

  use GenServer
  require Logger

  @impl true
  def init(opts) do
    state = init_encoders([])
    state = Keyword.put(state, :canvas, PixelflutCanvas.Canvas.gen_canvas(opts[:width], opts[:height]))

    notify_encoders(state)

    {:ok, state}
  end

  defp init_encoders(state) do
    {:ok, rgb64_pid} = DynamicSupervisor.start_child(PixelflutCanvas.EncoderSupervisor,
      {PixelflutCanvas.EncoderServer, [algorithm: :rgb64]})
    {:ok, rgba64_pid} = DynamicSupervisor.start_child(PixelflutCanvas.EncoderSupervisor,
      {PixelflutCanvas.EncoderServer, [algorithm: :rgba64]})

    state = Keyword.put(state, :encoder_rgb64, rgb64_pid)
    state = Keyword.put(state, :encoder_rgba64, rgba64_pid)
    state
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
      state = Keyword.put(state, :canvas, PixelflutCanvas.Canvas.set_pixel(state[:canvas], x, y, color))
      notify_encoders(state)

      {:noreply, state}
    rescue
       _ in [ArgumentError, FunctionClauseError] -> Logger.warn("Invalid CanvasServer input") && {:noreply, state}
    end
  end

  defp notify_encoders(state) do
    PixelflutCanvas.EncoderClient.encode(state[:encoder_rgb64], state[:canvas].content)
    PixelflutCanvas.EncoderClient.encode(state[:encoder_rgba64], state[:canvas].content)
  end

  @impl GenServer
  def handle_call({:get_pixel, [{:x, x}, {:y, y}]}, _, state) do
    try do
      {:reply, PixelflutCanvas.Canvas.get_pixel(state[:canvas], x, y), state}
    rescue
      ArgumentError -> {:reply, {:error, :argument_error}, state}
      FunctionClauseError -> {:reply, {:error, :argument_error}, state}
    end
  end

  @impl GenServer
  def handle_call({:get_size}, _, state) do
    {:reply, [width: state[:canvas].width, height: state[:canvas].height], state}
  end

  @impl GenServer
  def handle_call({:get_encoded, algorithm}, _, state) do
    try do
      encoder = String.to_existing_atom("encoder_#{algorithm}")
      {:reply, {:ok, PixelflutCanvas.EncoderClient.get_encoded(state[encoder])}, state}
    rescue
       ArgumentError -> {:reply, {:error, :invalid_encoding}, state}
     end
  end
end
