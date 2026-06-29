defmodule Truss.Workspace do
  @moduledoc """
  Workspace domain operations.
  """
  alias Truss.Schema

  def create(_scope, payload) do
    with {:ok, params} <- Zoi.parse(workspace_schema(), payload) do
      event = Map.put(params, :type, "workspace_created")
      Phoenix.PubSub.broadcast(Truss.PubSub, "workspace", {:event, event})
    end
  end

  defp workspace_schema do
    Zoi.map(%{
      id: Schema.id(),
      title: Zoi.string(),
      description: Zoi.optional(Zoi.string())
    })
  end
end
