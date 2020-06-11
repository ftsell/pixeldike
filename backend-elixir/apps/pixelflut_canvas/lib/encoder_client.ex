defmodule PixelflutCanvas.EncoderClient do
  @moduledoc false

  @doc """
  Instructs the encoder server to start encoding the given array
  """
  def encode(server, array) do
    GenServer.cast(server, {:encode, array})
  end

  @doc """
  Retrieves the current encoded representation from the encoder server
  """
  def get_encoded(server) do
    GenServer.call(server, {:get_encoded})
  end
end