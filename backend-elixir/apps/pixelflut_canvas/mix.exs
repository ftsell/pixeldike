defmodule PixelflutCanvas.MixProject do
  use Mix.Project

  def project do
    [
      app: :pixelflut_canvas,
      version: "0.1.0",
      elixir: "~> 1.7",
      build_path: "../../_build",
      config_path: "../../config/config.exs",
      deps_path: "../../deps",
      lockfile: "../../mix.lock",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      compilers: [:rustler] ++ Mix.compilers(),
      rustler_crates: rustler_crates(),
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger],
      mod: {PixelflutCanvas.Application, []}
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      # {:dep_from_hexpm, "~> 0.3.0"},
      # {:dep_from_git, git: "https://github.com/elixir-lang/my_dep.git", tag: "0.1.0"},
      # {:sibling_app_in_umbrella, in_umbrella: true},
      {:stream_data, "~> 0.5.0"},
      {:rustler, "~> 0.21"},
    ]
  end

  defp rustler_crates do
    [
      nativearray: [
        mode: (if Mix.env() == :prod, do: :release, else: :debug)
      ]
    ]
  end
end
