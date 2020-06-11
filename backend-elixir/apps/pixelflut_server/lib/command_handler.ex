defmodule PixelflutServer.CommandHandler do
  @moduledoc false

  @doc """
  Parses the input line and returns a response

  ## Examples
  Sending an invalid input returns an error
      iex> PixelflutServer.CommandHandler.parse_and_handle("hello my dear server")
      "invalid_command"

  Setting a pixel returns nothing
      iex> PixelflutCanvas.CanvasServer.start_link([name: PixelflutCanvas.CanvasServer])
      ...> PixelflutServer.CommandHandler.parse_and_handle("PX 0 0 FFFFFF")
      :empty

  Getting the size returns width and height
      iex> PixelflutCanvas.CanvasServer.start_link([name: PixelflutCanvas.CanvasServer])
      ...> PixelflutServer.CommandHandler.parse_and_handle("SIZE")
      "800 600"
  """
  @spec parse_and_handle(String.t()) :: String.t()
  def parse_and_handle(line) do
    case (with {:ok, command} <- parse(line),
               {:ok, command} <- convert_inputs(command),
               {:ok, response} <- handle(command)
          do
            {:ok, response}
          end)
    do
      {:error, e} -> Atom.to_string(e)
      {:ok, response} -> response
    end
  end

  def parse(line) do
    case String.split(String.downcase(line)) do
      ["px", x, y, color] -> {:ok, {:set_pixel, x, y, color}}
      ["size"] -> {:ok, {:get_size}}
      ["state"] -> {:ok, {:get_state, "rgb64"}}
      ["state", alg] -> {:ok, {:get_state, alg}}
      ["help"] -> {:ok, {:help, "help"}}
      ["help", topic] -> {:ok, {:help, topic}}
      _ -> {:error, :invalid_command}
    end
  end

  defp handle({:help, "size"}) do
    {:ok, """
      Syntax: 'SIZE\\n'
      Response: 'SIZE $width $height\\n'

      Returns the current canvas size.
      """}
  end

  defp handle({:help, "px"}) do
    {:ok, """
    Syntax: 'PX $x $y $rgb\\n'
    Response: no response

    Sets the pixel color addressed by the coordinates $x and $y.

    $x	- X position on the canvas counted from the left side.
    $y	- Y position on the canvas counted from the top.
    $rgb	- HEX encoded rgb format without # symbol (000000 - FFFFFF).
    """}
  end

  defp handle({:help, "state"}) do
    {:ok, """
    Syntax: 'STATE $algorithm\\n'
    Response: 'STATE $algorithm $data\\n'

    Retrieves the complete canvas in a special encoding chosen by $algorithm.
    Currently implemented algorithms are:

    rgb64:
    Each pixel is encoded into 3 bytes for the color values red, green and blue.
    These bytes are then simply appended to each other in row-major-order.
    At the end, the bytes are base64 encoded.
    rgba64:
    Each pixel is encoded into 4 bytes for the color values red, green and blue and one always-zero alpha channel.
    These bytes are then simply appended to each other in row-major-order.
    At the end, the bytes are base64 encoded.
    """}
  end

  defp handle({:help, _}) do
    {:ok, """
    pixelflut - a pixel drawing game for programmers inspired by reddits r/place.

    Available subcommands are:
    HELP	- This help message
    SIZE	- Get the current canvas size
    PX		- Get or Set one specific pixel color
    STATE	- Get the whole canvas in a specific binary format

    All commands end with a newline character (\\n) and need to be sent as a UTF-8 encoded string (numbers as well).
    Responses are always newline terminated.
    More help ist available with 'HELP $subcommand'
    """}
  end

  defp handle({:set_pixel, x, y, color}) do
    PixelflutCanvas.CanvasClient.set_pixel(x, y, color)
    {:ok, :empty}
  end

  defp handle({:get_size}) do
    [width: width, height: height] = PixelflutCanvas.CanvasClient.get_size()
    {:ok, "#{width} #{height}"}
  end

  defp handle({:get_state, algorithm}) do
    PixelflutCanvas.CanvasClient.get_encoded(algorithm)
  end

  defp convert_inputs({:set_pixel, x, y, color}) do
    # first we base16 decode the color
    case Base.decode16(color, [case: :lower]) do
      :error -> {:error, :invalid_color_encoding}
      # if that works, we try to convert the coordinates to integers
      {:ok, color} -> with {:ok, x, y} <- (try do
                 {:ok, String.to_integer(x), String.to_integer(y)}
               rescue
                  ArgumentError -> {:error, :invalid_coordinates}
               end) do
                  # finally return command with converted variables
                  {:ok, {:set_pixel, x, y, color}}
                end
    end
  end

  defp convert_inputs(command) do
    {:ok, command}
  end
end
