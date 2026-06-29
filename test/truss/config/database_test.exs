defmodule Truss.Config.DatabaseTest do
  use ExUnit.Case, async: true

  setup do
    original = Application.get_env(:truss, :infrastructure)
    on_exit(fn -> Application.put_env(:truss, :infrastructure, original) end)
  end

  describe "fetch!/0" do
    test "returns validated config map" do
      Application.put_env(:truss, :infrastructure,
        database: %{
          hostname: "localhost",
          port: 5432,
          username: "postgres",
          password: "secret",
          database: "truss_test"
        }
      )

      config = Truss.Config.Database.fetch!()
      assert config.hostname == "localhost"
      assert config.port == 5432
      assert config.username == "postgres"
      assert config.password == "secret"
      assert config.database == "truss_test"
    end

    test "coerces string port to integer" do
      Application.put_env(:truss, :infrastructure,
        database: %{
          hostname: "localhost",
          port: "5432",
          username: "postgres",
          password: "postgres",
          database: "truss_test"
        }
      )

      assert Truss.Config.Database.fetch!().port == 5432
    end

    test "raises when database key is missing" do
      Application.put_env(:truss, :infrastructure, %{})

      assert_raise ArgumentError, ~r/invalid database config/, fn ->
        Truss.Config.Database.fetch!()
      end
    end

    test "raises when database is nil" do
      Application.put_env(:truss, :infrastructure, database: nil)

      assert_raise ArgumentError, fn ->
        Truss.Config.Database.fetch!()
      end
    end

    test "raises on port below 1" do
      Application.put_env(:truss, :infrastructure,
        database: %{
          hostname: "localhost",
          port: 0,
          username: "postgres",
          password: "postgres",
          database: "truss_test"
        }
      )

      assert_raise ArgumentError, fn ->
        Truss.Config.Database.fetch!()
      end
    end

    test "raises on port above 65535" do
      Application.put_env(:truss, :infrastructure,
        database: %{
          hostname: "localhost",
          port: 70_000,
          username: "postgres",
          password: "postgres",
          database: "truss_test"
        }
      )

      assert_raise ArgumentError, fn ->
        Truss.Config.Database.fetch!()
      end
    end

    test "raises on missing required fields" do
      Application.put_env(:truss, :infrastructure, database: %{hostname: "localhost"})

      assert_raise ArgumentError, fn ->
        Truss.Config.Database.fetch!()
      end
    end
  end
end
