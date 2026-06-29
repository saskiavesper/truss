defmodule Mix.Tasks.Db.Migrate do
  use Mix.Task

  @shortdoc "Runs all pending SQL migrations in priv/migrations"

  def run(_args) do
    Mix.Task.run("app.start")

    conn_opts = db_connection_opts()

    {:ok, conn} = postgrex().start_link(conn_opts)

    create_schema_migrations_table(conn)

    applied = applied_migrations(conn)
    migrations = pending_migrations(applied)

    if migrations == [] do
      Mix.shell().info("No pending migrations")
    else
      run_migrations(conn, migrations)
    end
  end

  defp create_schema_migrations_table(conn) do
    postgrex().query!(
      conn,
      """
      CREATE TABLE IF NOT EXISTS schema_migrations (
        version VARCHAR(255) PRIMARY KEY,
        inserted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      )
      """,
      []
    )
  end

  defp applied_migrations(conn) do
    conn
    |> postgrex().query!("SELECT version FROM schema_migrations ORDER BY version", [])
    |> Map.get(:rows)
    |> List.flatten()
    |> MapSet.new()
  end

  defp pending_migrations(applied) do
    "priv/migrations"
    |> Path.join("*.sql")
    |> Path.wildcard()
    |> Enum.sort()
    |> Enum.reject(fn path ->
      version = Path.basename(path, ".sql")
      MapSet.member?(applied, version)
    end)
  end

  defp run_migrations(conn, migrations) do
    Enum.each(migrations, fn path ->
      version = Path.basename(path, ".sql")

      Mix.shell().info("Running migration: #{Path.basename(path)}")

      sql = File.read!(path)

      case postgrex().query(conn, sql, []) do
        {:ok, _} ->
          postgrex().query!(conn, "INSERT INTO schema_migrations (version) VALUES ($1)", [version])

          Mix.shell().info("  OK")

        {:error, reason} ->
          Mix.shell().error("  FAILED: #{Exception.message(reason)}")
          raise reason
      end
    end)
  end

  defp db_connection_opts do
    config = Truss.Config.Database.fetch!()
    Keyword.new(config) ++ [types: Truss.PostgrexTypes]
  end

  defp postgrex, do: Application.get_env(:truss, :postgrex_module, Postgrex)
end
