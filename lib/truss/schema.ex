defmodule Truss.Schema do
  @moduledoc """
  Composable schemas and values.

  Use these when appropriate to combine with other schemas, or as values of fields.
  """

  alias Uniq.UUID

  @doc """
  Defines the common id schema.
  """
  def id do
    Zoi.uuid()
  end

  @doc """
  Creates a new unique id to use.
  """
  def new_id do
    UUID.uuid7()
  end

  @doc """
  Defines a non empty string schema
  """
  def non_empty_string do
    Zoi.string()
    |> Zoi.min(1)
    |> Zoi.transform(&String.trim/1)
    |> Zoi.regex(~r/\S/)
  end

  @doc """
  Schema for paginated list queries.

  Fields:
  - `offset` — non-negative integer, defaults to 0
  - `limit` — integer 1–100, defaults to 20

  Coerces string inputs (e.g. `"10"` → `10`).
  """
  def pagination do
    Zoi.map(%{
      offset: Zoi.default(Zoi.min(Zoi.integer(coerce: true), 0), 0),
      limit: Zoi.default(Zoi.max(Zoi.min(Zoi.integer(coerce: true), 1), 100), 20)
    })
  end

  @doc """
  Schema for an n-dimensional float vector.

  Accepts a list of floats of exactly `dimensions` length.
  Used for pgvector embeddings.
  """
  def vector(dimensions) do
    Zoi.length(Zoi.array(Zoi.float()), dimensions)
  end
end
