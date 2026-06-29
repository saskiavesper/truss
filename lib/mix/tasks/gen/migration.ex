defmodule Mix.Tasks.Gen.Migration do
  use Mix.Task

  @moduledoc """
  Generates a timestamped SQL migration file in `priv/migrations/`.

  ## Usage

      mix gen.migration create_workspaces

  Produces `priv/migrations/20260126_163000__create_workspaces.sql`.
  """

  @shortdoc "Generates a SQL migration file in priv/migrations"

  def run([name | _]) do
    timestamp =
      NaiveDateTime.utc_now()
      |> NaiveDateTime.to_string()
      |> String.slice(0..18)
      |> String.replace(~r/[^0-9]/, "")

    filename = "#{timestamp}__#{name}.sql"
    path = Path.join(["priv", "migrations", filename])

    File.mkdir_p!("priv/migrations")

    File.write!(path, """
    -- #{filename}
    -- Migration generated for #{name}

    """)

    Mix.shell().info("Created migration: #{path}")
  end

  def run([]) do
    Mix.shell().error("Usage: mix gen.migration <name>")
  end
end
