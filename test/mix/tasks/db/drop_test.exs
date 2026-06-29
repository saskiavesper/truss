defmodule Mix.Tasks.Db.DropTest do
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

    on_exit(fn ->
      Application.put_env(:truss, :infrastructure, original)
      Application.delete_env(:truss, :postgrex_module)
      Truss.Test.PostgrexMock.reset!()
    end)
  end

  test "reports which database is being dropped" do
    output =
      capture_io(fn ->
        Mix.Tasks.Db.Drop.run([])
      end)

    assert output =~ "Dropping database: truss_test"
    assert output =~ "Database truss_test dropped"
  end

  test "uses different database name from config" do
    Application.put_env(:truss, :infrastructure,
      database: %{
        hostname: "other-host",
        port: 9999,
        username: "admin",
        password: "secret",
        database: "my_app"
      }
    )

    output =
      capture_io(fn ->
        Mix.Tasks.Db.Drop.run([])
      end)

    assert output =~ "Dropping database: my_app"
    assert output =~ "Database my_app dropped"
  end
end
