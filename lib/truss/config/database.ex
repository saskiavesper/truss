defmodule Truss.Config.Database do
  @moduledoc """
  Zoi-validated database configuration read from `Application.get_env`.

  Expects the following keys under `:truss, :infrastructure, :database`:

    * `hostname` — PostgreSQL host (string)
    * `port` — PostgreSQL port (integer, 1–65535, coerced from string)
    * `username` — PostgreSQL user (string)
    * `password` — PostgreSQL password (string)
    * `database` — PostgreSQL database name (string)
  """

  @schema Zoi.map(%{
            hostname: Zoi.string(),
            port:
              Zoi.max(
                Zoi.min(Zoi.integer(coerce: true), 1),
                65_535
              ),
            username: Zoi.string(),
            password: Zoi.string(),
            database: Zoi.string()
          })

  @doc """
  Returns the validated database configuration map.

  Raises if the config key is missing, nil, or fails validation.
  """
  def fetch! do
    config = Application.get_env(:truss, :infrastructure, [])[:database]

    case Zoi.parse(@schema, config) do
      {:ok, parsed} ->
        parsed

      {:error, errors} ->
        raise ArgumentError, "invalid database config: #{inspect(errors)}"
    end
  end
end
