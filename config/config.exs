import Config

config :truss, :infrastructure,
  database: [
    hostname: "localhost",
    port: 5432,
    username: "postgres",
    password: "postgres",
    database: "truss_dev",
    pool_size: 10,
    types: Truss.PostgrexTypes,
    name: Truss.Database
  ]

config :logger, level: :info
