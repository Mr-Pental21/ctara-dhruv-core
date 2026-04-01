defmodule CtaraDhruv.MixProject do
  use Mix.Project

  def project do
    [
      app: :ctara_dhruv,
      version: "0.1.0",
      description: "Open-source Elixir bindings for ctara-dhruv-core",
      elixir: "~> 1.19",
      start_permanent: Mix.env() == :prod,
      source_url: "https://github.com/Mr-Pental21/ctara-dhruv-core",
      homepage_url: "https://github.com/Mr-Pental21/ctara-dhruv-core",
      package: package(),
      deps: deps()
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:rustler, "~> 0.37"}
    ]
  end

  defp package do
    [
      licenses: ["MIT"],
      links: %{
        "GitHub" => "https://github.com/Mr-Pental21/ctara-dhruv-core"
      },
      files: [
        "lib",
        "native",
        "mix.exs",
        "mix.lock",
        "README.md"
      ]
    ]
  end
end
