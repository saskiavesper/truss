defmodule Truss.MixProject do
  use Mix.Project

  def project do
    [
      app: :truss,
      version: "0.1.0",
      elixir: "~> 1.20",
      start_permanent: Mix.env() == :prod,
      aliases: aliases(),
      deps: deps()
    ]
  end

  def cli do
    [preferred_envs: [coverage: :test]]
  end

  def application do
    [
      mod: {Truss.Application, []},
      extra_applications: [:logger]
    ]
  end

  defp aliases do
    [
      setup: ["deps.get", "local.hex --force", "local.rebar --force"],
      lint: ["format --check-formatted", "credo --strict"],
      coverage: ["test --cover --export-coverage default", "crap"]
    ]
  end

  defp deps do
    [
      {:credo, "~> 1.7", only: [:dev, :test], runtime: false},
      {:dialyxir, "~> 1.4", only: [:dev, :test], runtime: false},
      {:ex_doc, "~> 0.37", only: :dev, runtime: false},
      {:ex_crap, "~> 0.1", only: [:dev, :test], runtime: false},
      {:zoi, "~> 0.18"},
      {:postgrex, "~> 0.20"},
      {:pgvector, "~> 0.4"},
      {:uniq, "~> 0.1"},
      {:phoenix_pubsub, "~> 2.1"}
    ]
  end
end
