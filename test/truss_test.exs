defmodule TrussTest do
  use ExUnit.Case
  doctest Truss

  test "greets the world" do
    assert Truss.hello() == :world
  end
end
