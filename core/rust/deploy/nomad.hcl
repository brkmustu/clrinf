job "clrinfrs" {
  datacenters = ["dc1"]
  type        = "service"

  # Memory state is process-local. Use a durable adapter before scaling replicas.
  group "generic-http" {
    count = 1

    network {
      port "http" {
        to = 8080
      }
    }

    service {
      name = "clrinfrs-generic"
      port = "http"

      check {
        type     = "http"
        path     = "/health"
        interval = "10s"
        timeout  = "2s"
      }
    }

    task "server" {
      driver = "docker"

      config {
        image = "clrinf/clrinfrs:latest"
        ports = ["http"]
      }

      env {
        PORT     = "8080"
        RUST_LOG = "info"
      }

      resources {
        cpu    = 256
        memory = 256
      }
    }
  }
}
