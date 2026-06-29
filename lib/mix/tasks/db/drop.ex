defmodule Mix.Tasks.Db.Drop do
  use Mix.Task

  @shortdoc "Drops the PostgreSQL database"

  def run(_args) do
    Mix.Task.run("app.start")

    config = Truss.Config.Database.fetch!()
    db_name = config.database

    conn_opts =
      Keyword.new(config)
      |> Keyword.put(:database, "postgres")
      |> Keyword.put(:types, Truss.PostgrexTypes)

    {:ok, conn} = Postgrex.start_link(conn_opts)

    Mix.shell().info("Dropping database: #{db_name}")

    Postgrex.query!(
      conn,
      """
      SELECT pg_terminate_backend(pg_stat_activity.pid)
      FROM pg_stat_activity
      WHERE pg_stat_activity.datname = $1
        AND pid <> pg_backend_pid()
      """,
      [db_name]
    )

    Postgrex.query!(conn, "DROP DATABASE IF EXISTS #{db_name}", [])

    Mix.shell().info("Database #{db_name} dropped")
  end
end
