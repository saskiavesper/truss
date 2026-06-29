import Config

if config_env() == :prod do
  arango_url = System.get_env("TRUSS_ARANGO_URL", "http://localhost:8529")
  arango_user = System.get_env("TRUSS_ARANGO_USER", "root")
  arango_password = System.get_env("TRUSS_ARANGO_PASSWORD", "truss_password")
  arango_database = System.get_env("TRUSS_ARANGO_DATABASE", "truss")

  rabbitmq_host = System.get_env("TRUSS_RABBITMQ_HOST", "localhost")
  rabbitmq_port = String.to_integer(System.get_env("TRUSS_RABBITMQ_PORT", "5672"))
  rabbitmq_user = System.get_env("TRUSS_RABBITMQ_USER", "guest")
  rabbitmq_password = System.get_env("TRUSS_RABBITMQ_PASSWORD", "guest")
  rabbitmq_vhost = System.get_env("TRUSS_RABBITMQ_VHOST", "/")

  config :truss, :infrastructure,
    arango: [
      url: arango_url,
      user: arango_user,
      password: arango_password,
      database: arango_database
    ],
    rabbitmq: [
      host: rabbitmq_host,
      port: rabbitmq_port,
      user: rabbitmq_user,
      password: rabbitmq_password,
      vhost: rabbitmq_vhost
    ]
end
