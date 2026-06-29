defmodule Truss.SchemaTest do
  use ExUnit.Case, async: true
  alias Truss.Schema

  test "new_id/0 creates a valid id by id/0" do
    schema = Schema.id()
    created = Schema.new_id()

    assert {:ok, ^created} = Zoi.parse(schema, created)
  end

  test "non_empty_string doesnt accept blank" do
    blank = " "
    schema = Schema.non_empty_string()
    assert {:error, _} = Zoi.parse(schema, blank)
  end

  test "non_empty_string accepts any text" do
    text = "erwp[oíjgfq30ot98ij432skdaolnmafd"
    schema = Schema.non_empty_string()
    assert {:ok, ^text} = Zoi.parse(schema, text)
  end

  test "pagination defaults offset to 0 and limit to 20" do
    schema = Schema.pagination()
    assert {:ok, %{offset: 0, limit: 20}} = Zoi.parse(schema, %{})
  end

  test "pagination coerces string values" do
    schema = Schema.pagination()
    assert {:ok, %{offset: 5, limit: 10}} = Zoi.parse(schema, %{offset: "5", limit: "10"})
  end

  test "pagination rejects limit above 100" do
    schema = Schema.pagination()
    assert {:error, _} = Zoi.parse(schema, %{limit: 200})
  end

  test "pagination rejects negative offset" do
    schema = Schema.pagination()
    assert {:error, _} = Zoi.parse(schema, %{offset: -1})
  end

  test "vector accepts a list of the correct length" do
    schema = Schema.vector(3)
    assert {:ok, [1.0, 2.0, 3.0]} = Zoi.parse(schema, [1.0, 2.0, 3.0])
  end

  test "vector rejects wrong length" do
    schema = Schema.vector(3)
    assert {:error, _} = Zoi.parse(schema, [1, 2])
  end

  test "vector rejects non-numeric values" do
    schema = Schema.vector(2)
    assert {:error, _} = Zoi.parse(schema, ["a", "b"])
  end
end
