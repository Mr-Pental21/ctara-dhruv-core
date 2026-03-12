defmodule CtaraDhruv.Engine do
  @moduledoc """
  Engine lifecycle and mutable wrapper state.
  """

  alias CtaraDhruv.Native

  @enforce_keys [:resource]
  defstruct [:resource]

  @type t :: %__MODULE__{resource: reference()}

  def new(config) when is_map(config) do
    with {:ok, resource} <- Native.engine_new(config) do
      {:ok, %__MODULE__{resource: resource}}
    end
  end

  def close(%__MODULE__{} = engine), do: Native.engine_close(engine.resource)

  def load_config(%__MODULE__{} = engine, path),
    do: Native.engine_load_config(engine.resource, %{path: path})

  def clear_config(%__MODULE__{} = engine), do: Native.engine_clear_config(engine.resource)

  def load_eop(%__MODULE__{} = engine, path),
    do: Native.engine_load_eop(engine.resource, %{path: path})

  def clear_eop(%__MODULE__{} = engine), do: Native.engine_clear_eop(engine.resource)

  def load_tara_catalog(%__MODULE__{} = engine, path),
    do: Native.engine_load_tara_catalog(engine.resource, %{path: path})

  def reset_tara_catalog(%__MODULE__{} = engine),
    do: Native.engine_reset_tara_catalog(engine.resource)

  def set_time_policy(%__MODULE__{} = engine, policy),
    do: Native.engine_set_time_policy(engine.resource, policy)
end
