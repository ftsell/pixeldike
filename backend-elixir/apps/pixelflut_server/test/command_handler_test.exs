defmodule PixelflutServer.Test.CommandHandlerTest do
  use ExUnit.Case, async: true
  use ExUnitProperties

  alias PixelflutServer.CommandHandler
  doctest CommandHandler

  setup do
    #canvas_server = start_supervised!(PixelflutCanvas.CanvasServer)
    #%{canvas_server: canvas_server}
    %{}
  end
end
