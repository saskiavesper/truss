import Config

config :truss, :infrastructure,
  database: [
    hostname: "localhost",
    port: 5432,
    username: "postgres",
    password: "postgres",
    database: "truss_dev"
  ]

config :logger, level: :info
