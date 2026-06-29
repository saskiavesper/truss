import Config

if config_env() == :prod do
  postgres_host = System.get_env("POSTGRES_HOST", "localhost")
  postgres_port = String.to_integer(System.get_env("POSTGRES_PORT", "5432"))
  postgres_user = System.get_env("POSTGRES_USER", "postgres")
  postgres_password = System.get_env("POSTGRES_PASSWORD", "postgres")
  postgres_database = System.get_env("POSTGRES_DATABASE", "truss_dev")

  config :truss, :infrastructure,
    database: [
      hostname: postgres_host,
      port: postgres_port,
      username: postgres_user,
      password: postgres_password,
      database: postgres_database
    ]
end
