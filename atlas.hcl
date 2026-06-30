variable "pg_host" {
  type    = string
  default = getenv("POSTGRES_HOST")
}

variable "pg_port" {
  type    = string
  default = getenv("POSTGRES_PORT")
}

variable "pg_user" {
  type    = string
  default = getenv("POSTGRES_USER")
}

variable "pg_password" {
  type    = string
  default = getenv("POSTGRES_PASSWORD")
}

variable "pg_database" {
  type = string
  default = getenv("POSTGRES_DATABASE")
}

locals {
  base_url = "postgres://${var.pg_user}:${var.pg_password}@${var.pg_host}:${var.pg_port}"
  query    = "search_path=public&sslmode=disable"
}

env "dev" {
  src = "file://priv/schema.sql"
  url = "${local.base_url}/${var.pg_database}?${local.query}"
  dev = "docker+postgres://pgvector/pgvector:0.8.3-pg18-trixie/dev?search_path=public"
  migration {
    dir = "file://priv/migrations"
  }
}

env "test" {
  src = "file://priv/schema.sql"
  url = "${local.base_url}/${var.pg_database}?${local.query}"
  migration {
    dir = "file://priv/migrations"
  }
}

env "ci" {
  src = "file://priv/schema.sql"
  url = "${local.base_url}/${var.pg_database}?${local.query}"
  dev = "${local.base_url}/${var.pg_database}?${local.query}"
  migration {
    dir = "file://priv/migrations"
  }
}
