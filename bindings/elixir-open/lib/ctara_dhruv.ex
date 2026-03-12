defmodule CtaraDhruv do
  @moduledoc """
  Open-source Elixir bindings for `ctara-dhruv-core`.
  """

  alias CtaraDhruv.Engine

  defdelegate new_engine(config), to: Engine, as: :new
end
