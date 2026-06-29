defmodule Truss.Test.PostgrexMock do
  def start_link(_opts) do
    case Agent.start_link(fn -> %{applied: MapSet.new()} end, name: __MODULE__) do
      {:ok, pid} -> {:ok, pid}
      {:error, {:already_started, pid}} -> {:ok, pid}
    end
  end

  def query!(conn, "SELECT version FROM schema_migrations ORDER BY version", _params)
      when is_pid(conn) do
    applied = Agent.get(__MODULE__, fn state -> state.applied end)
    rows = Enum.map(applied, &[&1])

    %Postgrex.Result{rows: rows, columns: ["version"], command: :select, num_rows: length(rows)}
  end

  def query!(conn, "INSERT INTO schema_migrations (version) VALUES ($1)", [version])
      when is_pid(conn) do
    Agent.update(__MODULE__, fn state ->
      %{state | applied: MapSet.put(state.applied, version)}
    end)

    %Postgrex.Result{rows: [], columns: [], command: :insert, num_rows: 1}
  end

  def query!(_conn, _sql, _params) do
    %Postgrex.Result{rows: [], columns: [], command: :select, num_rows: 0}
  end

  def query(conn, sql, params) do
    {:ok, query!(conn, sql, params)}
  end

  def seed(versions) do
    ensure_started!()
    Agent.update(__MODULE__, fn _ -> %{applied: MapSet.new(versions)} end)
  end

  def reset! do
    if pid = Process.whereis(__MODULE__) do
      Agent.update(pid, fn _ -> %{applied: MapSet.new()} end)
    end
  end

  defp ensure_started! do
    unless Process.whereis(__MODULE__) do
      start_link([])
    end
  end
end
