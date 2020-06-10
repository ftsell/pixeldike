defmodule PixelflutCanvas.Test.CanvasTest do
  use ExUnit.Case, async: true
  use ExUnitProperties

  alias PixelflutCanvas.Canvas
  doctest Canvas

  setup do
    canvas_server = start_supervised!(PixelflutCanvas.CanvasServer)
    %{canvas_server: canvas_server}
  end

  property "newly generated canvases content length is width * height", _ do
    check all width <- StreamData.positive_integer(),
              height <- StreamData.positive_integer() do
      %Canvas{content: content} = Canvas.gen_canvas(width, height)
      assert :array.size(content) == width * height
    end
  end

  property "newly generated canvases content contains only bitstrings like <<0, 0, 0>>", _ do
    check all width <- StreamData.positive_integer(),
              height <- StreamData.positive_integer() do
      %Canvas{content: content} = Canvas.gen_canvas(width, height)
      assert Enum.all?(:array.to_list(content), fn <<0::8, 0::8, 0::8>> -> true end)
    end
  end

  property "getting and setting arbitrary pixel values works correctly", _ do
    check all r <- StreamData.positive_integer(),
              g <- StreamData.positive_integer(),
              b <- StreamData.positive_integer() do
      c = Canvas.gen_canvas(2, 1)
      c = Canvas.set_pixel(c, 0, 0, <<r, g, b>>)
      assert Canvas.get_pixel(c, 0, 0) == <<r, g, b>>, "get_pixel did not return correct value"
    end
  end

  test "pixel values persist after sending invalid input to CanvasServer",
       %{canvas_server: canvas_server} do
    PixelflutCanvas.CanvasClient.set_pixel(canvas_server, 0, 0, <<255, 255, 0>>)
    PixelflutCanvas.CanvasClient.set_pixel(canvas_server, 0, 1, <<255, 0, 255>>)
    PixelflutCanvas.CanvasClient.set_pixel(canvas_server, 1, 0, <<255, 255, 255>>)
    PixelflutCanvas.CanvasClient.set_pixel(canvas_server, 1, 1, <<0, 255, 255>>)

    PixelflutCanvas.CanvasClient.set_pixel(canvas_server, -1, 0, <<100, 100, 100>>)

    assert PixelflutCanvas.CanvasClient.get_pixel(canvas_server, 0, 0) == <<255, 255, 0>>
    assert PixelflutCanvas.CanvasClient.get_pixel(canvas_server, 0, 1) == <<255, 0, 255>>
    assert PixelflutCanvas.CanvasClient.get_pixel(canvas_server, 1, 0) == <<255, 255, 255>>
    assert PixelflutCanvas.CanvasClient.get_pixel(canvas_server, 1, 1) == <<0, 255, 255>>
  end
end
