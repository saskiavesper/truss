defmodule Mix.Tasks.Db.MigrateTest do
  use ExUnit.Case
  import ExUnit.CaptureIO

  setup do
    original = Application.get_env(:truss, :infrastructure)

    Application.put_env(:truss, :infrastructure,
      database: %{
        hostname: "localhost",
        port: 5432,
        username: "postgres",
        password: "postgres",
        database: "truss_test"
      }
    )

    Application.put_env(:truss, :postgrex_module, Truss.Test.PostgrexMock)
    Truss.Test.PostgrexMock.reset!()

    File.mkdir_p!("priv/migrations")

    on_exit(fn ->
      Application.put_env(:truss, :infrastructure, original)
      Application.delete_env(:truss, :postgrex_module)
      Truss.Test.PostgrexMock.reset!()
      File.rm_rf!("priv/migrations")
    end)
  end

  test "reports no pending migrations when none exist" do
    output =
      capture_io(fn ->
        Mix.Tasks.Db.Migrate.run([])
      end)

    assert output =~ "No pending migrations"
  end

  test "runs pending migrations" do
    path = Path.join("priv/migrations", "20260101_000000__test.sql")
    File.write!(path, "SELECT 1;")

    output =
      capture_io(fn ->
        Mix.Tasks.Db.Migrate.run([])
      end)

    assert output =~ "Running migration: 20260101_000000__test.sql"
    assert output =~ "OK"
  end

  test "runs migrations in sorted order" do
    paths = [
      "20260101_000000__first.sql",
      "20260102_000000__second.sql",
      "20260103_000000__third.sql"
    ]

    Enum.each(paths, fn name ->
      File.write!(Path.join("priv/migrations", name), "SELECT 1;")
    end)

    output =
      capture_io(fn ->
        Mix.Tasks.Db.Migrate.run([])
      end)

    lines = String.split(output, "\n", trim: true)
    running = Enum.filter(lines, &String.contains?(&1, "Running migration:"))

    assert [first, second, third] = running
    assert first =~ "20260101_000000__first"
    assert second =~ "20260102_000000__second"
    assert third =~ "20260103_000000__third"
  end

  test "skips already-applied migrations" do
    applied_path = Path.join("priv/migrations", "20200101_000000__already_applied.sql")
    new_path = Path.join("priv/migrations", "20260101_000000__new.sql")
    File.write!(applied_path, "SELECT 1;")
    File.write!(new_path, "SELECT 1;")

    Truss.Test.PostgrexMock.seed(["20200101_000000__already_applied"])

    output =
      capture_io(fn ->
        Mix.Tasks.Db.Migrate.run([])
      end)

    refute output =~ "already_applied"
    assert output =~ "Running migration: 20260101_000000__new.sql"
  end
end
