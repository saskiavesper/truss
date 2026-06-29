import Config

# Database is configured via environment variables (12-factor app).
# Copy dev/env/.env.example to .env and adjust values for your local setup.
# In production, set these vars directly in the runtime environment.

config :truss, :infrastructure,
  database: [
    hostname: System.get_env("POSTGRES_HOST"),
    port: System.get_env("POSTGRES_PORT"),
    username: System.get_env("POSTGRES_USER"),
    password: System.get_env("POSTGRES_PASSWORD"),
    database: System.get_env("POSTGRES_DATABASE"),
    pool_size: System.get_env("POOL_SIZE"),
    types: Truss.PostgrexTypes,
    name: Truss.Database
  ]

config :logger, level: :info
