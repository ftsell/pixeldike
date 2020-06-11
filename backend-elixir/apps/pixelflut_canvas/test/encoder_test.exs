defmodule PixelflutCanvas.Test.EncoderTest do
  use ExUnit.Case, async: true
  use ExUnitProperties

  alias PixelflutCanvas.EncoderClient, as: Client
  doctest Client

  test "encoding an array into rgb64", _ do
    # setup
    encoder_server = start_supervised!({PixelflutCanvas.EncoderServer, algorithm: :rgb64})
    array = :array.new(9, [default: <<0, 0, 0>>, fixed: true])

    # execution
    Client.encode(encoder_server, array)
    Process.sleep(100)
    encoded = Client.get_encoded(encoder_server)

    # verification
    assert encoded == "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
  end

  test "encoding an array into rgba64", _ do
    # setup
    encoder_server = start_supervised!({PixelflutCanvas.EncoderServer, algorithm: :rgba64})
    array = :array.new(9, [default: <<0, 0, 0>>, fixed: true])

    # execution
    Client.encode(encoder_server, array)
    Process.sleep(100)
    encoded = Client.get_encoded(encoder_server)

    # verification
    assert encoded == "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
  end
end
