defmodule CtaraDhruv.Error do
  @moduledoc false

  defexception [:kind, :message, details: %{}]

  @type t :: %__MODULE__{
          kind: atom() | String.t(),
          message: String.t(),
          details: map()
        }

  @spec from_term(map()) :: t()
  def from_term(%{} = term) do
    %__MODULE__{
      kind: Map.get(term, :kind, :unknown),
      message: Map.get(term, :message, "unknown error"),
      details: Map.get(term, :details, %{})
    }
  end
end
