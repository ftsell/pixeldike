defmodule PixelflutServer.Net.TcpServer do
  @moduledoc false

  # this is a task but it should always get restarted when it fails
  use Task, restart: :permanent
  require Logger

  def start_link(opts) do
    port = Application.get_env(:pixelflut_server, :tcp_port)

    Task.start_link(__MODULE__, :run, [opts[:task_supervisor], port])
    #Task.start_link(fn -> PixelflutServer.Net.TcpServer.run(opts[:task_supervisor], port) end)
  end

  def run(task_supervisor, port) do
    #{:ok, socket} = :gen_tcp.listen(port, [:binary, packet: :line, active: false, reuseaddr: true])
    Logger.info("Starting TCP Listener on port #{port}")
    with {:ok, socket} <- :gen_tcp.listen(port, [:binary, packet: :line, active: false, reuseaddr: true]) do
      loop_acceptor(task_supervisor, socket)
    end
  end

  defp loop_acceptor(task_supervisor, socket) do
    # accept new client connections and handle them in a different process.
    # that process is then made owner of the socket
    {:ok, client} = :gen_tcp.accept(socket)
    {:ok, pid} = Task.Supervisor.start_child(task_supervisor,
      fn -> serve(task_supervisor, client) end)
    :ok = :gen_tcp.controlling_process(client, pid)

    loop_acceptor(task_supervisor, socket)
  end

  defp serve(task_supervisor, socket) do
    socket
    |> read_line()
    |> PixelflutServer.CommandHandler.parse_and_handle()
    |> write_line(socket)

    serve(task_supervisor, socket)
  end

  defp read_line(socket) do
    case :gen_tcp.recv(socket, 0) do
      {:ok, data} -> data
      {:error, e} -> exit({:shutdown, e})
    end
  end

  defp write_line(line, socket) do
    if line != :empty do
      :gen_tcp.send(socket, "#{line}\n")
    end
  end
end
