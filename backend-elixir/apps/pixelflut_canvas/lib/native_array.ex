defmodule Elixir.NativeArray do
  @moduledoc false

  use Rustler, otp_app: :pixelflut_canvas, crate: :nativearray

  def new(_length), do: :erlang.nif_error(:nif_not_loaded)

  def set(_array, _position, _value), do: :erlang.nif_error(:nif_not_loaded)
end
