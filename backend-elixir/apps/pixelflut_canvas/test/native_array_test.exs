defmodule Elixir.NativeArray.Test do
  use ExUnit.Case, async: true
  use ExUnitProperties

  alias Elixir.NativeArray
  #doctest PixelflutCanvas.NativeArray

  test "creating a native-array works" do
    x = NativeArray.new(10)
    NativeArray.set(x, 0, 5)
  end
end
