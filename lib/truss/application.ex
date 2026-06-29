defmodule Truss.Application do
  @moduledoc false
  use Application

  @impl true
  def start(_type, _args) do
    children = [
      {Phoenix.PubSub, name: Truss.PubSub, adapter: Phoenix.PubSub.PG2},
      {Postgrex, Truss.Config.Database.fetch!()}
    ]

    Supervisor.start_link(children, strategy: :one_for_one, name: Truss.Supervisor)
  end
end
