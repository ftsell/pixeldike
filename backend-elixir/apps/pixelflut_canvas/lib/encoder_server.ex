defmodule PixelflutCanvas.EncoderServer do
  @moduledoc false

  use GenServer
  require Logger

  @impl GenServer
  def init(opts) do
    Logger.info("Encoder for #{opts[:algorithm]} encoding starting")
    state = opts
    state = Keyword.put(state, :encoded, <<>>)
    state = Keyword.put(state, :task, nil)
    state = Keyword.put(state, :encode_again, false)

    {:ok, state}
  end

  def start_link(opts) do
    GenServer.start_link(__MODULE__, opts, [])
  end

  @impl GenServer
  def handle_call({:get_encoded}, _, state) do
    {:reply, state[:encoded], state}
  end

  @impl GenServer
  def handle_cast({:encode, array}, state) do
    state = case state[:task] do
      nil -> Keyword.put(state, :task, Task.async(__MODULE__, :encode, [state[:algorithm], array]))
      _ -> state
    end

    {:noreply, state}
  end

  @impl GenServer
  def handle_info({:DOWN, _ref, :process, _pid, _reason}, state) do
    # gets called when the encoding process dies
    {:noreply, state}
  end

  @impl GenServer
  def handle_info(msg = {ref, encoded}, state) do
    # gets called with the encoding tasks result
    {
      :noreply,
      if ref == state[:task].ref do
        state = Keyword.put(state, :encoded, encoded)
        state = Keyword.put(state, :task, nil)
        state
      else
        Logger.warn("Unknown message received by EncoderServer: #{msg} with state #{state}")
        state
      end
    }
  end

  def encode(:rgb64, array) do
    Base.encode64(:array.foldl(fn _i, value, akk -> akk <> value end, <<>>, array))
  end

  def encode(:rgba64, array) do
    Base.encode64(:array.foldl(fn _i, value, akk -> akk <> value <> <<0>> end, <<>>, array))
  end
end
