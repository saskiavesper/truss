import Config

config :truss, :infrastructure,
  arango: [
    url: "http://localhost:8529",
    user: "root",
    password: "truss_password",
    database: "truss"
  ],
  rabbitmq: [
    host: "localhost",
    port: 5672,
    user: "guest",
    password: "guest",
    vhost: "/"
  ]

config :logger, level: :info
